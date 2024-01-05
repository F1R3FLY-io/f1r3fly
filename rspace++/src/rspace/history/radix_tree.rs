use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
pub enum Item {
    EmptyItem,
    Leaf(Leaf),
    NodePtr(NodePtr),
}

struct Leaf {
    prefix: Vec<u8>,
    value: Vec<u8>,
}

struct NodePtr {
    prefix: Vec<u8>,
    ptr: Vec<u8>,
}

pub type Node = Vec<Item>;

pub struct RadixTreeImpl {
    store: dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>,
}
