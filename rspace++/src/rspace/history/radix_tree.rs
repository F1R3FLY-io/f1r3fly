use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
#[derive(Clone, Serialize, Deserialize)]
pub enum Item {
    EmptyItem,

    Leaf { prefix: Vec<u8>, value: Vec<u8> },

    NodePtr { prefix: Vec<u8>, ptr: Vec<u8> },
}

pub type Node = Vec<Item>;

const NUM_ITEMS: usize = 256;

pub struct EmptyNode {
    pub node: Node,
}

impl EmptyNode {
    pub fn new() -> EmptyNode {
        EmptyNode {
            node: vec![Item::EmptyItem; NUM_ITEMS],
        }
    }
}

pub fn hash_node(node: &Node) -> (Vec<u8>, Vec<u8>) {
    let bytes = bincode::serialize(node).unwrap();
    let hash = blake3::hash(&bytes);
    (hash.as_bytes().to_vec(), bytes)
}

pub struct RadixTreeImpl {
    pub store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>> + Sync>>>,
    pub cache: DashMap<Vec<u8>, Node>,
}

impl RadixTreeImpl {
    async fn load_node_from_store(&self, node_ptr: &Vec<u8>) -> Option<Node> {
        let store_lock = self
            .store
            .lock()
            .expect("Radix Tree: Failed to acquire lock on store");
        let get_result = store_lock.get_one(&node_ptr).await;

        match get_result {
            Ok(bytes) => {
                let deserialized = bincode::deserialize(&bytes)
                    .expect("Radix Tree: Failed to deserialize node bytes");
                deserialized
            }
            Err(err) => {
                println!("Radix Tree: {}", err);
                None
            }
        }
    }

    pub async fn load_node(&self, node_ptr: Vec<u8>, no_assert: Option<bool>) -> Node {
        let no_assert = no_assert.unwrap_or(false);

        let error_msg = |node_ptr: &[u8]| {
            assert!(
                no_assert,
                "Missing node in database. ptr={:?}",
                node_ptr
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<_>>()
            );
        };

        let cache_miss = |node_ptr: Vec<u8>| async move {
            let store_node_opt = self.load_node_from_store(&node_ptr).await;

            let node_opt = store_node_opt
                .map(|node| self.cache.insert(node_ptr.clone(), node))
                .unwrap_or_else(|| {
                    error_msg(&node_ptr);
                    None
                });

            match node_opt {
                Some(node) => node,
                None => EmptyNode::new().node,
            }
        };

        let cache_node_opt = self.cache.get(&node_ptr);
        match cache_node_opt {
            Some(node) => node.to_vec(),
            None => cache_miss(node_ptr).await,
        }
    }
}
