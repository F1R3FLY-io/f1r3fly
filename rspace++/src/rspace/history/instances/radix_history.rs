use crate::rspace::history::radix_tree::{Node, RadixTreeImpl};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStoreStruct;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistory {
    root_hash: blake3::Hash,
    root_node: Node,
    imple: Box<RadixTreeImpl>,
    store: KeyValueTypedStoreStruct,
}
