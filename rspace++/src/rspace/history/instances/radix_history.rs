use crate::rspace::history::history::History;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::radix_tree::{Node, RadixTreeImpl};
use crate::rspace::shared::key_value_store::{KeyValueStore, KeyValueStoreOps};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistoryInstances;

impl RadixHistoryInstances {
    pub fn create(
        root: blake3::Hash,
        store: impl KeyValueTypedStore<Bytes, Bytes> + Clone,
    ) -> RadixHistory<impl KeyValueTypedStore<Bytes, Bytes>> {
        let imple = RadixTreeImpl {
            store: store.clone(),
        };
        let node = imple.load_node(Bytes::copy_from_slice(root.as_bytes()), Some(true));

        RadixHistory {
            root_hash: root,
            root_node: node,
            imple,
            store,
        }
    }

    pub fn create_store(
        store: impl KeyValueStore + Clone,
    ) -> impl KeyValueTypedStore<Bytes, Bytes> + Clone {
        KeyValueStoreOps::to_typed_store::<Bytes, Bytes>(store)
    }
}

pub struct RadixHistory<T: KeyValueTypedStore<Bytes, Bytes>> {
    root_hash: blake3::Hash,
    root_node: Node,
    imple: RadixTreeImpl<T>,
    store: T,
}

impl<T: KeyValueTypedStore<Bytes, Bytes>> History for RadixHistory<T> {
    fn read(&self, key: Bytes) -> Option<Bytes> {
        todo!()
    }

    fn process(&self, actions: Vec<HistoryAction>) -> Box<dyn History> {
        todo!()
    }

    fn root(&self) -> blake3::Hash {
        todo!()
    }

    fn reset(&self, root: blake3::Hash) -> Box<dyn History> {
        todo!()
    }
}
