// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeManagerTest.scala

use std::{future::Future, sync::Once};

use casper::rust::{
    errors::CasperError,
    util::{
        construct_deploy,
        rholang::{
            costacc::check_balance::CheckBalance, runtime_manager::RuntimeManager,
            system_deploy_user_error::SystemDeployUserError,
        },
    },
};
use crypto::rust::signatures::signed::Signed;
use models::rust::{
    block::state_hash::StateHash,
    casper::protocol::casper_message::{BlockMessage, DeployData, ProcessedDeploy},
};
use rholang::rust::interpreter::system_processes::BlockData;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;

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

    // println!(
    //     "genesis_block_hash: {:?}",
    //     Blake2b256Hash::from_bytes_prost(&genesis_block.body.state.post_state_hash)
    // );

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
