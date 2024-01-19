use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
#[derive(Clone, Serialize)]
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
    pub store: Box<dyn KeyValueTypedStore<Bytes, Bytes>>,
}

impl RadixTreeImpl {
    pub fn load_node(&self, node_ptr: Bytes, no_assert: Option<bool>) -> Node {
        let no_assert = no_assert.unwrap_or(false);
        todo!()
    }
}
