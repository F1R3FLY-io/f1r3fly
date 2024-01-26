use crate::rspace::shared::trie_exporter::TrieExporter;

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceExporter.scala
pub trait RSpaceExporter: TrieExporter + Send + Sync {
    // Get current root
    fn get_root(&self) -> Self::KeyHash;
}
