// See casper/src/main/scala/coop/rchain/casper/util/rholang/InterpreterUtil.scala

use dashmap::DashMap;
use prost::bytes::Bytes;

use block_storage::rust::key_value_block_store::KeyValueBlockStore;
use crypto::rust::signatures::signed::Signed;
use models::rust::{
    block::state_hash::StateHash,
    block_hash::BlockHash,
    casper::{
        pretty_printer::PrettyPrinter,
        protocol::casper_message::{
            BlockMessage, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
        },
    },
    validator::Validator,
};
use rholang::rust::interpreter::{errors::InterpreterError, system_processes::BlockData};

use crate::rust::{
    casper::CasperSnapshot, errors::CasperError, util::rholang::system_deploy::SystemDeployTrait,
};

use super::runtime_manager::RuntimeManager;

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
    deploys: Vec<Signed<DeployData>>,
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
