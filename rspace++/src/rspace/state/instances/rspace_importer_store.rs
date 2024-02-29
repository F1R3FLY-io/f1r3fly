use std::sync::{Arc, Mutex};

use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    history::roots_store::{RootsStore, RootsStoreInstances},
    shared::{key_value_store::KeyValueStore, trie_importer::TrieImporter},
    state::rspace_importer::RSpaceImporter,
    ByteVector,
};

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceImporterStore.scala
pub struct RSpaceImporterStore;

impl RSpaceImporterStore {
    pub fn create(
        history_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
        value_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
        roots_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    ) -> impl RSpaceImporter<KeyHash = Blake3Hash, Value = ByteVector> {
        RSpaceImporterImpl {
            history_store,
            value_store,
            roots_store,
        }
    }
}

struct RSpaceImporterImpl {
    history_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    value_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    roots_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
}

impl RSpaceImporter for RSpaceImporterImpl {
    fn get_history_item(&self, hash: Self::KeyHash) -> Option<ByteVector> {
        let history_store_lock = self
            .history_store
            .lock()
            .expect("RSpace Importer Store: Failed to acquire lock on history_store");

        history_store_lock
            .get(vec![hash.bytes()])
            .expect("RSpace Importer: history store get failed")
            .into_iter()
            .next()
            .flatten()
    }
}

impl TrieImporter for RSpaceImporterImpl {
    type KeyHash = Blake3Hash;

    type Value = ByteVector;

    fn set_history_items(&self, data: Vec<(Self::KeyHash, Self::Value)>) -> () {
        let mut history_store_lock = self
            .history_store
            .lock()
            .expect("RSpace Importer Store: Failed to acquire lock on history_store");

        history_store_lock
            .put(
                data.iter()
                    .map(|pair| (pair.0.bytes(), pair.1.clone()))
                    .collect(),
            )
            .expect("Rspace Importer: failed to put in history store")
    }

    fn set_data_items(&self, data: Vec<(Self::KeyHash, Self::Value)>) -> () {
        let mut value_store_lock = self
            .value_store
            .lock()
            .expect("RSpace Importer Store: Failed to acquire lock on value_store");

        value_store_lock
            .put(
                data.iter()
                    .map(|pair| (pair.0.bytes(), pair.1.clone()))
                    .collect(),
            )
            .expect("Rspace Importer: failed to put in value store")
    }

    fn set_root(&self, key: &Self::KeyHash) -> () {
        let roots = RootsStoreInstances::roots_store(self.roots_store.clone());
        roots
            .record_root(key)
            .expect("Rspace Importer: failed to record root")
    }
}
