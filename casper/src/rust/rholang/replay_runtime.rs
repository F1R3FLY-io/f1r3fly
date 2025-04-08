// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeReplaySyntax.scala

use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::{Arc, Mutex},
};

use models::rust::{
    block::state_hash::StateHash,
    block_hash::BlockHash,
    casper::protocol::casper_message::{
        Event, ProcessedDeploy, ProcessedSystemDeploy, SystemDeployData,
    },
    validator::Validator,
};
use rholang::rust::interpreter::{
    interpreter::EvaluateResult,
    rho_runtime::{RhoRuntime, RhoRuntimeImpl},
    system_processes::BlockData,
};
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash, history::Either,
    merger::merging_logic::NumberChannelsEndVal,
};

use crate::rust::{
    errors::CasperError,
    util::{
        event_converter,
        rholang::{
            costacc::{
                close_block_deploy::CloseBlockDeploy, pre_charge_deploy::PreChargeDeploy,
                refund_deploy::RefundDeploy, slash_deploy::SlashDeploy,
            },
            interpreter_util,
            replay_failure::ReplayFailure,
            system_deploy::SystemDeployTrait,
            system_deploy_util,
        },
    },
};

use super::runtime::{RuntimeOps, SysEvalResult};

pub struct ReplayRuntimeOps;

impl ReplayRuntimeOps {
    /* REPLAY Compute state with deploys (genesis block) and System deploys (regular block) */

    /**
     * Evaluates (and validates) deploys and System deploys with checkpoint to valiate final state hash
     */
    pub async fn replay_compute_state(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: &StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        block_data: &BlockData,
        invalid_blocks: Option<HashMap<BlockHash, Validator>>,
        is_genesis: bool, //FIXME have a better way of knowing this. Pass the replayDeploy function maybe? - OLD
    ) -> Result<(Blake2b256Hash, Vec<NumberChannelsEndVal>), CasperError> {
        let invalid_blocks = invalid_blocks.unwrap_or_default();

        let runtime_lock = runtime.lock().unwrap();
        runtime_lock.set_block_data(block_data.clone());
        runtime_lock.set_invalid_blocks(invalid_blocks);
        drop(runtime_lock);

        Self::replay_deploys(
            runtime.clone(),
            start_hash,
            terms,
            system_deploys,
            |processed_deploy| {
                Self::replay_deploy_e(runtime.clone(), !is_genesis, processed_deploy)
            },
            |processed_system_deploy| {
                Self::replay_block_system_deploy(
                    runtime.clone(),
                    block_data,
                    processed_system_deploy,
                )
            },
        )
        .await
    }

    /* REPLAY Deploy evaluators */

