// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeSyntax.scala

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crypto::rust::{
    public_key::PublicKey,
    signatures::{secp256k1::Secp256k1, signatures_alg::SignaturesAlg, signed::Signed},
};
use models::{
    rhoapi::{
        expr::ExprInstance, g_unforgeable::UnfInstance, tagged_continuation::TaggedCont,
        BindPattern, GPrivate, GUnforgeable, ListParWithRandom, Par, TaggedContinuation,
    },
    rust::{
        block::state_hash::StateHash,
        block_hash::BlockHash,
        casper::{
            pretty_printer::PrettyPrinter,
            protocol::casper_message::{
                Bond, DeployData, Event, ProcessedDeploy, ProcessedSystemDeploy, SystemDeployData,
            },
        },
        normalizer_env::normalizer_env_from_deploy,
        par_map_type_mapper::ParMapTypeMapper,
        par_set_type_mapper::ParSetTypeMapper,
        sorted_par_hash_set::SortedParHashSet,
        sorted_par_map::SortedParMap,
        utils::new_freevar_par,
        validator::Validator,
    },
};
use rholang::rust::interpreter::{
    accounting::costs::Cost,
    errors::InterpreterError,
    interpreter::EvaluateResult,
    merging::rholang_merging_logic::RholangMergingLogic,
    rho_runtime::{bootstrap_registry, RhoRuntime, RhoRuntimeImpl},
    system_processes::BlockData,
};
use rspace_plus_plus::rspace::{
    hashing::{blake2b256_hash::Blake2b256Hash, stable_hash_provider},
    history::{instances::radix_history::RadixHistory, Either},
    merger::merging_logic::NumberChannelsEndVal,
};

use crate::rust::{
    errors::CasperError,
    rholang::types::eval_collector::EvalCollector,
    util::{
        construct_deploy, event_converter,
        rholang::{
            costacc::{
                close_block_deploy::CloseBlockDeploy, pre_charge_deploy::PreChargeDeploy,
                refund_deploy::RefundDeploy, slash_deploy::SlashDeploy,
            },
            interpreter_util,
            system_deploy::SystemDeployTrait,
            system_deploy_result::SystemDeployResult,
            system_deploy_user_error::{SystemDeployPlatformFailure, SystemDeployUserError},
            system_deploy_util,
            tools::Tools,
        },
    },
};

pub struct RuntimeOps;

#[allow(type_alias_bounds)]
type SysEvalResult<S: SystemDeployTrait> =
    (Either<SystemDeployUserError, S::Result>, EvaluateResult);

fn system_deploy_consume_all_pattern() -> BindPattern {
    BindPattern {
        patterns: vec![new_freevar_par(0, Vec::new())],
        remainder: None,
        free_count: 1,
    }
}

impl RuntimeOps {
    /**
     * Because of the history legacy, the emptyStateHash does not really represent an empty trie.
     * The `emptyStateHash` is used as genesis block pre state which the state only contains registry
     * fixed channels in the state.
     */
    pub async fn empty_state_hash(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
    ) -> Result<StateHash, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(RadixHistory::empty_root_node_hash());
        drop(runtime_lock);

