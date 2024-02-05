use super::key_value_store::KvStoreError;

// See shared/src/main/scala/coop/rchain/state/TrieExporter.scala
// Defines basic operation to traverse tries and convert to path indexed list
pub trait TrieExporter {
    // Type of the key to uniquely defines the trie / in RSpace this is the hash of the trie
    type KeyHash;
    // Type of the full path to the node
    // - it contains parent nodes with indexes and node itself at the end
    type NodePath;
    // Scala: type NodePath = Seq[(KeyHash, Option[Byte])]

    type Value;

    // Get trie nodes with offset from start path and number of nodes
    // - skipping nodes can be expensive as taking nodes
    fn get_nodes(
        &self,
        start_path: Self::NodePath,
        skip: usize,
        take: usize,
    ) -> Vec<TrieNode<Self::KeyHash>>;

    // Get history values / from branch nodes in the trie
    fn get_history_items(
        &self,
        keys: Vec<Self::KeyHash>,
    ) -> Result<Vec<(Self::KeyHash, Self::Value)>, KvStoreError>;

    // Get data values / from leaf nodes in the trie
    fn get_data_items(
        &self,
        keys: Vec<Self::KeyHash>,
    ) -> Result<Vec<(Self::KeyHash, Self::Value)>, KvStoreError>;
}

pub struct TrieNode<KeyHash> {
    hash: KeyHash,
    is_leaf: bool,
    path: Vec<(KeyHash, Option<u8>)>,
}