    /**
     * Evaluates (and validates) deploys on root hash with checkpoint to validate final state hash
     */
    pub async fn replay_deploys<FutRd, FutRsd, Rd, Rsd>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: &StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        replay_deploy: Rd,
        replay_system_deploy: Rsd,
    ) -> Result<(Blake2b256Hash, Vec<NumberChannelsEndVal>), CasperError>
    where
        Rd: Fn(ProcessedDeploy) -> FutRd,
        Rsd: Fn(ProcessedSystemDeploy) -> FutRsd,
        FutRd: Future<Output = Result<NumberChannelsEndVal, CasperError>>,
        FutRsd: Future<Output = Result<NumberChannelsEndVal, CasperError>>,
    {
        {
            let mut runtime_lock = runtime.lock().unwrap();
            runtime_lock.reset(Blake2b256Hash::from_bytes_prost(start_hash));
        }

        let mut remaining_terms = terms;
        let mut deploy_results = Vec::new();
        while !remaining_terms.is_empty() {
            let term = remaining_terms.remove(0);
            match replay_deploy(term).await {
                Ok(value) => {
                    deploy_results.push(value);
                }
                Err(err) => return Err(err),
            }
        }

        let mut remaining_system_deploys = system_deploys;
        let mut system_deploy_results = Vec::new();
        while !remaining_system_deploys.is_empty() {
            let system_deploy = remaining_system_deploys.remove(0);
            match replay_system_deploy(system_deploy).await {
                Ok(value) => {
                    system_deploy_results.push(value);
                }
                Err(err) => return Err(err),
            }
        }

        let mut all_mergeable = Vec::new();
        all_mergeable.extend(deploy_results);
        all_mergeable.extend(system_deploy_results);

        let mut runtime_lock = runtime.lock().unwrap();
        let checkpoint = runtime_lock.create_checkpoint();
        Ok((checkpoint.root, all_mergeable))
    }

    /**
     * REPLAY Evaluates deploy
     */
    pub async fn replay_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        with_cost_accounting: bool,
        processed_deploy: ProcessedDeploy,
    ) -> Option<CasperError> {
        match Self::replay_deploy_e(runtime, with_cost_accounting, processed_deploy).await {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    pub async fn replay_deploy_e(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        with_cost_accounting: bool,
        processed_deploy: ProcessedDeploy,
    ) -> Result<NumberChannelsEndVal, CasperError> {
        let mergeable_channels = Arc::new(Mutex::new(HashSet::new()));

        let combined_evaluator = || async {
            if with_cost_accounting {
                let precharge_result = {
                    let mut pre_charge_deploy = PreChargeDeploy {
                        charge_amount: processed_deploy.deploy.data.total_phlo_charge(),
                        pk: processed_deploy.deploy.pk.clone(),
                        rand: system_deploy_util::generate_pre_charge_deploy_random_seed(
                            &processed_deploy.deploy,
                        ),
                    };
                    Self::replay_system_deploy_internal(
                        runtime.clone(),
                        &mut pre_charge_deploy,
                        &processed_deploy.system_deploy_error,
                    )
                    .await
                };

                match precharge_result {
                    Ok((_, mut system_eval_result)) => {
                        {
                            let mut rt_lock = runtime.lock().unwrap();
                            rt_lock.create_soft_checkpoint();
                        }
                        if system_eval_result.errors.is_empty() {
                            let mut mc_lock = mergeable_channels.lock().unwrap();
                            mc_lock.extend(system_eval_result.mergeable.drain());
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                };

                if processed_deploy.system_deploy_error.is_none() {
                    let soft_tx_result = RuntimeOps::with_soft_transaction(runtime.clone(), || async {
                        let mut user_eval_result =
                            RuntimeOps::evaluate(runtime.clone(), &processed_deploy.deploy).await?;

                        let eval_successful = user_eval_result.errors.is_empty();
                        /* Since the state of `replaySpace` is reset on each invocation of `replayComputeState`,
                        and `ReplayFailure`s mean that block processing is cancelled upstream, we only need to
                        reset state if the replay effects of valid deploys need to be discarded. */
                        if !eval_successful {
                            interpreter_util::print_deploy_errors(
                                &processed_deploy.deploy.sig,
                                &user_eval_result.errors,
                            );
                        }

                        if eval_successful {
                            let mut mc_lock = mergeable_channels.lock().unwrap();
                            mc_lock.extend(user_eval_result.mergeable.drain());
                        }

                        Ok((user_eval_result, eval_successful))
                    })
                    .await
                    .map(|res| {
                        if processed_deploy.is_failed != !res.errors.is_empty() {
                            return Err(CasperError::ReplayFailure(
                                ReplayFailure::replay_status_mismatch(
                                    processed_deploy.is_failed,
                                    !res.errors.is_empty(),
                                ),
                            ));
                        }

                        if processed_deploy.cost.cost != res.cost.value as u64 {
                            return Err(CasperError::ReplayFailure(
                                ReplayFailure::replay_cost_mismatch(
                                    processed_deploy.cost.cost,
                                    res.cost.value as u64,
                                ),
                            ));
                        }

                        Ok(res)
                    })??;

                    let eval_successful = soft_tx_result.errors.is_empty();
                    if eval_successful {
                        let mut rt_lock = runtime.lock().unwrap();
                        rt_lock.create_soft_checkpoint();
                    }

                    let refund_result = {
                        let mut refund_deploy = RefundDeploy {
                            refund_amount: processed_deploy.refund_amount(),
                            rand: system_deploy_util::generate_refund_deploy_random_seed(
                                &processed_deploy.deploy,
                            ),
                        };

                        Self::replay_system_deploy_internal(
                            runtime.clone(),
                            &mut refund_deploy,
                            &None,
                        )
                        .await
                    };

                    match refund_result {
                        Ok((_, mut system_eval_result)) => {
                            {
                                let mut rt_lock = runtime.lock().unwrap();
                                rt_lock.create_soft_checkpoint();
                            }

                            if system_eval_result.errors.is_empty() {
                                {
                                    let mut mc_lock = mergeable_channels.lock().unwrap();
                                    mc_lock.extend(system_eval_result.mergeable.drain());
                                }
                            }
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }

                    Ok(((), eval_successful))
                } else {
                    Ok(((), true))
                }
            } else {
                let soft_tx_result = RuntimeOps::with_soft_transaction(runtime.clone(), || async {
                    let mut user_eval_result =
                        RuntimeOps::evaluate(runtime.clone(), &processed_deploy.deploy).await?;

                    let eval_successful = user_eval_result.errors.is_empty();
                    /* Since the state of `replaySpace` is reset on each invocation of `replayComputeState`,
                    and `ReplayFailure`s mean that block processing is cancelled upstream, we only need to
                    reset state if the replay effects of valid deploys need to be discarded. */
                    if !eval_successful {
                        interpreter_util::print_deploy_errors(
                            &processed_deploy.deploy.sig,
                            &user_eval_result.errors,
                        );
                    }

                    if eval_successful {
                        let mut mc_lock = mergeable_channels.lock().unwrap();
                        mc_lock.extend(user_eval_result.mergeable.drain());
                    }

                    Ok((user_eval_result, eval_successful))
                })
                .await
                .map(|res| {
                    if processed_deploy.is_failed != !res.errors.is_empty() {
                        return Err(CasperError::ReplayFailure(
                            ReplayFailure::replay_status_mismatch(
                                processed_deploy.is_failed,
                                !res.errors.is_empty(),
                            ),
                        ));
                    }

                    if processed_deploy.cost.cost != res.cost.value as u64 {
                        return Err(CasperError::ReplayFailure(
                            ReplayFailure::replay_cost_mismatch(
                                processed_deploy.cost.cost,
                                res.cost.value as u64,
                            ),
                        ));
                    }

                    Ok(res)
                })??;

                Ok(((), soft_tx_result.errors.is_empty()))
            }
        };

        Self::rig_with_check(
            runtime.clone(),
            processed_deploy.clone(),
            combined_evaluator,
        )
        .await?;

        let final_mergeable = mergeable_channels.lock().unwrap().clone();
        let channels_data =
            RuntimeOps::get_number_channels_data(runtime.clone(), &final_mergeable)?;

        Ok(channels_data)
    }

    /* REPLAY System deploy evaluators */

    /**
     * Evaluates System deploy with checkpoint to get final state hash
     */
    pub async fn replay_block_system_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        block_data: &BlockData,
        processed_system_deploy: ProcessedSystemDeploy,
    ) -> Result<NumberChannelsEndVal, CasperError> {
        let system_deploy = match processed_system_deploy {
            ProcessedSystemDeploy::Succeeded {
                ref system_deploy, ..
            } => system_deploy,
            ProcessedSystemDeploy::Failed { .. } => &SystemDeployData::Empty,
        };

        match system_deploy {
            SystemDeployData::Slash {
                invalid_block_hash,
                issuer_public_key,
            } => {
                let mut slash_deploy = SlashDeploy {
                    invalid_block_hash: invalid_block_hash.clone(),
                    pk: issuer_public_key.clone(),
                    initial_rand: system_deploy_util::generate_slash_deploy_random_seed(
                        block_data.sender.bytes.clone(),
                        block_data.seq_num,
                    ),
                };

                Ok(Self::rig_with_check_system_deploy(
                    runtime.clone(),
                    processed_system_deploy,
                    || async {
                        Self::replay_system_deploy_internal(
                            runtime.clone(),
                            &mut slash_deploy,
                            &None,
                        )
                        .await
                        .map(|(_, eval_result)| {
                            if eval_result.errors.is_empty() {
                                let mut runtime_lock = runtime.lock().unwrap();
                                runtime_lock.create_soft_checkpoint();
                            }

                            let data = RuntimeOps::get_number_channels_data(
                                runtime.clone(),
                                &eval_result.mergeable,
                            )?;
                            Ok((data, eval_result))
                        })?
                    },
                )
                .await?
                .0)
            }

            SystemDeployData::CloseBlockSystemDeployData => {
                let mut close_block_deploy = CloseBlockDeploy {
                    initial_rand:
                        system_deploy_util::generate_close_deploy_random_seed_from_validator(
                            block_data.sender.bytes.clone(),
                            block_data.seq_num,
                        ),
                };

                Ok(Self::rig_with_check_system_deploy(
                    runtime.clone(),
                    processed_system_deploy,
                    || async {
                        Self::replay_system_deploy_internal(
                            runtime.clone(),
                            &mut close_block_deploy,
                            &None,
                        )
                        .await
                        .map(|(_, eval_result)| {
                            if eval_result.errors.is_empty() {
                                let mut runtime_lock = runtime.lock().unwrap();
                                runtime_lock.create_soft_checkpoint();
                            }

                            let data = RuntimeOps::get_number_channels_data(
                                runtime.clone(),
                                &eval_result.mergeable,
                            )?;
                            Ok((data, eval_result))
                        })?
                    },
                )
                .await?
                .0)
            }

            SystemDeployData::Empty => Err(CasperError::ReplayFailure(
                ReplayFailure::internal_error("Expected system deploy".to_string()),
            )),
        }
    }

    pub async fn replay_system_deploy_internal<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
        expected_failure_msg: &Option<String>,
    ) -> Result<SysEvalResult<S>, CasperError> {
        let (result, eval_res) = RuntimeOps::eval_system_deploy(runtime, system_deploy).await?;

        // Compare evaluation from play and replay, successful or failed
        match (expected_failure_msg, &result) {
            // Valid replay
            (None, Either::Right(_)) => {
                // Replayed successful execution
                Ok((result, eval_res))
            }
            (Some(expected_error), Either::Left(error)) => {
                let actual_error = &error.error_message;
                if expected_error == actual_error {
                    // Replayed failed execution - error messages match
                    Ok((result, eval_res))
                } else {
                    // Error messages different
                    Err(CasperError::ReplayFailure(
                        ReplayFailure::system_deploy_error_mismatch(
                            expected_error.clone(),
                            actual_error.clone(),
                        ),
                    ))
                }
            }
            // Invalid replay
            (Some(_), Either::Right(_)) => {
                // Error expected, replay successful
                Err(CasperError::ReplayFailure(
                    ReplayFailure::replay_status_mismatch(true, false),
                ))
            }
            (None, Either::Left(_)) => {
                // No error expected, replay failed
                Err(CasperError::ReplayFailure(
                    ReplayFailure::replay_status_mismatch(false, true),
                ))
            }
        }
    }

    /* Helper functions */

    pub async fn rig_with_check<A, F, Fut>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_deploy: ProcessedDeploy,
        action: F,
    ) -> Result<(A, bool), CasperError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(A, bool), CasperError>>,
    {
        // Rig the events first (synchronous operation)
        Self::rig(runtime.clone(), processed_deploy)?;

        // Execute the provided async action
        let action_result = action().await;

        match action_result {
            Ok((value, eval_successful)) => {
                match Self::check_replay_data_with_fix(runtime.clone(), eval_successful) {
                    Ok(_) => Ok((value, eval_successful)),
                    Err(replay_failure) => Err(CasperError::ReplayFailure(replay_failure)),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn rig_with_check_system_deploy<A, F, Fut>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_system_deploy: ProcessedSystemDeploy,
        action: F,
    ) -> Result<(A, EvaluateResult), CasperError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(A, EvaluateResult), CasperError>>,
    {
        Self::rig_system_deploy(runtime.clone(), processed_system_deploy)?;
        let (value, eval_res) = action().await?;
        Self::check_replay_data_with_fix(runtime, eval_res.errors.is_empty())?;
        Ok((value, eval_res))
    }

    pub fn rig(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_deploy: ProcessedDeploy,
    ) -> Result<(), CasperError> {
        let runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock.rig(
            processed_deploy
                .deploy_log
                .iter()
                .map(event_converter::to_rspace_event)
                .collect(),
        )?)
    }

    pub fn rig_system_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_system_deploy: ProcessedSystemDeploy,
    ) -> Result<(), CasperError> {
        let event_list = match processed_system_deploy {
            ProcessedSystemDeploy::Succeeded { event_list, .. } => event_list,
            ProcessedSystemDeploy::Failed { event_list, .. } => event_list,
        };

        let runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock.rig(
            event_list
                .into_iter()
                .map(|event: Event| event_converter::to_rspace_event(&event))
                .collect(),
        )?)
    }

    pub fn check_replay_data_with_fix(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        // https://rchain.atlassian.net/browse/RCHAIN-3505
        _eval_successful: bool,
    ) -> Result<(), ReplayFailure> {
        // Only check replay data for successful evaluations
        let runtime_lock = runtime.lock().unwrap();
        match runtime_lock.check_replay_data() {
            Ok(()) => Ok(()),
            Err(err) => {
                let err_msg = err.to_string();
                if err_msg.contains("unused") && err_msg.contains("COMM") {
                    Err(ReplayFailure::unused_comm_event(err_msg))
                } else {
                    Err(ReplayFailure::internal_error(format!(
                        "Replay check failed: {}",
                        err
                    )))
                }
            }
        }
    }
}
