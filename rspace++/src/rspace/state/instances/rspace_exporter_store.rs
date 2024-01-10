use crate::rspace::shared::trie_exporter::{TrieExporter, TrieNode};
use crate::rspace::{
    shared::key_value_store::KeyValueStore, state::rspace_exporter::RSpaceExporter,
};
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceExporterStore.scala
pub struct RSpaceExporterStore;

impl RSpaceExporterStore {
    pub fn create<U: KeyValueStore>(
        history_store: U,
        value_store: U,
        roots_store: U,
    ) -> impl RSpaceExporter {
        RSpaceExporterImpl {
            source_history_store: history_store,
            source_value_store: value_store,
            source_roots_store: roots_store,
        }
    }
}

struct RSpaceExporterImpl<U: KeyValueStore> {
    source_history_store: U,
    source_value_store: U,
    source_roots_store: U,
}

impl<U: KeyValueStore> RSpaceExporter for RSpaceExporterImpl<U> {
    fn get_root(&self) -> blake3::Hash {
        todo!()
    }
}

impl<U: KeyValueStore> TrieExporter for RSpaceExporterImpl<U> {
    type KeyHash = blake3::Hash;

    type NodePath = Vec<(Self::KeyHash, Option<u8>)>;

    type Value = Bytes;

    fn get_nodes(
        &self,
        start_path: Self::NodePath,
        skip: usize,
        take: usize,
    ) -> Vec<TrieNode<Self::KeyHash>> {
        todo!()
    }

    fn get_history_items(
        &self,
        keys: Vec<blake3::Hash>,
        from_buffer: fn(&[u8]) -> Self::Value,
    ) -> Vec<(Self::KeyHash, Self::Value)> {
        todo!()
    }

    fn get_data_items(
        &self,
        keys: Vec<blake3::Hash>,
        from_buffer: fn(&[u8]) -> Self::Value,
    ) -> Vec<(Self::KeyHash, Self::Value)> {
        todo!()
    }
}
