// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeManagerTest.scala

use std::{
    future::Future,
    sync::{Arc, Mutex, Once},
};

use casper::rust::{
    errors::CasperError,
    rholang::{replay_runtime::ReplayRuntimeOps, runtime::RuntimeOps},
    util::{
        construct_deploy,
        rholang::{
            costacc::{
                check_balance::CheckBalance, close_block_deploy::CloseBlockDeploy,
                pre_charge_deploy::PreChargeDeploy, refund_deploy::RefundDeploy,
            },
            runtime_manager::RuntimeManager,
            system_deploy::SystemDeployTrait,
            system_deploy_result::SystemDeployResult,
            system_deploy_user_error::SystemDeployUserError,
        },
    },
};
use crypto::rust::{hash::blake2b512_random::Blake2b512Random, signatures::signed::Signed};
use models::rust::{
    block::state_hash::StateHash,
    casper::protocol::casper_message::{
        BlockMessage, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
    },
};
use rholang::rust::interpreter::{
    rho_runtime::{RhoRuntime, RhoRuntimeImpl},
    system_processes::BlockData,
};
use rspace_plus_plus::rspace::{hashing::blake2b256_hash::Blake2b256Hash, history::Either};

use crate::util::genesis_builder::{GenesisContext, GenessisBuilder};

use super::resources::{copy_storage, mk_runtime_manager_at, mk_test_rnode_store_manager};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true) // ensures logs show up in test output
            .filter_level(log::LevelFilter::Info)
            .try_init()
            .unwrap();
    });
}

enum SystemDeployReplayResult<A> {
    ReplaySucceeded {
        state_hash: StateHash,
        result: A,
    },
    ReplayFailed {
        system_deploy_error: SystemDeployUserError,
    },
}

impl<A> SystemDeployReplayResult<A> {
    fn replay_succeeded(state_hash: StateHash, result: A) -> Self {
        Self::ReplaySucceeded { state_hash, result }
    }

    fn replay_failed(system_deploy_error: SystemDeployUserError) -> Self {
        Self::ReplayFailed {
            system_deploy_error,
        }
    }
}

async fn genesis_context() -> Result<GenesisContext, CasperError> {
    let mut genesis_builder = GenessisBuilder::new();
    let genesis_context = genesis_builder.build_genesis_with_parameters(None).await?;
    Ok(genesis_context)
}

async fn genesis() -> Result<BlockMessage, CasperError> {
    let genesis_context = genesis_context().await?;
    Ok(genesis_context.genesis_block)
}

async fn with_runtime_manager<F, Fut, R>(f: F) -> Result<R, CasperError>
where
    F: FnOnce(RuntimeManager, GenesisContext, BlockMessage) -> Fut,
    Fut: Future<Output = R>,
{
    init_logger();
    let genesis_context = genesis_context().await?;
    let genesis_block = genesis_context.genesis_block.clone();

    let storage_dir = copy_storage(genesis_context.storage_directory.clone());
    let kvm = mk_test_rnode_store_manager(storage_dir);
    let runtime_manager = mk_runtime_manager_at(kvm, None).await;

    Ok(f(runtime_manager, genesis_context, genesis_block).await)
}

async fn compute_state(
    runtime_manager: &mut RuntimeManager,
    genesis_context: &GenesisContext,
    deploy: Signed<DeployData>,
    state_hash: &StateHash,
) -> (StateHash, ProcessedDeploy) {
    let time_stamp = deploy.data.time_stamp;
    let (new_state_hash, processed_deploys, _extra) = runtime_manager
        .compute_state(
            state_hash,
            vec![deploy],
            Vec::<CheckBalance>::new(),
            BlockData {
                time_stamp,
                block_number: 0,
                sender: genesis_context.validator_pks()[0].clone(),
                seq_num: 0,
            },
            None,
        )
        .await
        .unwrap();

    let result = processed_deploys.into_iter().next().unwrap();
    (new_state_hash, result)
}

async fn replay_compute_state(
    runtime_manager: &mut RuntimeManager,
    genesis_context: &GenesisContext,
    processed_deploy: ProcessedDeploy,
    state_hash: &StateHash,
) -> Result<StateHash, CasperError> {
    let time_stamp = processed_deploy.deploy.data.time_stamp;
    runtime_manager
        .replay_compute_state(
            state_hash,
            vec![processed_deploy],
            Vec::new(),
            &BlockData {
                time_stamp,
                block_number: 0,
                sender: genesis_context.validator_pks()[0].clone(),
                seq_num: 0,
            },
            None,
            false,
        )
        .await
}

