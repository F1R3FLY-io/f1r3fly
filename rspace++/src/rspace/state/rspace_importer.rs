use crate::rspace::shared::trie_importer::TrieImporter;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceImporter.scala
pub trait RSpaceImporter: TrieImporter {
    fn get_history_item(&self, hash: Self::KeyHash) -> Option<Bytes>;
}
