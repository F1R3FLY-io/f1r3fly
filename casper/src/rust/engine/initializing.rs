// See casper/src/main/scala/coop/rchain/casper/engine/Initializing.scala

use async_trait::async_trait;
use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};

use block_storage::rust::{
    dag::block_dag_key_value_storage::BlockDagKeyValueStorage,
    key_value_block_store::KeyValueBlockStore,
};
use comm::rust::{
    rp::{connect::ConnectionsCell, rp_conf::RPConf},
    transport::transport_layer::TransportLayer,
};
use models::rust::{
    block_hash::BlockHash,
    casper::protocol::casper_message::{ApprovedBlock, BlockMessage, StoreItemsMessage},
};
use rspace_plus_plus::rspace::{history::Either, state::rspace_state_manager::RSpaceStateManager};

use crate::rust::{
    block_status::ValidBlock,
    casper::CasperShardConf,
    engine::lfs_block_requester::{self, BlockRequesterOps},
    errors::CasperError,
    util::proto_util,
    validate::Validate,
    validator_identity::ValidatorIdentity,
};

pub struct Initializing<T: TransportLayer> {
    transport_layer: T,
    rp_conf_ask: RPConf,
    connections_cell: ConnectionsCell,
    last_approved_block: Arc<Mutex<Option<ApprovedBlock>>>,
    block_store: KeyValueBlockStore,
    block_dag_storage: BlockDagKeyValueStorage,
    rspace_state_manager: RSpaceStateManager,

    block_processing_queue: VecDeque<BlockMessage>,
    blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
    casper_shard_conf: CasperShardConf,
    validator_id: Option<ValidatorIdentity>,
    the_init: Box<dyn FnOnce() -> () + Send + Sync>,
    block_message_queue: VecDeque<BlockMessage>,
    tuple_space_queue: VecDeque<StoreItemsMessage>,
    trim_state: Option<bool>,
    disable_state_exporter: bool,
}

impl<T: TransportLayer + Sync> Initializing<T> {
    pub async fn request_approved_state(
        &mut self,
        approved_block: ApprovedBlock,
    ) -> Result<(), CasperError> {
        // Starting minimum block height. When latest blocks are downloaded new minimum will be calculated.
        let block = &approved_block.candidate.block;
        let start_block_number = proto_util::block_number(block);
        let min_block_number_for_deploy_lifespan = std::cmp::max(
            0,
            start_block_number - self.casper_shard_conf.deploy_lifespan,
        );

        // Create channel for incoming block messages (equivalent to Scala's blockMessageQueue)
        let (response_message_tx, response_message_rx) = tokio::sync::mpsc::unbounded_channel();

        let block_request_stream = lfs_block_requester::stream(
            approved_block,
            // TODO: just use self.block_message_queue instead of clone?
            self.block_message_queue.clone(),
            response_message_rx,
            min_block_number_for_deploy_lifespan,
            Duration::from_secs(30),
            self,
        )
        .await?;

        todo!()
    }
}

// Implement BlockRequesterOps trait for Initializing
#[async_trait]
impl<T: TransportLayer + Sync> BlockRequesterOps for Initializing<T> {
    async fn request_for_block(&self, block_hash: &BlockHash) -> Result<(), CasperError> {
        self.transport_layer
            .broadcast_request_for_block(&self.connections_cell, &self.rp_conf_ask, block_hash)
            .await?;
        Ok(())
    }

    fn contains_block(&self, block_hash: &BlockHash) -> Result<bool, CasperError> {
        Ok(self.block_store.contains(block_hash)?)
    }

    fn get_block_from_store(&self, block_hash: &BlockHash) -> BlockMessage {
        self.block_store.get_unsafe(block_hash)
    }

    fn put_block_to_store(
        &mut self,
        block_hash: BlockHash,
        block: &BlockMessage,
    ) -> Result<(), CasperError> {
        Ok(self.block_store.put(block_hash, &block)?)
    }

    fn validate_block(&self, block: &BlockMessage) -> bool {
        let block_number = proto_util::block_number(block);
        if block_number == 0 {
            // TODO: validate genesis (zero) block correctly - OLD
            true
        } else {
            match Validate::block_hash(block) {
                Either::Right(ValidBlock::Valid) => true,
                _ => false,
            }
        }
    }
}