#[tokio::test]
async fn comput_state_should_charge_for_deploys() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let gen_post_state = genesis_block.body.state.post_state_hash;
            let source = r#"
            new rl(`rho:registry:lookup`), listOpsCh in {
                rl!(`rho:lang:listOps`, *listOpsCh) |
                for(x <- listOpsCh){
                    Nil
                }
            }
            "#;

            // TODO: Prohibit negative gas prices and gas limits in deploys. - OLD
            // TODO: Make minimum maximum yield for deploy parameter of node. - OLD
            let deploy = construct_deploy::source_deploy_now_full(
                source.to_string(),
                Some(100000),
                None,
                None,
                None,
                None,
            )
            .unwrap();

            let (new_state_hash, processed_deploy) = compute_state(
                &mut runtime_manager,
                &genesis_context,
                deploy,
                &gen_post_state,
            )
            .await;

            let replay_state_hash = replay_compute_state(
                &mut runtime_manager,
                &genesis_context,
                processed_deploy,
                &gen_post_state,
            )
            .await
            .unwrap();

            assert!(new_state_hash != gen_post_state && replay_state_hash == new_state_hash);
        },
    )
    .await
    .unwrap()
}

async fn compare_successful_system_deploys<S: SystemDeployTrait, F>(
    runtime_manager: &mut RuntimeManager,
    genesis_context: &GenesisContext,
    start_state: &StateHash,
    play_system_deploy: &mut S,
    replay_system_deploy: &mut S,
    result_assertion: F,
) -> Result<StateHash, CasperError>
where
    F: Fn(&S::Result) -> bool,
    <S as SystemDeployTrait>::Result: PartialEq,
{
    let runtime = runtime_manager.spawn_runtime().await;
    {
        let runtime_lock = runtime.lock().unwrap();
        runtime_lock.set_block_data(BlockData {
            time_stamp: 0,
            block_number: 0,
            sender: genesis_context.validator_pks()[0].clone(),
            seq_num: 0,
        });
    }

    let play_system_result =
        RuntimeOps::play_system_deploy(runtime, start_state, play_system_deploy).await?;

    match play_system_result {
        SystemDeployResult::PlaySucceeded {
            state_hash: final_play_state_hash,
            processed_system_deploy,
            mergeable_channels: _,
            result: play_result,
        } => {
            result_assertion(&play_result);

            let replay_runtime = runtime_manager.spawn_replay_runtime().await;
            {
                let replay_runtime_lock = replay_runtime.lock().unwrap();
                replay_runtime_lock.set_block_data(BlockData {
                    time_stamp: 0,
                    block_number: 0,
                    sender: genesis_context.validator_pks()[0].clone(),
                    seq_num: 0,
                });
            }

            let replay_system_result = exec_replay_system_deploy(
                replay_runtime,
                start_state,
                replay_system_deploy,
                &processed_system_deploy,
            )
            .await?;

            match replay_system_result {
                SystemDeployReplayResult::ReplaySucceeded {
                    state_hash: final_replay_state_hash,
                    result: replay_result,
                } => {
                    assert!(final_play_state_hash == final_replay_state_hash);
                    assert!(play_result == replay_result);
                    Ok(final_replay_state_hash)
                }

                SystemDeployReplayResult::ReplayFailed {
                    system_deploy_error,
                } => panic!(
                    "Unexpected user error during replay: {:?}",
                    system_deploy_error
                ),
            }
        }

        SystemDeployResult::PlayFailed {
            processed_system_deploy,
        } => panic!(
            "Unexpected system error during play: {:?}",
            processed_system_deploy
        ),
    }
}

