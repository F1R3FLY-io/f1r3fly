// See casper/src/test/scala/coop/rchain/casper/engine/RunningSpec.scala

use casper::rust::{
    casper::MultiParentCasper,
    engine::{block_retriever, running::Running},
    validator_identity::ValidatorIdentity,
};
use comm::rust::{
    peer_node::PeerNode,
    rp::connect::{Connections, ConnectionsCell},
    test_instances::{create_rp_conf_ask, TransportLayerStub},
};
use crypto::rust::{private_key::PrivateKey, public_key::PublicKey};
use models::{
    routing::Protocol,
    rust::{
        block_implicits::get_random_block,
        casper::protocol::casper_message::{
            ApprovedBlock, ApprovedBlockCandidate, BlockMessage, BlockRequest, CasperMessage,
            ForkChoiceTipRequest, HasBlock,
        },
        validator::Validator,
    },
};
use prost::bytes::Bytes;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio;

use crate::{
    engine::setup::peer_node, helper::mock_casper::MockCasper,
    util::genesis_builder::GenesisBuilder,
};
use casper::rust::engine::engine::Engine;
use prost::Message;

/// Test fixture struct to hold all test dependencies
struct TestFixture<'a> {
    transport_layer: Arc<TransportLayerStub>,
    local_peer: PeerNode,
    validator_identity: ValidatorIdentity,
    casper: MockCasper,
    engine: Running<'a, MockCasper, TransportLayerStub>,
}

impl<'a> TestFixture<'a> {
    async fn new() -> Self {
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

        let casper = MockCasper::new(approved_block.clone());
        casper.add_block_to_store(genesis.clone());
        casper.add_to_dag(genesis.block_hash.clone());

        let block_processing_queue = VecDeque::new();

        let block_retriever = Arc::new(block_retriever::BlockRetriever::new(
            transport_layer.clone(),
            Arc::new(connections_cell_for_retriever),
            Arc::new(rp_conf.clone()),
        ));

        let engine = Running::new(
            block_processing_queue,
            Arc::new(Mutex::new(Default::default())),
            casper.clone(),
            approved_block,
            Some(validator_identity.clone()),
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
        }
    }
}

/// Create a genesis block for testing
fn create_test_genesis() -> BlockMessage {
    let private_key_bytes = Bytes::from(vec![1u8; 32]);
    let public_key_bytes = Bytes::from(vec![2u8; 33]);
    let validator_keys = vec![(
        PrivateKey::new(private_key_bytes),
        PublicKey::new(public_key_bytes),
    )];
    GenesisBuilder::build_test_genesis(validator_keys)
}

fn to_casper_message(p: Protocol) -> CasperMessage {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engine_should_enqueue_block_message_for_processing() {
        let mut fixture = TestFixture::new().await;
        let new_block = create_test_genesis();

        let signed_block = fixture.validator_identity.sign_block(&new_block);

        fixture
            .engine
            .handle(
                fixture.local_peer.clone(),
                CasperMessage::BlockMessage(signed_block.clone()),
            )
            .await
            .unwrap();

        // Verify the block was enqueued for processing (following Scala test behavior)
        assert!(
            fixture
                .engine
                .is_block_in_processing_queue(&signed_block.block_hash),
            "Block should be enqueued in processing queue after being handled"
        );
    }

    #[tokio::test]
    async fn engine_should_respond_to_block_request() {
        let mut fixture = TestFixture::new().await;
        // Use the genesis block that's already stored in the approved block
        let genesis = fixture.casper.get_approved_block().candidate.block.clone();

        let block_request = BlockRequest {
            hash: genesis.block_hash.clone(),
        };

        fixture
            .engine
            .handle(
                fixture.local_peer.clone(),
                CasperMessage::BlockRequest(block_request),
            )
            .await
            .unwrap();

        assert_eq!(fixture.transport_layer.request_count(), 1);
        let sent_request = fixture.transport_layer.pop_request().unwrap();
        assert_eq!(sent_request.peer, fixture.local_peer);
        if let CasperMessage::BlockMessage(sent_msg) = to_casper_message(sent_request.msg) {
            assert_eq!(sent_msg, genesis);
        } else {
            panic!("Expected BlockMessage");
        }
    }

    #[tokio::test]
    async fn engine_should_respond_to_approved_block_request() {
        let mut fixture = TestFixture::new().await;
        let approved_block_request =
            models::rust::casper::protocol::casper_message::ApprovedBlockRequest {
                identifier: "test".to_string(),
                trim_state: false,
            };
        let expected_approved_block = fixture.casper.get_approved_block().candidate.block.clone();

        fixture
            .engine
            .handle(
                fixture.local_peer.clone(),
                CasperMessage::ApprovedBlockRequest(approved_block_request),
            )
            .await
            .unwrap();

        assert_eq!(fixture.transport_layer.request_count(), 1);
        let sent_request = fixture.transport_layer.pop_request().unwrap();
        assert_eq!(sent_request.peer, fixture.local_peer);
        if let CasperMessage::ApprovedBlock(sent_msg) = to_casper_message(sent_request.msg) {
            assert_eq!(sent_msg.candidate.block, expected_approved_block);
        } else {
            panic!("Expected ApprovedBlock");
        }
    }

    #[tokio::test]
    async fn engine_should_respond_to_fork_choice_tip_request() {
        let mut fixture = TestFixture::new().await;

        // Step 1: Create a request object
        let request = ForkChoiceTipRequest {};

        // Step 2: Create 2 blocks with empty sender
        let mut block1 = get_random_block(
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        );
        block1.sender = Bytes::new(); // Empty sender

        let mut block2 = get_random_block(
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        );
        block2.sender = Bytes::new(); // Empty sender

        // Step 3: Insert blocks in blockDagStorage
        fixture.casper.add_block_to_store(block1.clone());
        fixture.casper.add_to_dag(block1.block_hash.clone());
        fixture.casper.add_block_to_store(block2.clone());
        fixture.casper.add_to_dag(block2.block_hash.clone());

        // Update latest messages to include our blocks (simulating they are tips)
        let mut tips = HashMap::new();
        tips.insert(Validator::from(vec![1; 32]), block1.block_hash.clone());
        tips.insert(Validator::from(vec![2; 32]), block2.block_hash.clone());
        fixture.casper.set_latest_messages(tips);

        // Step 4: Get tips from casper.blockDag (this happens inside the engine)
        let dag = fixture.casper.block_dag().await.unwrap();
        let tips_from_dag: Vec<_> = dag
            .latest_messages_map
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Step 5: Call engine.handle with local peer and request object
        fixture
            .engine
            .handle(
                fixture.local_peer.clone(),
                CasperMessage::ForkChoiceTipRequest(request),
            )
            .await
            .unwrap();

        // Step 6: Get requests from transportLayer
        let requests = fixture.transport_layer.get_all_requests();

        // Step 7: Create Expected Tip value
        let expected_tips: HashSet<_> = tips_from_dag.into_iter().collect();

        // Step 8: Assert peer in head in requests in transport layer is local
        assert!(!requests.is_empty());
        let first_request = &requests[0];
        assert_eq!(first_request.peer, fixture.local_peer);
        let second_request = &requests[1];
        assert_eq!(second_request.peer, fixture.local_peer);

        // Step 9: Assert requests matches to expected tips value
        let mut received_tips = HashSet::new();
        for request in requests {
            if let CasperMessage::HasBlock(HasBlock { hash }) = to_casper_message(request.msg) {
                received_tips.insert(hash);
            }
        }

        assert_eq!(received_tips, expected_tips);
    }
}
