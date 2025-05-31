// See comm/src/main/scala/coop/rchain/comm/transport/StreamObservable.scala
// See comm/src/main/scala/coop/rchain/comm/transport/buffer/LimitedBufferObservable.scala
// See comm/src/main/scala/coop/rchain/comm/transport/buffer/LimitedBuffer.scala
// See comm/src/main/scala/coop/rchain/comm/transport/buffer/ConcurrentQueue.scala

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio_stream::Stream as TokioStream;

use crate::rust::{
    errors::CommError,
    peer_node::PeerNode,
    transport::{packet_ops::PacketExt, packet_ops::StreamCache, transport_layer::Blob},
};

/// Stream message containing a cache key and sender peer
#[derive(Debug, Clone)]
pub struct Stream {
    pub key: String,
    pub sender: PeerNode,
}

/// StreamObservable provides bounded buffering for streaming messages with overflow handling
///
/// Implements "drop new" behavior - when buffer is full, new messages are dropped
/// Uses flume bounded channels for lock-free performance
pub struct StreamObservable {
    peer: PeerNode,
    buffer_size: usize,
    cache: StreamCache,
    sender: flume::Sender<Stream>,
    receiver: Option<flume::Receiver<Stream>>,
    // State management
    upstream_complete: Arc<AtomicBool>,
    downstream_complete: Arc<AtomicBool>,
}

