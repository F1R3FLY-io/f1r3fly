// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeReplaySyntax.scala

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use models::rust::{
    block::state_hash::StateHash,
    block_hash::BlockHash,
    casper::protocol::casper_message::{ProcessedDeploy, ProcessedSystemDeploy, SystemDeployData},
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
    pub fn replay_compute_state(
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
    }

    /* REPLAY Deploy evaluators */

    /**
     * Evaluates (and validates) deploys on root hash with checkpoint to validate final state hash
     */
    pub fn replay_deploys(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: &StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        replay_deploy: impl Fn(&ProcessedDeploy) -> Result<NumberChannelsEndVal, CasperError>,
        replay_system_deploy: impl Fn(
            &ProcessedSystemDeploy,
        ) -> Result<NumberChannelsEndVal, CasperError>,
    ) -> Result<(Blake2b256Hash, Vec<NumberChannelsEndVal>), CasperError> {
        {
            let mut runtime_lock = runtime.lock().unwrap();
            runtime_lock.reset(Blake2b256Hash::from_bytes_prost(start_hash));
        }

        let mut remaining_terms = terms;
        let mut deploy_results = Vec::new();
        while let Some(term) = remaining_terms.first() {
            match replay_deploy(term) {
                Ok(value) => {
                    deploy_results.push(value);
                    remaining_terms.remove(0);
                }
                Err(err) => return Err(err),
            }
        }

        let mut remaining_system_deploys = system_deploys;
        let mut system_deploy_results = Vec::new();
        while let Some(system_deploy) = remaining_system_deploys.first() {
            match replay_system_deploy(system_deploy) {
                Ok(value) => {
                    system_deploy_results.push(value);
                    remaining_system_deploys.remove(0);
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
    pub fn replay_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        with_cost_accounting: bool,
        processed_deploy: &ProcessedDeploy,
    ) -> Option<CasperError> {
        match Self::replay_deploy_e(runtime, with_cost_accounting, processed_deploy) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    pub fn replay_deploy_e(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        with_cost_accounting: bool,
        processed_deploy: &ProcessedDeploy,
    ) -> Result<NumberChannelsEndVal, CasperError> {
        let mut mergeable_channels = HashSet::new();

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
            };

            match precharge_result {
                Ok((_, eval_result)) => {
                    let mut runtime_lock = runtime.lock().unwrap();
                    runtime_lock.create_soft_checkpoint();
                    if eval_result.errors.is_empty() {
                        mergeable_channels.extend(eval_result.mergeable.into_iter());
                    }
                }
                Err(err) => return Err(err),
            }
        }

        let deploy_result = {
            if processed_deploy.system_deploy_error.is_some() && with_cost_accounting {
                (EvaluateResult::default(), true)
            } else {
                let outer_result = RuntimeOps::with_soft_transaction(runtime.clone(), || {
                    let rt = tokio::runtime::Handle::current();
                    let result = rt.block_on(RuntimeOps::evaluate(
                        runtime.clone(),
                        &processed_deploy.deploy,
                    ))?;

                    /* Since the state of `replaySpace` is reset on each invocation of `replayComputeState`,
                    and `ReplayFailure`s mean that block processing is cancelled upstream, we only need to
                    reset state if the replay effects of valid deploys need to be discarded. */
                    if !result.errors.is_empty() {
                        interpreter_util::print_deploy_errors(
                            &processed_deploy.deploy.sig,
                            &result.errors,
                        )
                    }

                    // Collect user deploy mergeable channels if successful
                    if result.errors.is_empty() {
                        mergeable_channels.extend(result.mergeable.clone().into_iter());
                    }

                    Ok((result.clone(), result.errors.is_empty()))
                })?;

                (outer_result.clone(), outer_result.errors.is_empty())
            }
        };

        // Regardless of success or failure, verify that deploy status' match.
        if processed_deploy.is_failed != !deploy_result.1 {
            return Err(CasperError::ReplayFailure(
                ReplayFailure::replay_status_mismatch(processed_deploy.is_failed, deploy_result.1),
            ));
        }

        // Verify evaluation costs match
        if processed_deploy.cost.cost != deploy_result.0.cost.value as u64 {
            return Err(CasperError::ReplayFailure(
                ReplayFailure::replay_cost_mismatch(
                    processed_deploy.cost.cost,
                    deploy_result.0.cost.value as u64,
                ),
            ));
        }

        // Create a checkpoint if deploy was successful
        if deploy_result.1 {
            let mut runtime_lock = runtime.lock().unwrap();
            runtime_lock.create_soft_checkpoint();
        }

        // Handle refund if cost accounting is enabled
        if with_cost_accounting {
            let refund_result = {
                let mut refund_deploy = RefundDeploy {
                    refund_amount: processed_deploy.refund_amount(),
                    rand: system_deploy_util::generate_refund_deploy_random_seed(
                        &processed_deploy.deploy,
                    ),
                };

                Self::replay_system_deploy_internal(runtime.clone(), &mut refund_deploy, &None)
            };

            match refund_result {
                Ok((_, eval_result)) => {
                    // Collect refund mergeable channels if successful
                    if eval_result.errors.is_empty() {
                        let mut runtime_lock = runtime.lock().unwrap();
                        runtime_lock.create_soft_checkpoint();
                        mergeable_channels.extend(eval_result.mergeable.into_iter());
                    }
                }
                Err(err) => return Err(err),
            }
        }

        Self::rig_with_check(runtime.clone(), processed_deploy, || {
            Ok(
                Self::check_replay_data_with_fix(runtime.clone(), deploy_result.1)
                    .map(|_| ((), deploy_result.1))?,
            )
        })?;

        // Get number channels data from runtime
        let channels_data = RuntimeOps::get_number_channels_data(runtime, &mergeable_channels)?;

        Ok(channels_data)
    }

    /* REPLAY System deploy evaluators */

    /**
     * Evaluates System deploy with checkpoint to get final state hash
     */
    pub fn replay_block_system_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        block_data: &BlockData,
        processed_system_deploy: &ProcessedSystemDeploy,
    ) -> Result<NumberChannelsEndVal, CasperError> {
        let system_deploy = match processed_system_deploy {
            ProcessedSystemDeploy::Succeeded { system_deploy, .. } => system_deploy,
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
                    || {
                        Self::replay_system_deploy_internal(
                            runtime.clone(),
                            &mut slash_deploy,
                            &None,
                        )
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
                )?
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
                    || {
                        Self::replay_system_deploy_internal(
                            runtime.clone(),
                            &mut close_block_deploy,
                            &None,
                        )
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
                )?
                .0)
            }

            SystemDeployData::Empty => Err(CasperError::ReplayFailure(
                ReplayFailure::internal_error("Expected system deploy".to_string()),
            )),
        }
    }

    pub fn replay_system_deploy_internal<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
        expected_failure_msg: &Option<String>,
    ) -> Result<SysEvalResult<S>, CasperError> {
        let (result, eval_res) = RuntimeOps::eval_system_deploy(runtime, system_deploy)?;

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

    pub fn rig_with_check<A>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_deploy: &ProcessedDeploy,
        action: impl FnOnce() -> Result<(A, bool), CasperError>,
    ) -> Result<(A, bool), CasperError> {
        Self::rig(runtime.clone(), processed_deploy)?;
        let (value, eval_res) = action()?;
        Self::check_replay_data_with_fix(runtime, eval_res)?;
        Ok((value, eval_res))
    }

    pub fn rig_with_check_system_deploy<A>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_system_deploy: &ProcessedSystemDeploy,
        action: impl FnOnce() -> Result<(A, EvaluateResult), CasperError>,
    ) -> Result<(A, EvaluateResult), CasperError> {
        Self::rig_system_deploy(runtime.clone(), processed_system_deploy)?;
        let (value, eval_res) = action()?;
        Self::check_replay_data_with_fix(runtime, eval_res.errors.is_empty())?;
        Ok((value, eval_res))
    }

    pub fn rig(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        processed_deploy: &ProcessedDeploy,
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
        processed_system_deploy: &ProcessedSystemDeploy,
    ) -> Result<(), CasperError> {
        let event_list = match processed_system_deploy {
            ProcessedSystemDeploy::Succeeded { event_list, .. } => event_list,
            ProcessedSystemDeploy::Failed { event_list, .. } => event_list,
        };

        let runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock.rig(
            event_list
                .into_iter()
                .map(event_converter::to_rspace_event)
                .collect(),
        )?)
    }

    pub fn check_replay_data_with_fix(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        eval_successful: bool,
    ) -> Result<(), ReplayFailure> {
        if !eval_successful {
            // TODO: temp fix for replay error mismatch
            // https://rchain.atlassian.net/browse/RCHAIN-3505
            return Ok(());
        }

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
