// See casper/src/main/scala/coop/rchain/casper/util/rholang/InterpreterUtil.scala

use dashmap::DashMap;
use prost::bytes::Bytes;
use rspace_plus_plus::rspace::history::Either;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use block_storage::rust::{
    dag::block_dag_key_value_storage::KeyValueDagRepresentation,
    key_value_block_store::KeyValueBlockStore,
};
use crypto::rust::signatures::signed::Signed;
use models::{
    rhoapi::Par,
    rust::{
        block::state_hash::StateHash,
        block_hash::BlockHash,
        casper::{
            pretty_printer::PrettyPrinter,
            protocol::casper_message::{
                BlockMessage, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
            },
        },
        validator::Validator,
    },
};
use rholang::rust::interpreter::{
    compiler::compiler::Compiler, errors::InterpreterError, system_processes::BlockData,
};

use crate::rust::{
    block_status::BlockStatus,
    casper::CasperSnapshot,
    errors::CasperError,
    util::{proto_util, rholang::system_deploy::SystemDeployTrait},
    BlockProcessing,
};

use super::{replay_failure::ReplayFailure, runtime_manager::RuntimeManager};

pub fn mk_term(rho: &str, normalizer_env: HashMap<String, Par>) -> Result<Par, InterpreterError> {
    Compiler::source_to_adt_with_normalizer_env(rho, normalizer_env)
}

// Returns (None, checkpoints) if the block's tuplespace hash
// does not match the computed hash based on the deploys
pub fn validate_block_checkpoint(
    block: &BlockMessage,
    block_store: &mut KeyValueBlockStore,
    s: &CasperSnapshot,
    runtime_manager: &RuntimeManager,
) -> Result<BlockProcessing<Option<StateHash>>, CasperError> {
    let incoming_pre_state_hash = proto_util::pre_state_hash(block);
    let parents = proto_util::get_parents(block_store, block);
    let computed_parents_info =
        compute_parents_post_state(block_store, parents, s, runtime_manager);

    log::info!(
        "Computed parents post state for {}.",
        PrettyPrinter::build_string_block_message(&block, false)
    );

    match computed_parents_info {
        Ok((computed_pre_state_hash, rejected_deploys)) => {
            let rejected_deploy_ids: HashSet<_> = rejected_deploys.iter().cloned().collect();
            let block_rejected_deploy_sigs: HashSet<_> = block
                .body
                .rejected_deploys
                .iter()
                .map(|d| d.sig.clone())
                .collect();

            if incoming_pre_state_hash != computed_pre_state_hash {
                // TODO: at this point we may just as well terminate the replay, there's no way it will succeed.
                log::warn!(
                    "Computed pre-state hash {} does not equal block's pre-state hash {}.",
                    PrettyPrinter::build_string_bytes(&computed_pre_state_hash),
                    PrettyPrinter::build_string_bytes(&incoming_pre_state_hash)
                );

                return Ok(Either::Right(None));
            } else if rejected_deploy_ids != block_rejected_deploy_sigs {
                log::warn!(
                    "Computed rejected deploys {} does not equal block's rejected deploys {}.",
                    rejected_deploy_ids
                        .iter()
                        .map(|bytes| PrettyPrinter::build_string_bytes(bytes))
                        .collect::<Vec<_>>()
                        .join(","),
                    block
                        .body
                        .rejected_deploys
                        .iter()
                        .map(|d| PrettyPrinter::build_string_bytes(&d.sig))
                        .collect::<Vec<_>>()
                        .join(",")
                );

                return Ok(Either::Left(BlockStatus::invalid_rejected_deploy()));
            } else {
                let replay_result = replay_block(
                    incoming_pre_state_hash,
                    block,
                    block_store,
                    &s.dag,
                    runtime_manager,
                )?;

                handle_errors(proto_util::post_state_hash(block), replay_result)
            }
        }
        Err(ex) => {
            return Ok(Either::Left(BlockStatus::exception(ex)));
        }
    }
}

fn replay_block(
    initial_state_hash: StateHash,
    block: &BlockMessage,
    block_store: &mut KeyValueBlockStore,
    dag: &KeyValueDagRepresentation,
    runtime_manager: &RuntimeManager,
) -> Result<Either<ReplayFailure, StateHash>, CasperError> {
    todo!()
}

fn handle_errors(
    ts_hash: StateHash,
    result: Either<ReplayFailure, StateHash>,
) -> Result<BlockProcessing<Option<StateHash>>, CasperError> {
    todo!()
}

pub fn print_deploy_errors(deploy_sig: &Bytes, errors: &[InterpreterError]) {
    let deploy_info = PrettyPrinter::build_string_sig(&deploy_sig);
    let error_messages: String = errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    println!("Deploy ({}) errors: {}", deploy_info, error_messages);

    log::warn!("Deploy ({}) errors: {}", deploy_info, error_messages);
}

pub fn compute_deploys_checkpoint(
    block_store: &mut KeyValueBlockStore,
    parents: Vec<BlockMessage>,
    deploys: Vec<Arc<Signed<DeployData>>>,
    system_deploys: Vec<impl SystemDeployTrait>,
    s: &CasperSnapshot,
    runtime_manager: &RuntimeManager,
    block_data: BlockData,
    invalid_blocks: DashMap<BlockHash, Validator>,
) -> Result<
    (
        StateHash,
        StateHash,
        Vec<ProcessedDeploy>,
        Vec<prost::bytes::Bytes>,
        Vec<ProcessedSystemDeploy>,
    ),
    CasperError,
> {
    todo!()
}

fn compute_parents_post_state(
    block_store: &mut KeyValueBlockStore,
    parents: Vec<BlockMessage>,
    s: &CasperSnapshot,
    runtime_manager: &RuntimeManager,
) -> Result<(StateHash, Vec<Bytes>), CasperError> {
    todo!()
}