impl StreamObservable {
    /// Create a new StreamObservable with the given peer, buffer size, and cache
    pub fn new(peer: PeerNode, buffer_size: usize, cache: StreamCache) -> Self {
        // Validate buffer size
        assert!(
            buffer_size > 0,
            "bufferSize must be a strictly positive number"
        );

        let (sender, receiver) = flume::bounded(buffer_size);

        Self {
            peer,
            buffer_size,
            cache,
            sender,
            receiver: Some(receiver),
            upstream_complete: Arc::new(AtomicBool::new(false)),
            downstream_complete: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Enqueue a blob for streaming
    pub async fn enque(&self, blob: &Blob) -> Result<(), CommError> {
        // Check completion states first
        if self.upstream_complete.load(Ordering::Acquire)
            || self.downstream_complete.load(Ordering::Acquire)
        {
            // Return early if completed
            return Ok(());
        }

        // Log stream information
        log::debug!(
            "Pushing message to {} stream message queue.",
            self.peer.endpoint.host
        );

        // Store blob packet in cache
        let key = blob.packet.store(&self.cache)?;

        // Create stream message
        let stream_msg = Stream {
            key: key.clone(),
            sender: blob.sender.clone(),
        };

        // Try to push to bounded queue (non-blocking)
        match self.sender.try_send(stream_msg) {
            Ok(()) => {
                // Successfully enqueued
                Ok(())
            }
            Err(flume::TrySendError::Full(_)) => {
                // Buffer is full - implement "drop new" behavior
                log::warn!(
                    "Client stream message queue for {} is full ({} items). Dropping message.",
                    self.peer.endpoint.host,
                    self.buffer_size
                );
                self.cache.remove(&key);
                // Still return Ok(())
                Ok(())
            }
            Err(flume::TrySendError::Disconnected(_)) => {
                // Receiver has been dropped - mark downstream as complete
                log::warn!("Stream queue for {} is closed", self.peer.endpoint.host);
                self.downstream_complete.store(true, Ordering::Release);
                self.cache.remove(&key);
                // Still return Ok(())
                Ok(())
            }
        }
    }

    /// Complete the upstream
    /// Signals that no more items will be pushed
    pub fn complete(&self) {
        if !self.upstream_complete.load(Ordering::Acquire)
            && !self.downstream_complete.load(Ordering::Acquire)
        {
            self.upstream_complete.store(true, Ordering::Release);
            log::debug!("Stream for {} marked as complete", self.peer.endpoint.host);
        }
    }

    /// Check if upstream is complete
    pub fn is_upstream_complete(&self) -> bool {
        self.upstream_complete.load(Ordering::Acquire)
    }

    /// Check if downstream is complete  
    pub fn is_downstream_complete(&self) -> bool {
        self.downstream_complete.load(Ordering::Acquire)
    }

    /// Check if the stream is fully complete (both upstream and downstream)
    pub fn is_complete(&self) -> bool {
        self.is_upstream_complete() && self.is_downstream_complete()
    }

    /// Get a stream subscription
    pub fn subscribe(&mut self) -> Option<StreamSubscription> {
        self.receiver.take().map(|receiver| StreamSubscription {
            receiver,
            downstream_complete: self.downstream_complete.clone(),
        })
    }

    /// Get the peer this observable is associated with
    pub fn peer(&self) -> &PeerNode {
        &self.peer
    }

    /// Get the buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Check if the sender is still active (not closed)
    pub fn is_active(&self) -> bool {
        !self.sender.is_disconnected()
    }
}

/// Subscription handle that automatically manages downstream completion state
pub struct StreamSubscription {
    receiver: flume::Receiver<Stream>,
    downstream_complete: Arc<AtomicBool>,
}

impl StreamSubscription {
    /// Cancel the subscription
    /// Marks downstream as complete and closes the receiver
    pub fn cancel(self) {
        self.downstream_complete.store(true, Ordering::Release);
        // Receiver will be dropped here, closing the stream
    }

    /// Check if the subscription is still active
    pub fn is_active(&self) -> bool {
        !self.receiver.is_disconnected()
    }
}

impl Drop for StreamSubscription {
    fn drop(&mut self) {
        // Automatically mark downstream as complete when subscription is dropped
        self.downstream_complete.store(true, Ordering::Release);
    }
}

// Implement Stream trait for StreamSubscription to make it directly usable with tokio-stream
impl TokioStream for StreamSubscription {
    type Item = Stream;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Use flume's async receive functionality
        match self.receiver.try_recv() {
            Ok(item) => std::task::Poll::Ready(Some(item)),
            Err(flume::TryRecvError::Empty) => {
                // Register waker for future notification
                let waker = cx.waker().clone();
                let receiver = self.receiver.clone();
                tokio::spawn(async move {
                    if receiver.recv_async().await.is_ok() {
                        waker.wake();
                    }
                });
                std::task::Poll::Pending
            }
            Err(flume::TryRecvError::Disconnected) => std::task::Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::peer_node::{Endpoint, NodeIdentifier};
    use dashmap::DashMap;
    use models::routing::Packet;
    use prost::bytes::Bytes;
    use std::sync::Arc;
    use tokio_stream::StreamExt;

    fn create_test_peer() -> PeerNode {
        PeerNode {
            id: NodeIdentifier {
                key: Bytes::from("test_peer"),
            },
            endpoint: Endpoint::new("127.0.0.1".to_string(), 8080, 8080),
        }
    }

    fn create_test_cache() -> StreamCache {
        Arc::new(DashMap::new())
    }

    fn create_test_blob(sender: PeerNode, content: Vec<u8>) -> Blob {
        Blob {
            sender,
            packet: Packet {
                type_id: "TestPacket".to_string(),
                content: Bytes::from(content),
            },
        }
    }

    #[tokio::test]
    async fn test_stream_observable_creation() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let buffer_size = 10;

        let observable = StreamObservable::new(peer.clone(), buffer_size, cache);

        assert_eq!(observable.peer().id.key, peer.id.key);
        assert_eq!(observable.buffer_size(), buffer_size);
        assert!(observable.is_active());
        assert!(!observable.is_upstream_complete());
        assert!(!observable.is_downstream_complete());
        assert!(!observable.is_complete());
    }

    #[tokio::test]
    #[should_panic(expected = "bufferSize must be a strictly positive number")]
    async fn test_invalid_buffer_size() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let _observable = StreamObservable::new(peer, 0, cache);
    }

    #[tokio::test]
    async fn test_enque_and_subscribe() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let mut observable = StreamObservable::new(peer.clone(), 10, cache.clone());

        // Get the stream first
        let mut subscription = observable.subscribe().expect("Should get subscription");

        // Create and enqueue a blob
        let sender = create_test_peer();
        let blob = create_test_blob(sender.clone(), vec![1, 2, 3, 4, 5]);

        let result = observable.enque(&blob).await;
        assert!(result.is_ok());

        // Read from stream
        if let Some(stream_msg) = subscription.next().await {
            assert!(stream_msg.key.starts_with("packet_receive/"));
            assert_eq!(stream_msg.sender.id.key, sender.id.key);

            // Verify packet was stored in cache
            assert!(cache.contains_key(&stream_msg.key));
        } else {
            panic!("Should receive a stream message");
        }
    }

