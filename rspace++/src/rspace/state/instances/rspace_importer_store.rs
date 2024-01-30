use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    shared::{key_value_store::KeyValueStore, trie_importer::TrieImporter},
    state::rspace_importer::RSpaceImporter,
};
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceImporterStore.scala
pub struct RSpaceImporterStore;

impl RSpaceImporterStore {
    pub fn create(
        history_store: Box<dyn KeyValueStore>,
        value_store: Box<dyn KeyValueStore>,
        roots_store: Box<dyn KeyValueStore>,
    ) -> impl RSpaceImporter<KeyHash = Blake3Hash, Value = Bytes> {
        RSpaceImporterImpl {
            source_history_store: history_store,
            source_value_store: value_store,
            source_roots_store: roots_store,
        }
    }
}

struct RSpaceImporterImpl {
    source_history_store: Box<dyn KeyValueStore>,
    source_value_store: Box<dyn KeyValueStore>,
    source_roots_store: Box<dyn KeyValueStore>,
}

impl RSpaceImporter for RSpaceImporterImpl {
    fn get_history_item(&self, hash: Self::KeyHash) -> Option<Bytes> {
        todo!()
    }
}

impl TrieImporter for RSpaceImporterImpl {
    type KeyHash = Blake3Hash;

    type Value = Bytes;

    fn set_history_items(
        &self,
        data: Vec<(Self::KeyHash, Self::Value)>,
        to_buffer: fn(Self::Value) -> &'static [u8],
    ) -> () {
        todo!()
    }

    fn set_data_items(
        &self,
        data: Vec<(Self::KeyHash, Self::Value)>,
        to_buffer: fn(Self::Value) -> &'static [u8],
    ) -> () {
        todo!()
    }

    fn set_root(&self, key: Self::KeyHash) -> () {
        todo!()
    }
}
