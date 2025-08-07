// See casper/src/test/scala/coop/rchain/casper/engine/Setup.scala

use block_storage::rust::{
    dag::block_dag_key_value_storage::KeyValueDagRepresentation,
    key_value_block_store::KeyValueBlockStore,
};
use casper::rust::{
    engine::{block_retriever, running::Running},
    validator_identity::ValidatorIdentity,
};
use comm::rust::{
    peer_node::{Endpoint, NodeIdentifier, PeerNode},
    rp::connect::{Connections, ConnectionsCell},
    test_instances::{create_rp_conf_ask, TransportLayerStub},
};
use crypto::rust::{private_key::PrivateKey, public_key::PublicKey};
use models::{
    routing::Protocol,
    rust::{
        block_hash::BlockHash,
        casper::protocol::casper_message::{
            ApprovedBlock, ApprovedBlockCandidate, BlockMessage, CasperMessage, HasBlock,
        },
    },
};
use prost::bytes::Bytes;
use shared::rust::store::key_value_typed_store_impl::KeyValueTypedStoreImpl;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::{
    helper::no_ops_casper_effect::NoOpsCasperEffect,
    util::{
        genesis_builder::GenesisBuilder, rholang::resources::mk_runtime_manager,
        test_mocks::MockKeyValueStore,
    },
};
use prost::Message;

fn endpoint(port: u32) -> Endpoint {
    Endpoint {
        host: "host".to_string(),
        tcp_port: port,
        udp_port: port,
    }
}

pub fn peer_node(name: &str, port: u32) -> PeerNode {
    PeerNode {
        id: NodeIdentifier {
            key: Bytes::from(name.as_bytes().to_vec()),
        },
        endpoint: endpoint(port),
    }
}

/// Test fixture struct to hold all test dependencies
pub struct TestFixture<'a> {
    pub transport_layer: Arc<TransportLayerStub>,
    pub local_peer: PeerNode,
    pub validator_identity: ValidatorIdentity,
    pub casper: NoOpsCasperEffect,
    pub engine: Running<'a, NoOpsCasperEffect, TransportLayerStub>,
    pub block_processing_queue: Arc<Mutex<VecDeque<(NoOpsCasperEffect, BlockMessage)>>>,
}

