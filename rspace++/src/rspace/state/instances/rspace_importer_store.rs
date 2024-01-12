use bytes::Bytes;

use crate::rspace::{
    shared::{key_value_store::KeyValueStore, trie_importer::TrieImporter},
    state::rspace_importer::RSpaceImporter,
};

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceImporterStore.scala
pub struct RSpaceImporterStore;

impl RSpaceImporterStore {
    pub fn create<U: KeyValueStore>(
        history_store: U,
        value_store: U,
        roots_store: U,
    ) -> impl RSpaceImporter<KeyHash = blake3::Hash, Value = Bytes> {
        RSpaceImporterImpl {
            source_history_store: history_store,
            source_value_store: value_store,
            source_roots_store: roots_store,
        }
    }
}

struct RSpaceImporterImpl<U: KeyValueStore> {
    source_history_store: U,
    source_value_store: U,
    source_roots_store: U,
}

impl<U: KeyValueStore> RSpaceImporter for RSpaceImporterImpl<U> {
    fn get_history_item(&self, hash: Self::KeyHash) -> Option<Bytes> {
        todo!()
    }
}

impl<U: KeyValueStore> TrieImporter for RSpaceImporterImpl<U> {
    type KeyHash = blake3::Hash;

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
