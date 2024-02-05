use crate::rspace::{shared::trie_importer::TrieImporter, ByteVector};

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceImporter.scala
pub trait RSpaceImporter: TrieImporter + Send + Sync {
    fn get_history_item(&self, hash: Self::KeyHash) -> Option<ByteVector>;
}
