use crate::rspace::history::radix_tree::{Node, RadixTreeImpl};
use crate::rspace::shared::key_value_store::{KeyValueStore, KeyValueStoreOps};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistoryInstance;

impl RadixHistoryInstance {
    pub fn create_store<U: KeyValueStore>(store: U) -> impl KeyValueTypedStore<Bytes, Bytes> {
        KeyValueStoreOps::to_typed_store(store)
    }
}

pub struct RadixHistory<T: KeyValueTypedStore<Bytes, Bytes>> {
    root_hash: blake3::Hash,
    root_node: Node,
    imple: RadixTreeImpl<T>,
    store: T,
}

impl<T: KeyValueTypedStore<Bytes, Bytes>> RadixHistory<T> {
    pub fn create(root: blake3::Hash, store: T) -> RadixHistory<T> {
        let imple = RadixTreeImpl { store };
        let node = imple.load_node(Bytes::copy_from_slice(root.as_bytes()), Some(true));

        RadixHistory {
            root_hash: root,
            root_node: node,
            imple,
            store,
        }
    }
}
