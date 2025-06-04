// See comm/src/test/scala/coop/rchain/comm/transport/TransportLayerSpec.scala

use std::sync::Once;

use comm::rust::transport::transport_layer::{Blob, TransportLayer};
use models::routing::Packet;
use prost::bytes::Bytes;

use crate::transport::transport_layer_runtime::{
    broadcast_heartbeat, send_heartbeat, TestProtocolDispatcher, TestStreamDispatcher,
    TransportLayerTestRuntime,
};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true) // ensures logs show up in test output
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .unwrap();
    });
}

/// Create big content for streaming tests
pub fn create_big_content(max_message_size: i32) -> Bytes {
    // Create content that is 4.5 times the max message size to force streaming
    let size = (4 * max_message_size) + (max_message_size / 2);
    let content = vec![128u8; size as usize];
    Bytes::from(content)
}

#[tokio::test]
async fn sending_a_message_should_deliver_the_message() {
    init_logger();

    let runtime = TransportLayerTestRuntime::new("test".to_string());
    let protocol_dispatcher = TestProtocolDispatcher::new();

    let result = runtime
        .run_two_nodes_test(
            |transport, local, remote| async move {
                send_heartbeat(&transport, &local, &remote, "test").await
            },
            Some(protocol_dispatcher.clone()),
            None, // Use default stream dispatcher
            true, // block_until_dispatched
        )
        .await
        .expect("Test should succeed");

    // Verify message was received
    let received = protocol_dispatcher.received();
    assert_eq!(received.len(), 1);

    let (receiver_peer, protocol_msg) = &received[0];
    assert_eq!(receiver_peer, &result.remote_node);

    // Verify the sender in the protocol message matches local node
    let header = protocol_msg.header.as_ref().expect("Header should exist");
    let sender_node = header.sender.as_ref().expect("Sender should exist");
    let sender_peer = comm::rust::peer_node::PeerNode::from_node(sender_node.clone())
        .expect("Should convert to PeerNode");
    assert_eq!(sender_peer, result.local_node);

    // Verify it's a heartbeat message
    assert!(protocol_msg.message.is_some());
    if let Some(models::routing::protocol::Message::Heartbeat(_)) = &protocol_msg.message {
        // Expected heartbeat message
    } else {
        panic!("Expected heartbeat message");
    }

    // Verify the send operation succeeded
    assert!(result.result.is_ok());
}

#[tokio::test]
async fn broadcasting_a_message_should_send_the_message_to_all_peers() {
    init_logger();

    let runtime = TransportLayerTestRuntime::new("test".to_string());
    let protocol_dispatcher = TestProtocolDispatcher::new();

    let result = runtime
        .run_three_nodes_test(
            |transport, local, remote1, remote2| async move {
                broadcast_heartbeat(&transport, &local, &[remote1, remote2], "test").await
            },
            Some(protocol_dispatcher.clone()),
            None, // Use default stream dispatcher
            true, // block_until_dispatched
        )
        .await
        .expect("Test should succeed");

    // Verify messages were received by both peers
    let received = protocol_dispatcher.received();
    assert_eq!(received.len(), 2);

    let (receiver1, protocol1) = &received[0];
    let (receiver2, protocol2) = &received[1];

    // Verify senders match local node
    for protocol_msg in [protocol1, protocol2] {
        let header = protocol_msg.header.as_ref().expect("Header should exist");
        let sender_node = header.sender.as_ref().expect("Sender should exist");
        let sender_peer = comm::rust::peer_node::PeerNode::from_node(sender_node.clone())
            .expect("Should convert to PeerNode");
        assert_eq!(sender_peer, result.local_node);

        // Verify it's a heartbeat message
        assert!(protocol_msg.message.is_some());
        if let Some(models::routing::protocol::Message::Heartbeat(_)) = &protocol_msg.message {
            // Expected heartbeat message
        } else {
            panic!("Expected heartbeat message");
        }
    }

    // Verify receivers are the two remote nodes (order may vary)
    let receivers = vec![receiver1, receiver2];
    assert!(receivers.contains(&&result.remote_node1));
    assert!(receivers.contains(&&result.remote_node2));

    // Verify the broadcast operation succeeded
    assert!(result.result.is_ok());
}

#[tokio::test]
async fn stream_blob_should_send_a_blob_and_receive_by_single_remote_side() {
    init_logger();

    let runtime = TransportLayerTestRuntime::new("test".to_string());
    let stream_dispatcher = TestStreamDispatcher::new();

    let big_content = create_big_content(runtime.max_message_size);
    let big_content_for_assert = big_content.clone();

    let result = runtime
        .run_two_nodes_test(
            |transport, local, remote| async move {
                let blob = Blob {
                    sender: local.clone(),
                    packet: Packet {
                        type_id: "Test".to_string(),
                        content: big_content.clone(),
                    },
                };
                transport.stream(&remote, &blob).await
            },
            None, // Use default protocol dispatcher
            Some(stream_dispatcher.clone()),
            true, // block_until_dispatched
        )
        .await
        .expect("Test should succeed");

    // Verify blob was received
    let received = stream_dispatcher.received();
    assert_eq!(received.len(), 1);

    let (receiver_peer, received_blob) = &received[0];
    assert_eq!(receiver_peer, &result.remote_node);

    // Verify blob content
    assert_eq!(received_blob.packet.type_id, "Test");
    assert_eq!(received_blob.packet.content, big_content_for_assert);

    // Verify the stream operation succeeded
    assert!(result.result.is_ok());
}

#[tokio::test]
async fn stream_blob_should_send_a_blob_and_receive_by_multiple_remote_side() {
    init_logger();

    let runtime = TransportLayerTestRuntime::new("test".to_string());
    let stream_dispatcher = TestStreamDispatcher::new();

    let big_content = create_big_content(runtime.max_message_size);
    let big_content_for_assert = big_content.clone();

    let result = runtime
        .run_three_nodes_test(
            |transport, local, remote1, remote2| async move {
                let blob = Blob {
                    sender: local.clone(),
                    packet: Packet {
                        type_id: "N/A".to_string(),
                        content: big_content.clone(),
                    },
                };
                transport.stream_mult(&[remote1, remote2], &blob).await
            },
            None, // Use default protocol dispatcher
            Some(stream_dispatcher.clone()),
            true, // block_until_dispatched
        )
        .await
        .expect("Test should succeed");

    // Verify blobs were received by both peers
    let received = stream_dispatcher.received();
    assert_eq!(received.len(), 2);

    let (receiver1, blob1) = &received[0];
    let (receiver2, blob2) = &received[1];

    // Verify blob content for both
    for received_blob in [blob1, blob2] {
        assert_eq!(received_blob.packet.type_id, "N/A");
        assert_eq!(received_blob.packet.content, big_content_for_assert);
    }

    // Verify receivers are the two remote nodes (order may vary)
    let receivers = vec![receiver1, receiver2];
    assert!(receivers.contains(&&result.remote_node1));
    assert!(receivers.contains(&&result.remote_node2));

    // Verify the stream operation succeeded
    assert!(result.result.is_ok());
}
