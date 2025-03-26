// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeReplaySyntax.scala

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use models::rust::{
    block::state_hash::StateHash,
    block_hash::BlockHash,
    casper::protocol::casper_message::{ProcessedDeploy, ProcessedSystemDeploy},
    validator::Validator,
};
use rholang::rust::interpreter::{rho_runtime::RhoRuntimeImpl, system_processes::BlockData};
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash, merger::merging_logic::NumberChannelsEndVal,
};

use crate::rust::util::rholang::replay_failure::ReplayFailure;

pub struct ReplayRuntimeOps;

impl ReplayRuntimeOps {
    pub fn replay_compute_state(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        block_data: BlockData,
        invalid_blocks: Option<HashMap<BlockHash, Validator>>,
        is_genesis: bool, //FIXME have a better way of knowing this. Pass the replayDeploy function maybe? - OLD
    ) -> Result<(Blake2b256Hash, Vec<NumberChannelsEndVal>), ReplayFailure> {
        todo!()
    }
}
