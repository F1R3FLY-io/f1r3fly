use super::trie_exporter::{KeyHash, Value};

// See shared/src/main/scala/coop/rchain/state/TrieImporter.scala
pub trait TrieImporter {
    // Set history values / branch nodes in the trie
    fn set_history_items(&self, data: Vec<(KeyHash, Value)>) -> ();

    // Set data values / leaf nodes in the trie
    fn set_data_items(&self, data: Vec<(KeyHash, Value)>) -> ();

    // Set current root hash
    fn set_root(&self, key: &KeyHash) -> ();
}
