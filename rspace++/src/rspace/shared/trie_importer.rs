// See shared/src/main/scala/coop/rchain/state/TrieImporter.scala
pub trait TrieImporter {
    // Type of the key to uniquely defines the trie / in RSpace this is the hash of the trie
    type KeyHash;

    type Value;

    // Set history values / branch nodes in the trie
    fn set_history_items(&self, data: Vec<(Self::KeyHash, Self::Value)>) -> ();

    // Set data values / leaf nodes in the trie
    fn set_data_items(&self, data: Vec<(Self::KeyHash, Self::Value)>) -> ();

    // Set current root hash
    fn set_root(&self, key: &Self::KeyHash) -> ();
}
