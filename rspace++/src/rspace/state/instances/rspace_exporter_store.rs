use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::shared::trie_exporter::{TrieExporter, TrieNode};
use crate::rspace::{
    shared::key_value_store::KeyValueStore, state::rspace_exporter::RSpaceExporter,
};
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/state/instances/RSpaceExporterStore.scala
pub struct RSpaceExporterStore;

impl RSpaceExporterStore {
    pub fn create(
        history_store: Box<dyn KeyValueStore>,
        value_store: Box<dyn KeyValueStore>,
        roots_store: Box<dyn KeyValueStore>,
    ) -> impl RSpaceExporter<KeyHash = Blake3Hash, NodePath = Vec<(Blake3Hash, Option<u8>)>, Value = Bytes>
    {
        RSpaceExporterImpl {
            source_history_store: history_store,
            source_value_store: value_store,
            source_roots_store: roots_store,
        }
    }
}

struct RSpaceExporterImpl {
    source_history_store: Box<dyn KeyValueStore>,
    source_value_store: Box<dyn KeyValueStore>,
    source_roots_store: Box<dyn KeyValueStore>,
}

impl RSpaceExporter for RSpaceExporterImpl {
    fn get_root(&self) -> Blake3Hash {
        todo!()
    }
}

impl TrieExporter for RSpaceExporterImpl {
    type KeyHash = Blake3Hash;

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
        keys: Vec<Blake3Hash>,
        from_buffer: fn(&[u8]) -> Self::Value,
    ) -> Vec<(Self::KeyHash, Self::Value)> {
        todo!()
    }

    fn get_data_items(
        &self,
        keys: Vec<Blake3Hash>,
        from_buffer: fn(&[u8]) -> Self::Value,
    ) -> Vec<(Self::KeyHash, Self::Value)> {
        todo!()
    }
}
