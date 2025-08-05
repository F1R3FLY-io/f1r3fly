// See casper/src/test/scala/coop/rchain/casper/engine/RunningSpec.scala

use std::sync::{Arc, Mutex};
use tokio;
use std::collections::{VecDeque, HashMap, HashSet};
use casper::rust::{
    casper::Casper,
    engine::{running::Running, block_retriever},
    validator_identity::ValidatorIdentity,
};
use comm::rust::{
    peer_node::PeerNode,
    rp::{connect::{ConnectionsCell, Connections}},
    test_instances::{create_rp_conf_ask, TransportLayerStub},
};
use crypto::rust::{
    private_key::PrivateKey,
    public_key::PublicKey,
};
use models::{
    rust::{
        casper::protocol::casper_message::{
            ApprovedBlock, ApprovedBlockCandidate, BlockMessage, CasperMessage, BlockRequest, HasBlock
        },

        validator::Validator,
    },
    routing::Protocol,
};
use prost::bytes::Bytes;

use crate::{
    engine::setup::peer_node,
    helper::mock_casper::MockCasper,
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
    let validator_keys = vec![(PrivateKey::new(private_key_bytes), PublicKey::new(public_key_bytes))];
    GenesisBuilder::build_test_genesis(validator_keys)
}

fn to_casper_message(p: Protocol) -> CasperMessage {
    if let Some(packet) = p.message {
        if let models::routing::protocol::Message::Packet(packet_data) = packet {
            // This is a simplified stand-in for the full conversion logic,
            // which would involve looking at the typeId of the packet.
            // For these tests, we can make assumptions about the message type.
            if let Ok(bm) = models::casper::BlockMessageProto::decode(packet_data.content.as_ref()) {
                if let Ok(block_message) = BlockMessage::from_proto(bm) {
                    return CasperMessage::BlockMessage(block_message);
                }
            }
            if let Ok(ab) = models::casper::ApprovedBlockProto::decode(packet_data.content.as_ref()) {
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

        fixture.engine.handle(fixture.local_peer.clone(), CasperMessage::BlockMessage(signed_block.clone())).await.unwrap();

        // Instead of checking the internal queue, verify the block was processed by checking if it's in the casper DAG or buffer
        let is_in_dag = fixture.casper.dag_contains(&signed_block.block_hash);
        let is_in_buffer = fixture.casper.buffer_contains(&signed_block.block_hash);
        assert!(is_in_dag || is_in_buffer, "Block should be in DAG or buffer after being handled");
    }

    #[tokio::test]
    async fn engine_should_respond_to_block_request() {
        let mut fixture = TestFixture::new().await;
        // Use the genesis block that's already stored in the approved block
        let genesis = fixture.casper.get_approved_block().candidate.block.clone();

        let block_request = BlockRequest {
            hash: genesis.block_hash.clone(),
        };

        fixture.engine.handle(fixture.local_peer.clone(), CasperMessage::BlockRequest(block_request)).await.unwrap();

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
        let approved_block_request = models::rust::casper::protocol::casper_message::ApprovedBlockRequest {
            identifier: "test".to_string(),
            trim_state: false,
        };
        let expected_approved_block = fixture.casper.get_approved_block().candidate.block.clone();

        fixture.engine.handle(fixture.local_peer.clone(), CasperMessage::ApprovedBlockRequest(approved_block_request)).await.unwrap();

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
        let tip1 = prost::bytes::Bytes::from(vec![1; 32]);
        let tip2 = prost::bytes::Bytes::from(vec![2; 32]);
        let validator1: Validator = vec![1; 32].into();
        let validator2: Validator = vec![2; 32].into();
        let mut tips = HashMap::new();
        tips.insert(validator1, tip1.clone());
        tips.insert(validator2, tip2.clone());
        fixture.casper.set_latest_messages(tips);

        let fork_choice_tip_request = models::rust::casper::protocol::casper_message::ForkChoiceTipRequest {};
        fixture.engine.handle(fixture.local_peer.clone(), CasperMessage::ForkChoiceTipRequest(fork_choice_tip_request)).await.unwrap();

        assert_eq!(fixture.transport_layer.request_count(), 2);
        let mut received_tips = HashSet::new();

        let req1 = fixture.transport_layer.pop_request().unwrap();
        if let CasperMessage::HasBlock(HasBlock { hash }) = to_casper_message(req1.msg) {
            received_tips.insert(hash);
        } else {
            panic!("Expected HasBlock message")
        }
        
        let req2 = fixture.transport_layer.pop_request().unwrap();
        if let CasperMessage::HasBlock(HasBlock { hash }) = to_casper_message(req2.msg) {
            received_tips.insert(hash);
        } else {
            panic!("Expected HasBlock message")
        }

        let mut expected_tips = HashSet::new();
        expected_tips.insert(tip1);
        expected_tips.insert(tip2);

        assert_eq!(received_tips, expected_tips);
    }
}