    #[tokio::test]
    async fn test_completion_states() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let observable = StreamObservable::new(peer.clone(), 10, cache);

        // Initially not complete
        assert!(!observable.is_upstream_complete());
        assert!(!observable.is_downstream_complete());
        assert!(!observable.is_complete());

        // Complete upstream
        observable.complete();
        assert!(observable.is_upstream_complete());
        assert!(!observable.is_downstream_complete());
        assert!(!observable.is_complete());

        // Try to enqueue after completion
        let sender = create_test_peer();
        let blob = create_test_blob(sender, vec![1, 2, 3]);
        let result = observable.enque(&blob).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subscription_lifecycle() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let mut observable = StreamObservable::new(peer.clone(), 10, cache);

        assert!(!observable.is_downstream_complete());

        // Create and drop subscription
        let subscription = observable.subscribe().expect("Should get subscription");
        drop(subscription);

        // Give some time for the drop to take effect
        tokio::task::yield_now().await;

        // Downstream should be marked complete
        assert!(observable.is_downstream_complete());
    }

    #[tokio::test]
    async fn test_buffer_overflow_drop_new_behavior() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let buffer_size = 2; // Small buffer to test overflow
        let observable = StreamObservable::new(peer.clone(), buffer_size, cache.clone());

        // Don't subscribe yet - this will cause buffer to fill up
        let sender = create_test_peer();

        // Fill the buffer
        for i in 0..buffer_size {
            let blob = create_test_blob(sender.clone(), vec![i as u8; 10]);
            let result = observable.enque(&blob).await;
            assert!(
                result.is_ok(),
                "Buffer should accept first {} items",
                buffer_size
            );
        }

        // This should still return Ok(()) but drop the message
        let overflow_blob = create_test_blob(sender.clone(), vec![99; 10]);
        let result = observable.enque(&overflow_blob).await;
        assert!(result.is_ok(), "Always returns success even when dropping");

        // Cache should only contain the successfully stored items
        assert_eq!(cache.len(), buffer_size);
    }

    #[tokio::test]
    async fn test_subscription_cancel() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let mut observable = StreamObservable::new(peer.clone(), 10, cache);

        let subscription = observable.subscribe().expect("Should get subscription");
        assert!(subscription.is_active());

        // Cancel the subscription
        subscription.cancel();

        // Downstream should be marked complete
        assert!(observable.is_downstream_complete());
    }

    #[tokio::test]
    async fn test_multiple_enque_operations() {
        let peer = create_test_peer();
        let cache = create_test_cache();
        let mut observable = StreamObservable::new(peer.clone(), 10, cache.clone());

        let mut subscription = observable.subscribe().expect("Should get subscription");

        // Enqueue multiple blobs
        let sender = create_test_peer();
        for i in 0..3 {
            let blob = create_test_blob(sender.clone(), vec![i; 10]);
            let result = observable.enque(&blob).await;
            assert!(result.is_ok());
        }

        // Read all messages
        let mut received_count = 0;
        while let Some(_stream_msg) = subscription.next().await {
            received_count += 1;
            if received_count == 3 {
                break;
            }
        }

        assert_eq!(received_count, 3);
        assert_eq!(cache.len(), 3); // All packets should be in cache
    }

    #[tokio::test]
    async fn test_stream_message_properties() {
        let _peer = create_test_peer();
        let sender = create_test_peer();

        let stream_msg = Stream {
            key: "test_key".to_string(),
            sender: sender.clone(),
        };

        assert_eq!(stream_msg.key, "test_key");
        assert_eq!(stream_msg.sender.id.key, sender.id.key);
    }
}
