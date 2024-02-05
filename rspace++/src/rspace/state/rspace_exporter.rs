use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    shared::{
        key_value_store::KvStoreError,
        trie_exporter::{TrieExporter, TrieNode},
    },
};

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceExporter.scala
pub trait RSpaceExporter: TrieExporter + Send + Sync {
    // Get current root
    fn get_root(&self) -> Self::KeyHash;
}

pub struct RSpaceExporterInstance;

impl RSpaceExporterInstance {
    pub fn traverse_history<K, V, S>(
        start_path: Vec<(Blake3Hash, Option<u8>)>,
        skip: usize,
        take: usize,
        get_fromhistory: fn(&S, &K) -> Result<Option<V>, KvStoreError>,
    ) -> Vec<TrieNode<Blake3Hash>> {
        todo!()
    }
}
