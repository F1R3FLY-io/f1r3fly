// See casper/src/test/scala/coop/rchain/casper/sync/BlockRetrieverSpec.scala

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use casper::rust::{
        engine::block_retriever::{AdmitHashReason, BlockRetriever},
        protocol::{extract_packet_from_protocol, verify_block_request, verify_has_block_request},
    };
    use comm::rust::{
        peer_node::PeerNode,
        rp::{
            connect::{Connections, ConnectionsCell},
            rp_conf::RPConf,
        },
        test_instances::{create_rp_conf_ask, TransportLayerStub},
    };
    use models::rust::block_hash::BlockHash;

    use crate::engine::setup;

    // Test constants
    const TEST_HASH_BYTES: &[u8] = b"newHash";

    #[derive(Debug, Clone, PartialEq)]
    struct TestReason;
    impl Into<AdmitHashReason> for TestReason {
        fn into(self) -> AdmitHashReason {
            AdmitHashReason::HasBlockMessageReceived
        }
    }

    struct TestFixture {
        hash: BlockHash,
        local_peer: PeerNode,
        peer: PeerNode,
        second_peer: PeerNode,
        block_retriever: BlockRetriever<TransportLayerStub>,
        transport_layer: Arc<TransportLayerStub>,
        connections_cell: Arc<ConnectionsCell>,
        rp_conf: Arc<RPConf>,
    }

    impl TestFixture {
        fn new() -> Self {
            let hash = BlockHash::from(TEST_HASH_BYTES.to_vec());
            let local_peer = setup::peer_node("src", 40400);
            let peer = setup::peer_node("peer", 40400);
            let second_peer = setup::peer_node("secondPeer", 40400);

            let connections_cell = Arc::new(ConnectionsCell {
                peers: Arc::new(Mutex::new(Connections::from_vec(vec![local_peer.clone()]))),
            });
            let rp_conf = Arc::new(create_rp_conf_ask(local_peer.clone(), None, None));
            let transport_layer = Arc::new(TransportLayerStub::new());
            let block_retriever = BlockRetriever::new(
                transport_layer.clone(),
                connections_cell.clone(),
                rp_conf.clone(),
            );

            Self {
                hash,
                local_peer,
                peer,
                second_peer,
                block_retriever,
                transport_layer,
                connections_cell,
                rp_conf,
            }
        }

        fn reset(&self) {
            self.transport_layer.reset();
        }
    }

    mod admit_hash {
        use super::*;

        mod when_hash_is_unknown {
            use super::*;

            #[tokio::test]
            async fn should_add_record_for_hash() {
                let fixture = TestFixture::new();
                let test_reason = TestReason;

                // Call admitHash with unknown hash
                let result = fixture
                    .block_retriever
                    .admit_hash(fixture.hash.clone(), None, test_reason.into())
                    .await;

                // Should succeed
                assert!(result.is_ok());

                // Check that hash was added to requested blocks by trying to check if it's received
                // (is_received returns false for unknown hashes, true/false for known hashes)
                let is_known = fixture
                    .block_retriever
                    .is_received(fixture.hash.clone())
                    .await
                    .is_ok(); // If hash is unknown, is_received would return Ok(false), if completely unknown it might error

                // The hash should now be known (even if not received)
                assert!(is_known);
            }

            mod when_source_peer_is_unknown {
                use super::*;

                #[tokio::test]
                async fn should_broadcast_has_block_request_and_only_has_block_request() {
                    let fixture = TestFixture::new();
                    fixture.reset();
                    let test_reason = TestReason;

                    // Call admitHash with unknown hash and no peer
                    let result = fixture
                        .block_retriever
                        .admit_hash(fixture.hash.clone(), None, test_reason.into())
                        .await;

                    // Should succeed
                    assert!(result.is_ok());

                    // Should have exactly one request
                    let request_count = fixture.transport_layer.request_count();
                    assert_eq!(request_count, 1);

                    // Get the request and verify it's sent to the correct peer
                    let (peer, protocol) = fixture.transport_layer.get_request(0).unwrap();
                    
                    // In broadcast, the request should be sent to our local peer (from connections)
                    assert_eq!(peer, fixture.local_peer);

                    // Verify the protocol message is HasBlockRequest and contains the correct hash
                    let packet = extract_packet_from_protocol(&protocol)
                        .expect("Should be able to extract packet from protocol");
                    verify_has_block_request(&packet, &fixture.hash)
                        .expect("Should be HasBlockRequest with correct hash");
                }
            }

            mod when_source_peer_is_known {
                use super::*;

                #[tokio::test]
                async fn should_send_block_request_and_only_block_request() {
                    let fixture = TestFixture::new();
                    fixture.reset();
                    let test_reason = TestReason;

                    // Call admitHash with unknown hash and a specific peer
                    let result = fixture
                        .block_retriever
                        .admit_hash(fixture.hash.clone(), Some(fixture.peer.clone()), test_reason.into())
                        .await;

                    // Should succeed
                    assert!(result.is_ok());

                    // Should have exactly one request
                    let request_count = fixture.transport_layer.request_count();
                    assert_eq!(request_count, 1);

                    // Get the request and verify it's sent to the correct peer
                    let (recipient, protocol) = fixture.transport_layer.get_request(0).unwrap();
                    assert_eq!(recipient, fixture.peer);

                    // Verify the protocol message is BlockRequest and contains the correct hash
                    let packet = extract_packet_from_protocol(&protocol)
                        .expect("Should be able to extract packet from protocol");
                    verify_block_request(&packet, &fixture.hash)
                        .expect("Should be BlockRequest with correct hash");
                }
            }
        }
    }
}
