// See casper/src/main/scala/coop/rchain/casper/engine/Running.scala

use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use comm::rust::peer_node::PeerNode;
use models::rust::{
    block_hash::BlockHash,
    casper::protocol::casper_message::{ApprovedBlock, BlockMessage, CasperMessage},
};

use crate::rust::{
    casper::MultiParentCasper, engine::engine::Engine, errors::CasperError,
    validator_identity::ValidatorIdentity,
};

pub struct Running<T: MultiParentCasper> {
    block_processing_queue: VecDeque<(T, BlockMessage)>,
    blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
    casper: T,
    approved_block: ApprovedBlock,
    validator_id: Option<ValidatorIdentity>,
    the_init: Box<dyn FnOnce() -> Result<(), CasperError> + Send + Sync>,
    disable_state_exporter: bool,
}

impl<T: MultiParentCasper> Running<T> {
    pub fn new(
        block_processing_queue: VecDeque<(T, BlockMessage)>,
        blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
        casper: T,
        approved_block: ApprovedBlock,
        validator_id: Option<ValidatorIdentity>,
        init: Box<dyn FnOnce() -> Result<(), CasperError> + Send + Sync>,
        disable_state_exporter: bool,
    ) -> Self {
        Running {
            block_processing_queue,
            blocks_in_processing,
            casper,
            approved_block,
            validator_id,
            the_init: init,
            disable_state_exporter,
        }
    }
}

impl<T: MultiParentCasper + Send + Sync> Engine for Running<T> {
    fn init(&self) -> Result<(), CasperError> {
        // Note: In Rust we can't call FnOnce multiple times, so this is a simplified implementation
        // The actual init should be called externally when constructing Running
        Ok(())
    }

    fn handle(&self, _peer: PeerNode, _msg: CasperMessage) -> Result<(), CasperError> {
        // TODO: Implement message handling logic from Scala Running class
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        // Note: This is simplified - full implementation would need proper cloning
        panic!("Running engine cannot be cloned - not implemented")
    }
}