        bootstrap_registry(runtime.clone()).await;
        let mut runtime_lock = runtime.lock().unwrap();
        let checkpoint = runtime_lock.create_checkpoint();
        Ok(checkpoint.root.bytes().into())
    }

    /* Compute state with deploys (genesis block) and System deploys (regular block) */

    /**
     * Evaluates deploys and System deploys with checkpoint to get final state hash
     */
    pub fn compute_state(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: StateHash,
        terms: Vec<Signed<DeployData>>,
        system_deploys: Vec<impl SystemDeployTrait>,
        block_data: BlockData,
        invalid_blocks: HashMap<BlockHash, Validator>,
    ) -> Result<
        (
            StateHash,
            Vec<(ProcessedDeploy, NumberChannelsEndVal)>,
            Vec<(ProcessedSystemDeploy, NumberChannelsEndVal)>,
        ),
        CasperError,
    > {
        let runtime_lock = runtime.lock().unwrap();
        runtime_lock.set_block_data(block_data);
        runtime_lock.set_invalid_blocks(invalid_blocks);
        drop(runtime_lock);

        let deploy_process_result =
            Self::play_deploys(runtime.clone(), start_hash, terms, |deploy| {
                Self::play_deploy_with_cost_accounting(runtime.clone(), deploy)
            })?;

        let (start_hash, processed_deploys) = deploy_process_result;
        let system_deploy_process_result = system_deploys.into_iter().try_fold(
            (start_hash, Vec::new()),
            |(current_hash, mut processed_system_deploys_acc), mut system_deploy| {
                match Self::play_system_deploy(runtime.clone(), current_hash, &mut system_deploy)? {
                    SystemDeployResult::PlaySucceeded {
                        state_hash,
                        processed_system_deploy,
                        mergeable_channels,
                        result: _,
                    } => {
                        processed_system_deploys_acc
                            .push((processed_system_deploy, mergeable_channels));
                        Ok((state_hash, processed_system_deploys_acc))
                    }
                    SystemDeployResult::PlayFailed {
                        processed_system_deploy: ProcessedSystemDeploy::Failed { error_msg, .. },
                    } => Err(CasperError::RuntimeError(format!(
                        "Unexpected system error during play of system deploy: {}",
                        error_msg
                    ))),
                    SystemDeployResult::PlayFailed {
                        processed_system_deploy: ProcessedSystemDeploy::Succeeded { .. },
                    } => Err(CasperError::RuntimeError(format!(
                        "Unreachable code path. This is likely caused by a bug in the runtime."
                    ))),
                }
            },
        );

        let (post_state_hash, processed_system_deploys) = system_deploy_process_result?;

        Ok((post_state_hash, processed_deploys, processed_system_deploys))
    }

    /**
     * Evaluates genesis deploys with checkpoint to get final state hash
     */
    pub async fn compute_genesis(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        terms: Vec<Signed<DeployData>>,
        block_time: i64,
        block_number: i64,
    ) -> Result<
        (
            StateHash,
            StateHash,
            Vec<(ProcessedDeploy, NumberChannelsEndVal)>,
        ),
        CasperError,
    > {
        let runtime_lock = runtime.lock().unwrap();
        runtime_lock.set_block_data(BlockData {
            time_stamp: block_time,
            block_number,
            sender: PublicKey::from_bytes(&Vec::new()),
            seq_num: 0,
        });
        drop(runtime_lock);

        let genesis_pre_state_hash = Self::empty_state_hash(runtime.clone()).await?;
        let play_result = Self::play_deploys(
            runtime.clone(),
            genesis_pre_state_hash.clone(),
            terms,
            |deploy| Self::process_deploy_with_mergeable_data(runtime.clone(), deploy),
        )?;

        let (post_state_hash, processed_deploys) = play_result;
        Ok((genesis_pre_state_hash, post_state_hash, processed_deploys))
    }

    /* Deploy evaluators */

    /**
     * Evaluates deploys on root hash with checkpoint to get final state hash
     */
    pub fn play_deploys(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: StateHash,
        terms: Vec<Signed<DeployData>>,
        process_deploy: impl Fn(
            Signed<DeployData>,
        ) -> Result<(ProcessedDeploy, NumberChannelsEndVal), CasperError>,
    ) -> Result<(StateHash, Vec<(ProcessedDeploy, NumberChannelsEndVal)>), CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(Blake2b256Hash::from_bytes_prost(&start_hash));
        drop(runtime_lock);

        let mut res = Vec::new();
        for deploy in terms {
            res.push(process_deploy(deploy)?);
        }

        let mut runtime_lock = runtime.lock().unwrap();
        let final_checkpoint = runtime_lock.create_checkpoint();
        Ok((final_checkpoint.root.to_bytes_prost(), res))
    }

    /**
     * Evaluates deploy with cost accounting (PoS Pre-charge and Refund calls)
     */
    pub fn play_deploy_with_cost_accounting(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        deploy: Signed<DeployData>,
    ) -> Result<(ProcessedDeploy, NumberChannelsEndVal), CasperError> {
        let eval_collector_state = Arc::new(Mutex::new(EvalCollector::new()));

        // System deploy result of evaluation
        #[allow(type_alias_bounds)]
        type SysDeployRes<S: SystemDeployTrait> = Either<SystemDeployUserError, S::Result>;

        // Combines system deploy evaluation and update of local state with resulting event logs
        fn exec_and_save<S: SystemDeployTrait>(
            eval_collector_state: Arc<Mutex<EvalCollector>>,
            deploy_eval: impl FnOnce()
                -> Result<(Vec<Event>, SysDeployRes<S>, HashSet<Par>), CasperError>,
        ) -> Result<SysDeployRes<S>, CasperError> {
            let (event_log, result, mergeable_channels) = deploy_eval()?;
            let mut eval_collector_state_lock = eval_collector_state.lock().unwrap();
            eval_collector_state_lock.add(event_log, mergeable_channels);
            Ok(result)
        }

        let deploy_pk = deploy.pk.bytes.clone();
        let random_seed = system_deploy_util::generate_pre_charge_deploy_random_seed(&deploy);

        // Evaluates Pre-charge system deploy
        let pre_charge_result = {
            log::info!(
                "PreCharging {} for {}",
                hex::encode(&deploy_pk),
                deploy.data.total_phlo_charge()
            );
            exec_and_save::<PreChargeDeploy>(eval_collector_state.clone(), || {
                Self::play_system_deploy_internal(
                    runtime.clone(),
                    &mut PreChargeDeploy {
                        charge_amount: deploy.data.total_phlo_charge(),
                        pk: deploy.pk.clone(),
                        rand: system_deploy_util::generate_pre_charge_deploy_random_seed(&deploy),
                    },
                )
            })
        }?;

        match pre_charge_result {
            Either::Right(_) => {
                // Evaluates user deploy
                let pd = {
                    log::info!("Processing user deploy {}", hex::encode(&deploy_pk));
                    // Evaluates user deploy and append event log to local state
                    Self::process_deploy(runtime.clone(), deploy).map(|(pd, mc)| {
                        let mut eval_collector_state_lock = eval_collector_state.lock().unwrap();
                        eval_collector_state_lock.add(pd.deploy_log.clone(), mc);
                        pd
                    })
                }?;

                // Evaluates Refund system deploy
                let refund_result = {
                    log::info!(
                        "Refunding {} with {}",
                        hex::encode(&deploy_pk),
                        pd.refund_amount()
                    );
                    exec_and_save::<RefundDeploy>(eval_collector_state.clone(), || {
                        Self::play_system_deploy_internal(
                            runtime.clone(),
                            &mut RefundDeploy {
                                refund_amount: pd.refund_amount(),
                                rand: random_seed,
                            },
                        )
                    })
                }?;

                match refund_result {
                    Either::Right(_) => {
                        // Update result with accumulated event logs
                        let eval_collector_state_lock = eval_collector_state.lock().unwrap();
                        let collected = eval_collector_state_lock;

                        // Get mergeable channels data
                        let mergeable_channels_data = Self::get_number_channels_data(
                            runtime.clone(),
                            &collected.mergeable_channels,
                        )?;

                        Ok((
                            ProcessedDeploy {
                                deploy_log: collected.event_log.clone(),
                                ..pd
                            },
                            mergeable_channels_data,
                        ))
                    }

                    Either::Left(error) => {
                        // If Pre-charge succeeds and Refund fails, it's a platform error
                        log::warn!("Refund failure '{}'", error.error_message);
                        Err(CasperError::SystemRuntimeError(
                            SystemDeployPlatformFailure::GasRefundFailure(error.error_message),
                        ))
                    }
                }
            }

            Either::Left(error) => {
                // Handle evaluation errors from PreCharge
                // - assigning 0 cost - replay should reach the same state
                let mut empty_pd = ProcessedDeploy::empty(deploy);
                empty_pd.system_deploy_error = Some(error.error_message);

                // Update result with accumulated event logs
                let eval_collector_state_lock = eval_collector_state.lock().unwrap();
                let collected = eval_collector_state_lock;

                // Get mergeable channels data
                let mergeable_channels_data =
                    Self::get_number_channels_data(runtime.clone(), &collected.mergeable_channels)?;

                Ok((
                    ProcessedDeploy {
                        deploy_log: collected.event_log.clone(),
                        ..empty_pd
                    },
                    mergeable_channels_data,
                ))
            }
        }
    }

    /**
     * Evaluates deploy
     */
    pub fn process_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        deploy: Signed<DeployData>,
    ) -> Result<(ProcessedDeploy, HashSet<Par>), CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        let fallback = runtime_lock.create_soft_checkpoint();
        drop(runtime_lock);

        // Evaluate deploy
        let rt = tokio::runtime::Handle::current();
        let eval_result = rt.block_on(Self::evaluate(runtime.clone(), &deploy))?;

        let mut runtime_lock = runtime.lock().unwrap();
        let checkpoint = runtime_lock.create_soft_checkpoint();
        drop(runtime_lock);

        let eval_succeeded = eval_result.errors.is_empty();
        let deploy_sig = deploy.sig.clone();

        let deploy_result = ProcessedDeploy {
            deploy: Arc::new(deploy),
            cost: Cost::to_proto(eval_result.cost),
            deploy_log: checkpoint
                .log
                .into_iter()
                .map(|event| event_converter::to_casper_event(event))
                .collect(),
            is_failed: !eval_succeeded,
            system_deploy_error: None,
        };

        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.revert_to_soft_checkpoint(fallback);
        drop(runtime_lock);

        interpreter_util::print_deploy_errors(&deploy_sig, &eval_result.errors);

        Ok((deploy_result, eval_result.mergeable))
    }

    pub fn process_deploy_with_mergeable_data(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        deploy: Signed<DeployData>,
    ) -> Result<(ProcessedDeploy, NumberChannelsEndVal), CasperError> {
        Self::process_deploy(runtime.clone(), deploy).and_then(|(pd, merge_chs)| {
            Self::get_number_channels_data(runtime, &merge_chs).map(|data| (pd, data))
        })
    }

    pub fn get_number_channels_data(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channels: &HashSet<Par>,
    ) -> Result<NumberChannelsEndVal, CasperError> {
        Ok(channels
            .iter()
            .map(|chan| Self::get_number_channel(runtime.clone(), chan))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<HashMap<_, _>>())
    }

    pub fn get_number_channel(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channel: &Par,
    ) -> Result<Option<(Blake2b256Hash, i64)>, CasperError> {
        let runtime_lock = runtime.lock().unwrap();
        let ch_values = runtime_lock.get_data(channel);
        drop(runtime_lock);

        if ch_values.is_empty() {
            return Ok(None);
        } else {
            if ch_values.len() != 1 {
                return Err(CasperError::RuntimeError(format!(
                    "NumberChannel must have singleton value."
                )));
            }

            let num_par = &ch_values[0].a;
            let (num, _) = RholangMergingLogic::get_number_with_rnd(num_par);
            let ch_hash = stable_hash_provider::hash(channel);
            Ok(Some((ch_hash, num)))
        }
    }

    /* System deploy evaluators */

    /**
     * Evaluates System deploy with checkpoint to get final state hash
     */
    pub fn play_system_deploy<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        state_hash: StateHash,
        system_deploy: &mut S,
    ) -> Result<SystemDeployResult<S::Result>, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(Blake2b256Hash::from_bytes_prost(&state_hash));
        drop(runtime_lock);

        let (event_log, result, mergeable_channels) =
            Self::play_system_deploy_internal(runtime.clone(), system_deploy)?;

        let final_state_hash = {
            let mut runtime_lock = runtime.lock().unwrap();
            let checkpoint = runtime_lock.create_checkpoint();
            checkpoint.root.to_bytes_prost()
        };

        match result {
            Either::Right(system_deploy_result) => {
                let mcl = Self::get_number_channels_data(runtime, &mergeable_channels)?;
                if let Some(SlashDeploy {
                    invalid_block_hash,
                    pk,
                    initial_rand: _,
                }) = system_deploy.as_any().downcast_ref::<SlashDeploy>()
                {
                    Ok(SystemDeployResult::play_succeeded(
                        final_state_hash,
                        event_log,
                        SystemDeployData::create_slash(invalid_block_hash.clone(), pk.clone()),
                        mcl,
                        system_deploy_result,
                    ))
                } else if let Some(CloseBlockDeploy { .. }) =
                    system_deploy.as_any().downcast_ref::<CloseBlockDeploy>()
                {
                    Ok(SystemDeployResult::play_succeeded(
                        final_state_hash,
                        event_log,
                        SystemDeployData::create_close(),
                        mcl,
                        system_deploy_result,
                    ))
                } else {
                    Ok(SystemDeployResult::play_succeeded(
                        final_state_hash,
                        event_log,
                        SystemDeployData::Empty,
                        mcl,
                        system_deploy_result,
                    ))
                }
            }

            Either::Left(usr_err) => Ok(SystemDeployResult::play_failed(event_log, usr_err)),
        }
    }

    pub fn play_system_deploy_internal<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
    ) -> Result<
        (
            Vec<Event>,
            Either<SystemDeployUserError, S::Result>,
            HashSet<Par>,
        ),
        CasperError,
    > {
        // Get System deploy result / throw fatal errors for unexpected results
        let (result_or_system_deploy_error, eval_result) =
            Self::eval_system_deploy(runtime.clone(), system_deploy)?;

        let mut runtime_lock = runtime.lock().unwrap();
        let post_deploy_soft_checkpoint = runtime_lock.create_soft_checkpoint();
        let log = post_deploy_soft_checkpoint.log;

        Ok((
            log.into_iter()
                .map(event_converter::to_casper_event)
                .collect(),
            result_or_system_deploy_error,
            eval_result.mergeable,
        ))
    }

    /**
     * Evaluates System deploy (applicative errors are fatal)
     */
    pub fn eval_system_deploy<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
    ) -> Result<SysEvalResult<S>, CasperError> {
        let rt = tokio::runtime::Handle::current();
        let eval_result =
            rt.block_on(Self::evaluate_system_source(runtime.clone(), system_deploy))?;

        if !eval_result.errors.is_empty() {
            return Err(CasperError::SystemRuntimeError(
                SystemDeployPlatformFailure::UnexpectedSystemErrors(eval_result.errors),
            ));
        }

        let r = match Self::consume_system_result(runtime, system_deploy)? {
            Some((_, vec_list)) => match vec_list.as_slice() {
                [ListParWithRandom { pars, .. }] if pars.len() == 1 => {
                    Ok(system_deploy.extract_result(&pars[0]))
                }
                _ => Err(CasperError::SystemRuntimeError(
                    SystemDeployPlatformFailure::UnexpectedResult(
                        vec_list.iter().flat_map(|lp| lp.pars.clone()).collect(),
                    ),
                )),
            },
            None => Err(CasperError::SystemRuntimeError(
                SystemDeployPlatformFailure::ConsumeFailed,
            )),
        }?;

        Ok((r, eval_result))
    }

    /**
     * Evaluates exploratory (read-only) deploy
     */
    pub fn play_exploratory_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        term: String,
        hash: &StateHash,
    ) -> Result<Vec<Par>, CasperError> {
        let deploy_result = (|| {
            // Create a deploy with newly created private key
            let (priv_key, _) = Secp256k1.new_key_pair();

            // Creates signed deploy
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| CasperError::RuntimeError(e.to_string()))?
                .as_millis() as i64;
            let deploy = construct_deploy::source_deploy(
                term,
                timestamp,
                // Hardcoded phlogiston limit / 1 REV if phloPrice=1
                Some(100 * 1000 * 1000),
                None,
                Some(priv_key),
                None,
                None,
            )?;

            // Create return channel as first private name created in deploy term
            let mut rand = Tools::unforgeable_name_rng(&deploy.pk, deploy.data.time_stamp);
            let return_name = Par::default().with_unforgeables(vec![GUnforgeable {
                unf_instance: Some(UnfInstance::GPrivateBody(GPrivate {
                    id: rand.next().into_iter().map(|b| b as u8).collect(),
                })),
            }]);

            // Execute deploy on top of specified block hash
            Self::capture_results_with_name(runtime, hash, &deploy, &return_name)
        })();

        match deploy_result {
            Ok(result) => Ok(result),
            Err(err) => {
                println!("Error in play_exploratory_deploy: {:?}", err);
                log::error!("Error in play_exploratory_deploy: {:?}", err);
                Ok(Vec::new())
            }
        }
    }

    /* Checkpoints */

    /**
     * Creates soft checkpoint with rollback if result is false.
     */
    pub fn with_soft_transaction<A>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        fa: impl Fn() -> (A, bool),
    ) -> Result<A, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        let fallback = runtime_lock.create_soft_checkpoint();
        drop(runtime_lock);

        // Execute action
        let (a, success) = fa();

        // Revert the state if failed
        if !success {
            let mut runtime_lock = runtime.lock().unwrap();
            runtime_lock.revert_to_soft_checkpoint(fallback);
        }

        Ok(a)
    }

    /* Evaluates and captures results */

    // Return channel on which result is captured is the first name
    // in the deploy term `new return in { return!(42) }`
    pub fn capture_results(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start: &StateHash,
        deploy: &Signed<DeployData>,
    ) -> Result<Vec<Par>, CasperError> {
        // Create return channel as first unforgeable name created in deploy term
        let mut rand = Tools::unforgeable_name_rng(&deploy.pk, deploy.data.time_stamp);
        let return_name = Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GPrivateBody(GPrivate {
                id: rand.next().into_iter().map(|b| b as u8).collect(),
            })),
        }]);

        Self::capture_results_with_name(runtime, start, deploy, &return_name)
    }

    pub fn capture_results_with_name(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start: &StateHash,
        deploy: &Signed<DeployData>,
        name: &Par,
    ) -> Result<Vec<Par>, CasperError> {
        match Self::capture_results_with_errors(runtime, start, deploy, name) {
            Ok(result) => Ok(result),
            Err(err) => Err(CasperError::InterpreterError(
                InterpreterError::BugFoundError(format!(
                    "Unexpected error while capturing results from Rholang: {}",
                    err
                )),
            )),
        }
    }

    pub fn capture_results_with_errors(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start: &StateHash,
        deploy: &Signed<DeployData>,
        name: &Par,
    ) -> Result<Vec<Par>, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(Blake2b256Hash::from_bytes_prost(start));
        drop(runtime_lock);

        let rt = tokio::runtime::Handle::current();
        let eval_res = rt.block_on(Self::evaluate(runtime.clone(), deploy))?;
        if !eval_res.errors.is_empty() {
            return Err(CasperError::InterpreterError(eval_res.errors[0].clone()));
        }

        Ok(Self::get_data_par(runtime, name))
    }

    /* Evaluates Rholang source code */

    pub async fn evaluate(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        deploy: &Signed<DeployData>,
    ) -> Result<EvaluateResult, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock
            .evaluate(
                deploy.data.term.clone(),
                Cost::create(deploy.data.phlo_limit, "Evaluate deploy".to_string()),
                normalizer_env_from_deploy(deploy),
                Tools::unforgeable_name_rng(&deploy.pk, deploy.data.time_stamp),
            )
            .await?)
    }

    pub async fn evaluate_system_source<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
    ) -> Result<EvaluateResult, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock
            .evaluate(
                S::source(),
                Cost::unsafe_max(),
                system_deploy.env(),
                // TODO: Review this clone and whether to pass mut ref down into evaluate
                system_deploy.rand().clone(),
            )
            .await?)
    }

    pub fn get_data_par(runtime: Arc<Mutex<RhoRuntimeImpl>>, channel: &Par) -> Vec<Par> {
        let runtime_lock = runtime.lock().unwrap();
        let data = runtime_lock.get_data(channel);
        data.into_iter().flat_map(|datum| datum.a.pars).collect()
    }

    pub fn get_continuation_par(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channels: Vec<Par>,
    ) -> Vec<(Vec<BindPattern>, Par)> {
        let runtime_lock = runtime.lock().unwrap();
        runtime_lock
            .get_continuations(channels)
            .into_iter()
            .filter_map(|wk| {
                if let Some(TaggedCont::ParBody(par_body)) = wk.continuation.tagged_cont {
                    Some((wk.patterns, par_body.body.unwrap()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn consume_result(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channel: Par,
        pattern: BindPattern,
    ) -> Result<Option<(TaggedContinuation, Vec<ListParWithRandom>)>, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        Ok(runtime_lock.consume_result(vec![channel], vec![pattern])?)
    }

    pub fn consume_system_result<S: SystemDeployTrait>(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        system_deploy: &mut S,
    ) -> Result<Option<(TaggedContinuation, Vec<ListParWithRandom>)>, CasperError> {
        Self::consume_result(
            runtime,
            system_deploy.return_channel()?,
            system_deploy_consume_all_pattern(),
        )
    }

    /* Read only Rholang evaluator helpers */

    pub fn get_active_validators(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: &StateHash,
    ) -> Result<Vec<Validator>, CasperError> {
        let validators_pars = Self::play_exploratory_deploy(
            runtime,
            Self::activate_validator_query_source(),
            start_hash,
        )?;

        if validators_pars.len() != 1 {
            return Err(CasperError::RuntimeError(format!(
                "Incorrect number of results from query of current bonds in state {}: {}",
                PrettyPrinter::build_string_bytes(start_hash),
                validators_pars.len()
            )));
        }

        let validators = Self::to_validator_vec(validators_pars[0].to_owned())?;
        let vlds: Vec<String> = validators.iter().map(|v| hex::encode(&v)).collect();
        log::info!(
            "*** ACTIVE VALIDATORS FOR StateHash {}: {}",
            hex::encode(start_hash),
            vlds.join("\n")
        );

        Ok(validators)
    }

    pub fn compute_bonds(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        hash: &StateHash,
    ) -> Result<Vec<Bond>, CasperError> {
        let bonds_pars = Self::play_exploratory_deploy(runtime, Self::bonds_query_source(), hash)?;

        if bonds_pars.len() != 1 {
            return Err(CasperError::RuntimeError(format!(
                "Incorrect number of results from query of current bonds in state {}: {}",
                PrettyPrinter::build_string_bytes(hash),
                bonds_pars.len()
            )));
        }

        Self::to_bond_vec(bonds_pars[0].to_owned())
    }

    fn activate_validator_query_source() -> String {
        r#"
          new return, rl(`rho:registry:lookup`), poSCh in {
          rl!(`rho:rchain:pos`, *poSCh) |
          for(@(_, PoS) <- poSCh) {
            @PoS!("getActiveValidators", *return)
          }
        }
      "#
        .to_string()
    }

    fn bonds_query_source() -> String {
        r#"
        new return, rl(`rho:registry:lookup`), poSCh in {
          rl!(`rho:rchain:pos`, *poSCh) |
          for(@(_, PoS) <- poSCh) {
            @PoS!("getBonds", *return)
          }
        }
      "#
        .to_string()
    }

    fn to_validator_vec(validators_par: Par) -> Result<Vec<Validator>, CasperError> {
        if validators_par.exprs.is_empty() {
            return Ok(Vec::new());
        }

        let ps = match validators_par.exprs[0].expr_instance.as_ref().unwrap() {
            ExprInstance::ESetBody(set) => ParSetTypeMapper::eset_to_par_set(set.clone()).ps,
            _ => SortedParHashSet::create_from_empty(),
        };

        Ok(ps
            .map_iter(|v| {
                if v.exprs.len() != 1 {
                    Err(CasperError::RuntimeError(
                        "Validator in bonds map wasn't a single string.".to_string(),
                    ))
                } else {
                    match v.exprs[0].expr_instance.as_ref().unwrap() {
                        ExprInstance::GByteArray(g_byte_array) => Ok(g_byte_array.clone().into()),
                        _ => Err(CasperError::RuntimeError(
                            "Expected GByteArray in validator data".to_string(),
                        )),
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    fn to_bond_vec(bonds_map: Par) -> Result<Vec<Bond>, CasperError> {
        if bonds_map.exprs.is_empty() {
            return Ok(Vec::new());
        }

        let ps = match bonds_map.exprs[0].expr_instance.as_ref().unwrap() {
            ExprInstance::EMapBody(map) => ParMapTypeMapper::emap_to_par_map(map.clone()).ps,
            _ => SortedParMap::create_from_empty(),
        };

        Ok(ps
            .map_iter(|(validator, bond)| {
                if validator.exprs.len() != 1 {
                    Err(CasperError::RuntimeError(
                        "Validator in bonds map wasn't a single string.".to_string(),
                    ))
                } else if bond.exprs.len() != 1 {
                    Err(CasperError::RuntimeError(
                        "Stake in bonds map wasn't a single string.".to_string(),
                    ))
                } else {
                    let validator_name = match validator.exprs[0].expr_instance.as_ref().unwrap() {
                        ExprInstance::GByteArray(g_byte_array) => Ok(g_byte_array.clone().into()),
                        _ => Err(CasperError::RuntimeError(
                            "Expected GByteArray in validator data".to_string(),
                        )),
                    }?;

                    let stake_amount = match bond.exprs[0].expr_instance.as_ref().unwrap() {
                        ExprInstance::GInt(g_int) => Ok(*g_int),
                        _ => Err(CasperError::RuntimeError(
                            "Expected GInt in stake data".to_string(),
                        )),
                    }?;

                    Ok(Bond {
                        validator: validator_name,
                        stake: stake_amount,
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?)
    }
}
