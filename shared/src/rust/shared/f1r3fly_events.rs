// See node/src/main/scala/coop/rchain/node/effects/RchainEvents.scala

use futures::stream::Stream;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::broadcast;

pub use super::f1r3fly_event::F1r3flyEvent;

/// Structure to publish and consume F1r3flyEvents
pub struct F1r3flyEvents {
    queue: Arc<Mutex<VecDeque<F1r3flyEvent>>>,
    capacity: usize,
    sender: broadcast::Sender<F1r3flyEvent>,
}

impl Clone for F1r3flyEvents {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            capacity: self.capacity,
            sender: self.sender.clone(),
        }
    }
}

impl F1r3flyEvents {
    /// Create a new F1r3flyEvents with a circular buffer
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(1);
        let (sender, _) = broadcast::channel(10); // Use larger capacity for broadcast channel

        F1r3flyEvents {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
            sender,
        }
    }

    /// Create a new F1r3flyEvents with default capacity of 1
    pub fn default() -> Self {
        Self::new(None)
    }

    /// Publish an event
    pub fn publish(&self, event: F1r3flyEvent) -> Result<(), String> {
        let mut queue = match self.queue.lock() {
            Ok(queue) => queue,
            Err(_) => return Err("Failed to acquire lock on event queue".to_string()),
        };

        // If queue is full, remove oldest event (circular buffer behavior)
        if queue.len() >= self.capacity {
            queue.pop_front();
        }

        queue.push_back(event.clone());

        // Broadcast to all consumers
        let _ = self.sender.send(event);

        Ok(())
    }

    /// Get a stream to consume events
    pub fn consume(&self) -> EventStream {
        EventStream {
            receiver: self.sender.subscribe(),
        }
    }

    /// Get all events currently in the queue.
    /// NOTE: This is intended for testing purposes.
    pub fn get_events(&self) -> Vec<F1r3flyEvent> {
        self.queue.lock().unwrap().iter().cloned().collect()
    }
}

/// Stream implementation for consuming events
pub struct EventStream {
    receiver: broadcast::Receiver<F1r3flyEvent>,
}

impl Stream for EventStream {
    type Item = F1r3flyEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::future::Future;
        use std::pin::pin;

        let mut future = pin!(self.receiver.recv());

        match Future::poll(future.as_mut(), cx) {
            Poll::Ready(Ok(item)) => Poll::Ready(Some(item)),
            Poll::Ready(Err(_)) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::f1r3fly_event::{BlockAdded, BlockCreated, BlockFinalised};
    use super::*;
    use futures::StreamExt;

    fn create_block_finalised_event() -> F1r3flyEvent {
        F1r3flyEvent::block_finalised("hash123".to_string())
    }

    fn create_block_created_event() -> F1r3flyEvent {
        F1r3flyEvent::block_created(
            "hash456".to_string(),
            vec!["parent1".to_string(), "parent2".to_string()],
            vec![("j1".to_string(), "j2".to_string())],
            vec!["deploy1".to_string()],
            "creator1".to_string(),
            42,
        )
    }

    fn create_block_added_event() -> F1r3flyEvent {
        F1r3flyEvent::block_added(
            "hash789".to_string(),
            vec!["parent3".to_string()],
            vec![("j3".to_string(), "j4".to_string())],
            vec!["deploy2".to_string(), "deploy3".to_string()],
            "creator2".to_string(),
            43,
        )
    }

    #[tokio::test]
    async fn test_publish_and_consume() {
        // Create events publisher with capacity 2
        let events = F1r3flyEvents::new(Some(2));

        // Get stream first (before publishing)
        let mut stream = events.consume();

        // Publish first event
        let event1 = create_block_finalised_event();
        events
            .publish(event1.clone())
            .expect("Failed to publish event");

        // Consume first event
        let received = stream.next().await.expect("No event available");

        match received {
            F1r3flyEvent::BlockFinalised(BlockFinalised { block_hash }) => {
                assert_eq!(block_hash, "hash123");
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_circular_buffer_behavior() {
        // Create events publisher with capacity 2
        let events = F1r3flyEvents::new(Some(2));

        // Get stream first (before publishing)
        let mut stream = events.consume();

        // Publish three events
        events
            .publish(create_block_finalised_event())
            .expect("Failed to publish event");
        events
            .publish(create_block_created_event())
            .expect("Failed to publish event");
        events
            .publish(create_block_added_event())
            .expect("Failed to publish event");

        // Consume events from the stream
        let first_received = stream.next().await.expect("No event available");
        match first_received {
            F1r3flyEvent::BlockFinalised(BlockFinalised { block_hash }) => {
                assert_eq!(block_hash, "hash123");
            }
            _ => panic!("Wrong event type received"),
        }

        let second_received = stream.next().await.expect("No event available");
        match second_received {
            F1r3flyEvent::BlockCreated(BlockCreated { block_hash, .. }) => {
                assert_eq!(block_hash, "hash456");
            }
            _ => panic!("Wrong event type received"),
        }

        let third_received = stream.next().await.expect("No event available");
        match third_received {
            F1r3flyEvent::BlockAdded(BlockAdded { block_hash, .. }) => {
                assert_eq!(block_hash, "hash789");
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_default_capacity() {
        let events = F1r3flyEvents::default();

        // Get stream first (before publishing)
        let mut stream = events.consume();

        // Publish first event
        events
            .publish(create_block_finalised_event())
            .expect("Failed to publish event");

        // Publish second event - should not affect broadcast
        events
            .publish(create_block_created_event())
            .expect("Failed to publish event");

        // Consume events from the stream
        let first_received = stream.next().await.expect("No event available");
        match first_received {
            F1r3flyEvent::BlockFinalised(BlockFinalised { block_hash }) => {
                assert_eq!(block_hash, "hash123");
            }
            _ => panic!("Wrong event type received"),
        }

        let second_received = stream.next().await.expect("No event available");
        match second_received {
            F1r3flyEvent::BlockCreated(BlockCreated { block_hash, .. }) => {
                assert_eq!(block_hash, "hash456");
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_publish_and_consume() {
        let events = Arc::new(F1r3flyEvents::new(Some(10)));
        let mut handles = vec![];

        // Spawn multiple consumers first
        for _ in 0..3 {
            let events = Arc::clone(&events);
            let handle = tokio::spawn(async move {
                let mut stream = events.consume();
                let mut count = 0;
                
                // Add a timeout to avoid waiting forever
                while let Some(_) = tokio::time::timeout(Duration::from_millis(100), stream.next())
                    .await
                    .unwrap_or(None)
                {
                    count += 1;
                    if count >= 5 {
                        break;
                    }
                }
                count
            });
            handles.push(handle);
        }
        
        // Give consumers time to start up
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Then publish events
        for _ in 0..5 {
            let event = create_block_created_event();
            events.publish(event).expect("Failed to publish event");
            // Add a small delay between publishing events
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        // Wait for all consumers to finish
        let results = futures::future::join_all(handles).await;
        let total_events: usize = results.into_iter().map(|r| r.expect("Task failed")).sum();

        // Each consumer should get all 5 events
        assert_eq!(total_events, 15);
    }
}
