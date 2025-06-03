// See comm/src/main/scala/coop/rchain/comm/transport/buffer/LimitedBuffer.scala
// See comm/src/main/scala/coop/rchain/comm/transport/buffer/LimitedBufferObservable.scala
// See comm/src/main/scala/coop/rchain/comm/transport/buffer/ConcurrentQueue.scala

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio_stream::Stream;

/// LimitedBuffer trait providing bounded buffering with overflow policy
pub trait LimitedBuffer<T> {
    /// Push the next element to the buffer
    /// Returns true if successfully enqueued, false if dropped due to overflow
    fn push_next(&self, elem: T) -> bool;

    /// Signal completion - no more elements will be pushed
    fn complete(&self);

    /// Check if the buffer is complete
    fn is_complete(&self) -> bool;
}

/// Observable interface for LimitedBuffer
pub trait LimitedBufferObservable<T>: LimitedBuffer<T> {
    type Subscription: Stream<Item = T> + Unpin;

    /// Subscribe to receive items from this buffer
    fn subscribe(&mut self) -> Option<Self::Subscription>;
}

/// FlumeLimitedBuffer: LimitedBuffer implementation using flume bounded channels
///
/// Implements "drop new" overflow policy: when buffer is full, new items are rejected
/// and push_next returns false.
#[derive(Debug)]
pub struct FlumeLimitedBuffer<T> {
    sender: flume::Sender<T>,
    receiver: Option<flume::Receiver<T>>,
    buffer_size: usize,
    // Completion state management
    complete: Arc<AtomicBool>,
}

impl<T> FlumeLimitedBuffer<T> {
    /// Create a new FlumeLimitedBuffer with the specified buffer size
    pub fn drop_new(buffer_size: usize) -> Self {
        assert!(
            buffer_size > 0,
            "bufferSize must be a strictly positive number"
        );

        let (sender, receiver) = flume::bounded(buffer_size);

        Self {
            sender,
            receiver: Some(receiver),
            buffer_size,
            complete: Arc::new(AtomicBool::new(false)),
        }
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

impl<T> LimitedBuffer<T> for FlumeLimitedBuffer<T> {
    fn push_next(&self, elem: T) -> bool {
        if self.complete.load(Ordering::Acquire) {
            return false;
        }

        match self.sender.try_send(elem) {
            Ok(()) => true,
            Err(flume::TrySendError::Full(_)) => {
                // Buffer is full - implement "drop new" behavior
                false
            }
            Err(flume::TrySendError::Disconnected(_)) => {
                // Receiver has been dropped
                false
            }
        }
    }

    fn complete(&self) {
        self.complete.store(true, Ordering::Release);
        // Closing the sender will signal EOF to receivers
        // We don't actually close it here because we want pushNext to keep working
        // The receiver will see completion when it's empty and complete flag is set
    }

    fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Acquire)
    }
}

/// Subscription handle for FlumeLimitedBuffer
pub struct FlumeLimitedBufferSubscription<T> {
    receiver: flume::Receiver<T>,
    complete: Arc<AtomicBool>,
}

impl<T: Send + 'static> Stream for FlumeLimitedBufferSubscription<T> {
    type Item = T;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Try to receive an item immediately
        match self.receiver.try_recv() {
            Ok(item) => std::task::Poll::Ready(Some(item)),
            Err(flume::TryRecvError::Empty) => {
                // Buffer is empty - check if we're complete
                if self.complete.load(Ordering::Acquire) {
                    // Complete and empty - end the stream
                    std::task::Poll::Ready(None)
                } else {
                    // Not complete yet - register for future notification
                    let waker = cx.waker().clone();
                    let receiver = self.receiver.clone();
                    let complete = self.complete.clone();

                    tokio::spawn(async move {
                        // Use flume's async API instead of polling
                        tokio::select! {
                            _ = receiver.recv_async() => {
                                // Either received a message or sender was disconnected
                                waker.wake();
                            }
                            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                                // Check completion status periodically
                                if complete.load(Ordering::Acquire) && receiver.is_empty() {
                                    waker.wake();
                                }
                            }
                        }
                    });

                    std::task::Poll::Pending
                }
            }
            Err(flume::TryRecvError::Disconnected) => {
                // Sender disconnected - end the stream
                std::task::Poll::Ready(None)
            }
        }
    }
}

impl<T: Send + 'static> LimitedBufferObservable<T> for FlumeLimitedBuffer<T> {
    type Subscription = FlumeLimitedBufferSubscription<T>;

    fn subscribe(&mut self) -> Option<Self::Subscription> {
        self.receiver
            .take()
            .map(|receiver| FlumeLimitedBufferSubscription {
                receiver,
                complete: self.complete.clone(),
            })
    }
}

/// Convenience constructor functions
impl<T> FlumeLimitedBuffer<T> {
    /// Create a new drop-new limited buffer observable
    pub fn drop_new_observable(buffer_size: usize) -> Self {
        Self::drop_new(buffer_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_limited_buffer_push_next() {
        let buffer = FlumeLimitedBuffer::<i32>::drop_new(2);

        // Should accept first two items
        assert!(buffer.push_next(1));
        assert!(buffer.push_next(2));

        // Should reject third item (buffer full)
        assert!(!buffer.push_next(3));

        assert!(!buffer.is_complete());
    }

    #[tokio::test]
    async fn test_limited_buffer_completion() {
        let buffer = FlumeLimitedBuffer::<i32>::drop_new(10);

        assert!(!buffer.is_complete());

        buffer.complete();
        assert!(buffer.is_complete());

        // Should reject new items after completion
        assert!(!buffer.push_next(1));
    }

    #[tokio::test]
    async fn test_limited_buffer_subscription() {
        let mut buffer = FlumeLimitedBuffer::<i32>::drop_new(10);

        // Push some items
        assert!(buffer.push_next(1));
        assert!(buffer.push_next(2));
        assert!(buffer.push_next(3));

        // Subscribe and read items
        let mut subscription = buffer.subscribe().expect("Should get subscription");

        let items: Vec<i32> = vec![
            subscription.next().await.unwrap(),
            subscription.next().await.unwrap(),
            subscription.next().await.unwrap(),
        ];

        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_limited_buffer_completion_with_subscription() {
        let mut buffer = FlumeLimitedBuffer::<i32>::drop_new(10);

        let mut subscription = buffer.subscribe().expect("Should get subscription");

        // Push an item and complete
        assert!(buffer.push_next(42));
        buffer.complete();

        // Should receive the item
        assert_eq!(subscription.next().await, Some(42));

        // Should end the stream after completion
        assert_eq!(subscription.next().await, None);
    }

    #[test]
    fn test_buffer_size_validation() {
        // Should panic with zero buffer size
        std::panic::catch_unwind(|| {
            FlumeLimitedBuffer::<i32>::drop_new(0);
        })
        .expect_err("Should panic with zero buffer size");
    }

    #[tokio::test]
    async fn test_drop_new_behavior() {
        let mut buffer = FlumeLimitedBuffer::<String>::drop_new(2);

        // Fill buffer
        assert!(buffer.push_next("first".to_string()));
        assert!(buffer.push_next("second".to_string()));

        // These should be dropped
        assert!(!buffer.push_next("dropped1".to_string()));
        assert!(!buffer.push_next("dropped2".to_string()));

        // Subscribe and verify only first two items are present
        let mut subscription = buffer.subscribe().expect("Should get subscription");

        assert_eq!(subscription.next().await, Some("first".to_string()));
        assert_eq!(subscription.next().await, Some("second".to_string()));

        buffer.complete();
        assert_eq!(subscription.next().await, None);
    }
}