async fn exec_replay_system_deploy<S: SystemDeployTrait>(
    runtime: Arc<Mutex<RhoRuntimeImpl>>,
    state_hash: &StateHash,
    system_deploy: &mut S,
    processed_system_deploy: &ProcessedSystemDeploy,
) -> Result<SystemDeployReplayResult<S::Result>, CasperError> {
    let expected_failure = match processed_system_deploy {
        ProcessedSystemDeploy::Failed { error_msg, .. } => Some(error_msg.clone()),
        _ => None,
    };

    let rig_result = ReplayRuntimeOps::rig_with_check_system_deploy(
        runtime.clone(),
        processed_system_deploy.clone(),
        || async {
            {
                let mut runtime_lock = runtime.lock().unwrap();
                runtime_lock.reset(Blake2b256Hash::from_bytes_prost(state_hash));
            }

            ReplayRuntimeOps::replay_system_deploy_internal(
                runtime.clone(),
                system_deploy,
                &expected_failure,
            )
            .await
        },
    )
    .await?;

    match rig_result {
        (Either::Right(result), _) => {
            let mut runtime_lock = runtime.lock().unwrap();
            let checkpoint = runtime_lock.create_checkpoint();

            Ok(SystemDeployReplayResult::ReplaySucceeded {
                state_hash: checkpoint.root.to_bytes_prost(),
                result,
            })
        }

        (Either::Left(error), _) => Ok(SystemDeployReplayResult::ReplayFailed {
            system_deploy_error: error,
        }),
    }
}

#[tokio::test]
async fn pre_charge_deploy_should_reduce_user_account_balance_by_correct_amount() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let user_pk = construct_deploy::DEFAULT_PUB.clone();
            let state_hash_0 = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &genesis_block.body.state.post_state_hash,
                &mut PreChargeDeploy {
                    charge_amount: 9000000,
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![0]),
                },
                &mut PreChargeDeploy {
                    charge_amount: 9000000,
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![0]),
                },
                |_| true,
            )
            .await
            .unwrap();

            let state_hash_1 = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &state_hash_0,
                &mut CheckBalance {
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![1]),
                },
                &mut CheckBalance {
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![1]),
                },
                |result| *result == 0,
            )
            .await
            .unwrap();

            let state_hash_2 = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &state_hash_1,
                &mut RefundDeploy {
                    refund_amount: 9000000,
                    rand: Blake2b512Random::create_from_bytes(&vec![2]),
                },
                &mut RefundDeploy {
                    refund_amount: 9000000,
                    rand: Blake2b512Random::create_from_bytes(&vec![2]),
                },
                |_| true,
            )
            .await
            .unwrap();

            let _ = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &state_hash_2,
                &mut CheckBalance {
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![3]),
                },
                &mut CheckBalance {
                    pk: user_pk,
                    rand: Blake2b512Random::create_from_bytes(&vec![3]),
                },
                |result| *result == 9000000,
            )
            .await
            .unwrap();
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn close_block_should_make_epoch_change_and_reward_validator() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let _ = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &genesis_block.body.state.post_state_hash,
                &mut CloseBlockDeploy {
                    initial_rand: Blake2b512Random::create_from_bytes(&vec![0]),
                },
                &mut CloseBlockDeploy {
                    initial_rand: Blake2b512Random::create_from_bytes(&vec![0]),
                },
                |_| true,
            )
            .await
            .unwrap();
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn close_block_replay_should_fail_with_different_random_seed() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let res = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &genesis_block.body.state.post_state_hash,
                &mut CloseBlockDeploy {
                    initial_rand: Blake2b512Random::create_from_bytes(&vec![0]),
                },
                &mut CloseBlockDeploy {
                    initial_rand: Blake2b512Random::create_from_bytes(&vec![1]),
                },
                |_| true,
            )
            .await;

            assert!(res.is_err());
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn balance_deploy_should_compute_rev_balances() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let user_pk = construct_deploy::DEFAULT_PUB.clone();
            let _ = compare_successful_system_deploys(
                &mut runtime_manager,
                &genesis_context,
                &genesis_block.body.state.post_state_hash,
                &mut CheckBalance {
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![]),
                },
                &mut CheckBalance {
                    pk: user_pk.clone(),
                    rand: Blake2b512Random::create_from_bytes(&vec![]),
                },
                |result| *result == 9000000,
            )
            .await
            .unwrap();
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn compute_state_should_capture_rholang_errors() {
    with_runtime_manager(
        |mut runtime_manager, genesis_context, genesis_block| async move {
            let bad_rholang =
                r#" for(@x <- @"x" & @y <- @"y"){ @"xy"!(x + y) } | @"x"!(1) | @"y"!("hi") "#;
            let deploy = construct_deploy::source_deploy_now_full(
                bad_rholang.to_string(),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

            let result = compute_state(
                &mut runtime_manager,
                &genesis_context,
                deploy,
                &genesis_block.body.state.post_state_hash,
            )
            .await;

            assert!(result.1.is_failed == true);
        },
    )
    .await
    .unwrap();
}
