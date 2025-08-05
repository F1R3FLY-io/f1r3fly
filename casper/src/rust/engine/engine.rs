// See casper/src/main/scala/coop/rchain/casper/engine/Engine.scala

use async_trait::async_trait;
use block_storage::rust::dag::block_dag_key_value_storage::BlockDagKeyValueStorage;
use block_storage::rust::key_value_block_store::KeyValueBlockStore;
use comm::rust::peer_node::PeerNode;
use comm::rust::rp::connect::ConnectionsCell;
use comm::rust::rp::rp_conf::RPConf;
use comm::rust::transport::transport_layer::{Blob, TransportLayer};
use models::rust::block_hash::BlockHash;
use models::rust::casper::pretty_printer::PrettyPrinter;
use models::rust::casper::protocol::casper_message::{
    ApprovedBlock, BlockMessage, CasperMessage, NoApprovedBlockAvailable,
};
use models::rust::casper::protocol::packet_type_tag::ToPacket;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::rust::casper::MultiParentCasper;
use crate::rust::engine::block_retriever::BlockRetriever;
use crate::rust::engine::engine_cell::EngineCell;
use crate::rust::engine::running::Running;
use crate::rust::errors::CasperError;
use crate::rust::validator_identity::ValidatorIdentity;

/// Object-safe Engine trait that matches Scala Engine[F] behavior
/// Note: with_casper method is not included here due to object-safety constraints
/// Implementations should provide their own with_casper methods when needed
#[async_trait(?Send)]
pub trait Engine: Send + Sync {
    async fn init(&self) -> Result<(), CasperError>;

    async fn handle(&mut self, peer: PeerNode, msg: CasperMessage) -> Result<(), CasperError>;

    /// Clone the engine into a boxed trait object
    fn clone_box(&self) -> Box<dyn Engine>;
}

/// Trait for engines that provide withCasper functionality
/// This matches the Scala Engine[F] withCasper method behavior
#[async_trait(?Send)]
pub trait WithCasper<M: MultiParentCasper>: Send + Sync {
    async fn with_casper<A, F, Fut>(
        &mut self,
        f: F,
        default: Result<A, CasperError>,
    ) -> Result<A, CasperError>
    where
        F: FnOnce(&mut M) -> Fut + Send,
        Fut: std::future::Future<Output = Result<A, CasperError>> + Send,
        A: Send;
}

pub fn noop() -> Result<impl Engine, CasperError> {
    #[derive(Clone)]
    struct NoopEngine;

    #[async_trait(?Send)]
    impl Engine for NoopEngine {
        async fn init(&self) -> Result<(), CasperError> {
            Ok(())
        }

        async fn handle(
            &mut self,
            _peer: PeerNode,
            _msg: CasperMessage,
        ) -> Result<(), CasperError> {
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn Engine> {
            Box::new(self.clone())
        }
    }

    Ok(NoopEngine)
}

pub fn log_no_approved_block_available(identifier: &str) {
    log::info!(
        "No approved block available on node {}. Will request again in 10 seconds.",
        identifier
    )
}

/*
 * Note the ordering of the insertions is important.
 * We always want the block dag store to be a subset of the block store.
 */
pub fn insert_into_block_and_dag_store(
    block_store: &mut KeyValueBlockStore,
    block_dag_storage: &mut BlockDagKeyValueStorage,
    genesis: &BlockMessage,
    approved_block: ApprovedBlock,
) -> Result<(), CasperError> {
    block_store.put(genesis.block_hash.clone(), genesis)?;
    block_dag_storage.insert(genesis, false, true)?;
    block_store.put_approved_block(approved_block)?;
    Ok(())
}

pub async fn send_no_approved_block_available(
    rp_conf_ask: &RPConf,
    transport_layer: &impl TransportLayer,
    identifier: &str,
    peer: PeerNode,
) -> Result<(), CasperError> {
    let local = rp_conf_ask.local.clone();
    // TODO: remove NoApprovedBlockAvailable.nodeIdentifier, use `sender` provided by TransportLayer
    let no_approved_block_available = NoApprovedBlockAvailable {
        node_identifier: local.to_string(),
        identifier: identifier.to_string(),
    }
    .to_proto();

    let msg = Blob {
        sender: local,
        packet: no_approved_block_available.mk_packet(),
    };

    transport_layer.stream(&peer, &msg).await?;
    Ok(())
}

pub async fn transition_to_running<
    T: MultiParentCasper + Send + Sync + Clone + 'static,
    U: TransportLayer + Send + Sync + 'static,
>(
    block_processing_queue: Arc<Mutex<VecDeque<(T, BlockMessage)>>>,
    blocks_in_processing: Arc<Mutex<HashSet<BlockHash>>>,
    casper: T,
    approved_block: ApprovedBlock,
    validator_id: Option<ValidatorIdentity>,
    _init: Box<dyn FnOnce() -> Result<(), CasperError> + Send + Sync>,
    disable_state_exporter: bool,
    connections_cell: ConnectionsCell,
    transport: Arc<U>,
    conf: RPConf,
    block_retriever: Arc<BlockRetriever<U>>,
    engine_cell: &EngineCell,
) -> Result<(), CasperError> {
    let approved_block_info =
        PrettyPrinter::build_string_block_message(&approved_block.candidate.block, true);

    log::info!(
        "Making a transition to Running state. Approved {}",
        approved_block_info
    );

    // TODO: Publish EnteredRunningState event when event system is available
    // let block_hash_string = PrettyPrinter::build_string_no_limit(&approved_block.candidate.block.block_hash);
    // event_log.publish(Event::EnteredRunningState { block_hash: block_hash_string })?;

    let the_init = Arc::new(|| Ok(()));
    let running = Running::new(
        block_processing_queue,
        blocks_in_processing,
        casper,
        approved_block,
        validator_id,
        the_init,
        disable_state_exporter,
        connections_cell,
        transport,
        conf,
        block_retriever,
    );

    engine_cell.set(Arc::new(running)).await?;

    Ok(())
}
