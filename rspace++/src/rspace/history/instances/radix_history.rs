use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::history::history::History;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::radix_tree::{hash_node, EmptyNode, Node, RadixTreeImpl};
use crate::rspace::shared::key_value_store::{KeyValueStore, KeyValueStoreOps};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;
use std::sync::Arc;
use std::sync::Mutex;

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RadixHistory.scala
pub struct RadixHistory {
    root_hash: Blake3Hash,
    root_node: Node,
    imple: RadixTreeImpl,
    store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>>>,
}

impl RadixHistory {
    pub fn create(
        root: Blake3Hash,
        store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>>>,
    ) -> RadixHistory {
        let imple = RadixTreeImpl::new(store.clone());
        let node = imple.load_node(root.bytes(), Some(true));

        RadixHistory {
            root_hash: root,
            root_node: node,
            imple,
            store,
        }
    }

    pub fn create_store(
        store: Box<dyn KeyValueStore>,
    ) -> Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>>> {
        Arc::new(Mutex::new(Box::new(KeyValueStoreOps::to_typed_store::<Vec<u8>, Vec<u8>>(store))))
    }

    pub fn empty_root_hash() -> Blake3Hash {
        let hash_bytes = hash_node(&EmptyNode::new().node).0;
        let hash_array: [u8; 32] = match hash_bytes.try_into() {
            Ok(array) => array,
            Err(_) => panic!("Radix_History: Expected a Blake3 hash of length 32"),
        };
        let hash = Blake3Hash::new(&hash_array);
        hash
    }
}

impl History for RadixHistory {
    fn read(&self, key: Bytes) -> Option<Bytes> {
        todo!()
    }

    fn process(&self, actions: Vec<HistoryAction>) -> Box<dyn History> {
        todo!()
    }

    fn root(&self) -> Blake3Hash {
        self.root_hash.clone()
    }

    fn reset(&self, root: &Blake3Hash) -> Box<dyn History> {
        let imple = RadixTreeImpl::new(self.store.clone());
        let node = imple.load_node(root.bytes(), Some(true));

        Box::new(RadixHistory {
            root_hash: root.clone(),
            root_node: node,
            imple,
            store: self.store.clone(),
        })
    }
}