impl<'a> TestFixture<'a> {
    pub async fn new() -> Self {
        let local_peer = peer_node("test-peer", 40400);
        let connections = Connections::from_vec(vec![local_peer.clone()]);
        let connections_cell = ConnectionsCell {
            peers: Arc::new(Mutex::new(connections.clone())),
        };
        let connections_cell_for_retriever = ConnectionsCell {
            peers: Arc::new(Mutex::new(connections)),
        };
        let rp_conf = create_rp_conf_ask(local_peer.clone(), None, None);
        let transport_layer = Arc::new(TransportLayerStub::new());

        // Create validator identity
        let private_key_bytes = Bytes::from(vec![1u8; 32]);
        let private_key = PrivateKey::new(private_key_bytes);
        let validator_identity = ValidatorIdentity::new(&private_key);

        let genesis = create_test_genesis();
        let approved_block = ApprovedBlock {
            candidate: ApprovedBlockCandidate {
                block: genesis.clone(),
                required_sigs: 0,
            },
            sigs: Vec::new(),
        };

        // Create dependencies for NoOpsCasperEffect
        let mut blocks = HashMap::new();
        blocks.insert(genesis.block_hash.clone(), genesis.clone());

        // For testing, we'll use minimal stub implementations that satisfy the interface
        // but don't need full functionality since NoOpsCasperEffect returns todo!() for most methods

        // Create mock stores for the KeyValueBlockStore
        let store = Box::new(MockKeyValueStore::new());
        let store_approved_block = Box::new(MockKeyValueStore::new());
        let block_store = KeyValueBlockStore::new(store, store_approved_block);

        // Since NoOpsCasperEffect is meant for testing and most methods return todo!() or defaults,
        // we can provide minimal implementations that satisfy the type system

        // For the runtime manager, DAG storage - since these are complex to construct and NoOpsCasperEffect
        // doesn't actually use them meaningfully in most methods, let's provide a simpler approach

        // Create a basic DAG representation using the Default implementation from the no_ops_casper_effect helper
        use block_storage::rust::dag::block_dag_key_value_storage::DeployId;
        use block_storage::rust::dag::block_metadata_store::BlockMetadataStore;
        use models::rust::block_hash::BlockHashSerde;
        use models::rust::block_metadata::BlockMetadata;

        let metadata_store = Box::new(MockKeyValueStore::new());
        let metadata_typed_store =
            KeyValueTypedStoreImpl::<BlockHashSerde, BlockMetadata>::new(metadata_store);
        let block_metadata_store = BlockMetadataStore::new(metadata_typed_store);

        let deploy_store = Box::new(MockKeyValueStore::new());
        let deploy_typed_store =
            KeyValueTypedStoreImpl::<DeployId, BlockHashSerde>::new(deploy_store);

        let block_dag_storage = KeyValueDagRepresentation {
            dag_set: Default::default(),
            latest_messages_map: Default::default(),
            child_map: Default::default(),
            height_map: Default::default(),
            invalid_blocks_set: Default::default(),
            last_finalized_block_hash: genesis.block_hash.clone(), // Set to genesis block hash so engine can find it
            finalized_blocks_set: Default::default(),
            block_metadata_index: Arc::new(std::sync::RwLock::new(block_metadata_store)),
            deploy_index: Arc::new(std::sync::RwLock::new(deploy_typed_store)),
        };

        // Create NoOpsCasperEffect with minimal dependencies
        // Use the proper test utility to create a RuntimeManager
        let runtime_manager = mk_runtime_manager("running_spec_test", None).await;

        let mut casper = NoOpsCasperEffect::new(
            Some(blocks),
            None, // estimator_func
            runtime_manager,
            block_store,
            block_dag_storage,
        );

        // Add the genesis block using the new test-friendly methods
        casper.add_block_to_store(genesis.clone());
        casper.add_to_dag(genesis.block_hash.clone());

        let block_processing_queue: Arc<Mutex<VecDeque<(NoOpsCasperEffect, BlockMessage)>>> =
            Arc::new(Mutex::new(VecDeque::new()));

        let block_retriever = Arc::new(block_retriever::BlockRetriever::new(
            transport_layer.clone(),
            Arc::new(connections_cell_for_retriever),
            Arc::new(rp_conf.clone()),
        ));

        let engine = Running::new(
            block_processing_queue.clone(),
            Arc::new(Mutex::new(Default::default())),
            casper.clone(),
            approved_block,
            Arc::new(|| Ok(())),
            false,
            connections_cell,
            transport_layer.clone(),
            rp_conf,
            block_retriever,
        );

        Self {
            transport_layer,
            local_peer,
            validator_identity,
            casper,
            engine,
            block_processing_queue,
        }
    }

    /// Get the current length of the block processing queue (for testing)
    pub fn block_processing_queue_len(&self) -> usize {
        match self.block_processing_queue.lock() {
            Ok(queue) => queue.len(),
            Err(_) => 0,
        }
    }

    /// Check if a block with the given hash is in the processing queue (for testing)
    pub fn is_block_in_processing_queue(&self, hash: &BlockHash) -> bool {
        match self.block_processing_queue.lock() {
            Ok(queue) => queue.iter().any(|(_, block)| &block.block_hash == hash),
            Err(_) => false,
        }
    }
}

/// Create a genesis block for testing
pub fn create_test_genesis() -> BlockMessage {
    let private_key_bytes = Bytes::from(vec![1u8; 32]);
    let public_key_bytes = Bytes::from(vec![2u8; 33]);
    let validator_keys = vec![(
        PrivateKey::new(private_key_bytes),
        PublicKey::new(public_key_bytes),
    )];
    GenesisBuilder::build_test_genesis(validator_keys)
}

pub fn to_casper_message(p: Protocol) -> CasperMessage {
    if let Some(packet) = p.message {
        if let models::routing::protocol::Message::Packet(packet_data) = packet {
            // This is a simplified stand-in for the full conversion logic,
            // which would involve looking at the typeId of the packet.
            // For these tests, we can make assumptions about the message type.
            if let Ok(bm) = models::casper::BlockMessageProto::decode(packet_data.content.as_ref())
            {
                if let Ok(block_message) = BlockMessage::from_proto(bm) {
                    return CasperMessage::BlockMessage(block_message);
                }
            }
            if let Ok(ab) = models::casper::ApprovedBlockProto::decode(packet_data.content.as_ref())
            {
                if let Ok(approved_block) = ApprovedBlock::from_proto(ab) {
                    return CasperMessage::ApprovedBlock(approved_block);
                }
            }
            if let Ok(hb) = models::casper::HasBlockProto::decode(packet_data.content.as_ref()) {
                let has_block = HasBlock::from_proto(hb);
                return CasperMessage::HasBlock(has_block);
            }
        }
    }
    panic!("Could not convert protocol to casper message");
}
