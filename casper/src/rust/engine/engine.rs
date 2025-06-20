// See casper/src/main/scala/coop/rchain/casper/engine/Engine.scala

use block_storage::rust::dag::block_dag_key_value_storage::BlockDagKeyValueStorage;
use block_storage::rust::key_value_block_store::KeyValueBlockStore;
use comm::rust::peer_node::PeerNode;
use comm::rust::rp::rp_conf::RPConf;
use comm::rust::transport::transport_layer::{Blob, TransportLayer};
use models::rust::casper::protocol::casper_message::{
    ApprovedBlock, BlockMessage, CasperMessage, NoApprovedBlockAvailable,
};
use models::rust::casper::protocol::packet_type_tag::ToPacket;

use crate::rust::errors::CasperError;
use crate::rust::Casper;

pub trait Engine {
    fn init(&self) -> Result<(), CasperError>;

    fn handle(&self, peer: PeerNode, msg: CasperMessage) -> Result<(), CasperError>;

    fn with_casper<A>(
        &self,
        f: impl FnOnce(&mut dyn Casper) -> Result<A, CasperError>,
        default: impl FnOnce() -> Result<A, CasperError>,
    ) -> Result<A, CasperError>;
}

pub fn noop() -> Result<impl Engine, CasperError> {
    struct NoopEngine;

    impl Engine for NoopEngine {
        fn init(&self) -> Result<(), CasperError> {
            Ok(())
        }

        fn handle(&self, _peer: PeerNode, _msg: CasperMessage) -> Result<(), CasperError> {
            Ok(())
        }

        fn with_casper<A>(
            &self,
            _f: impl FnOnce(&mut dyn Casper) -> Result<A, CasperError>,
            default: impl FnOnce() -> Result<A, CasperError>,
        ) -> Result<A, CasperError> {
            default()
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
