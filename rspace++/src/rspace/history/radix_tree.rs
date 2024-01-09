use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
pub enum Item {
    EmptyItem,
    Leaf(Leaf),
    NodePtr(NodePtr),
}

struct Leaf {
    prefix: Bytes,
    value: Bytes,
}

struct NodePtr {
    prefix: Bytes,
    ptr: Bytes,
}

pub type Node = Vec<Item>;

pub struct RadixTreeImpl<T: KeyValueTypedStore<Bytes, Bytes>> {
    pub store: T,
}

impl<T: KeyValueTypedStore<Bytes, Bytes>> RadixTreeImpl<T> {
    pub fn load_node(&self, node_ptr: Bytes, no_assert: Option<bool>) -> Node {
        let no_assert = no_assert.unwrap_or(false);
        todo!()
    }
}
