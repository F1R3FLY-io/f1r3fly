use crate::rspace::shared::key_value_store::KvStoreError;
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

use super::history_action::HistoryAction;

// See rspace/src/main/scala/coop/rchain/rspace/history/RadixTree.scala
#[derive(Clone, Serialize, Deserialize)]
pub enum Item {
    EmptyItem,

    Leaf { prefix: Vec<u8>, value: Vec<u8> },

    NodePtr { prefix: Vec<u8>, ptr: Vec<u8> },
}

pub type Node = Vec<Item>;

const NUM_ITEMS: usize = 256;

pub struct EmptyNode {
    pub node: Node,
}

impl EmptyNode {
    pub fn new() -> EmptyNode {
        EmptyNode {
            node: vec![Item::EmptyItem; NUM_ITEMS],
        }
    }
}

pub fn hash_node(node: &Node) -> (Vec<u8>, Vec<u8>) {
    let bytes = bincode::serialize(node).unwrap();
    let hash = blake3::hash(&bytes);
    (hash.as_bytes().to_vec(), bytes)
}

/**
 * Data returned after export
 *
 * @param nodePrefixes Node prefixes
 * @param nodeKeys Node KVDB keys
 * @param nodeValues Node KVDB values
 * @param leafPrefixes Leaf prefixes
 * @param leafValues Leaf values (it's pointer for data in datastore)
 */
pub struct ExportData {
    pub node_prefixes: Vec<Vec<u8>>,
    pub node_keys: Vec<Vec<u8>>,
    pub node_values: Vec<Vec<u8>>,
    pub leaf_prefixes: Vec<Vec<u8>>,
    pub leaf_values: Vec<Vec<u8>>,
}

/**
 * Settings for [[ExportData]]
 *
 * If false - data will not be exported.
 */
pub struct ExportDataSettings {
    pub flag_node_prefixes: bool,
    pub flag_node_keys: bool,
    pub flag_node_values: bool,
    pub flag_leaf_prefixes: bool,
    pub flag_leaf_values: bool,
}

/**
 * Sequential export algorithm
 *
 * @param rootHash Root node hash, starting point
 * @param lastPrefix Describes the path of root to last processed element (if None - start from root)
 * @param skipSize Describes how many elements to skip
 * @param takeSize Describes how many elements to take
 * @param getNodeDataFromStore Function to get data from storage
 * @param settings [[ExportDataSettings]]
 *
 * @return
 * Return the data and prefix of the last processed item.
 * If all bonds in the tree are processed, returns None as prefix.
 * {{{
 * prefix - Prefix that describes the path of root to node
 * decoded - Deserialized data (from parsing)
 * lastItemIndex - Last processed item index
 * }}}
 */
pub fn sequential_export<K, V, S>(
    root_hash: Vec<u8>,
    last_prefix: Option<Vec<u8>>,
    skip_size: usize,
    take_size: usize,
    get_node_data_from_store: fn(&S, &K) -> Result<Option<V>, KvStoreError>,
    settings: ExportDataSettings,
) -> (ExportData, Option<Vec<u8>>) {
    todo!()
}

/**
 * Radix Tree implementation
 */
#[derive(Clone)]
pub struct RadixTreeImpl {
    pub store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>>>,
    /**
     * Cache for storing read and decoded nodes.
     *
     * Cache stores kv-pairs (hash, node).
     * Where hash - Blake2b256Hash of serializing nodes data,
     *       node - deserialized data of this node.
     */
    pub cache_r: DashMap<Vec<u8>, Node>,
    /**
     * Cache for storing serializing nodes. For subsequent unloading in KVDB
     *
     * Cache stores kv-pairs (hash, bytes).
     * Where hash -  Blake2b256Hash of bytes,
     *       bytes - serializing data of nodes.
     */
    pub cache_w: DashMap<Vec<u8>, Vec<u8>>,
}

impl RadixTreeImpl {
    pub fn new(store: Arc<Mutex<Box<dyn KeyValueTypedStore<Vec<u8>, Vec<u8>>>>>) -> Self {
        RadixTreeImpl {
            store,
            cache_r: DashMap::new(),
            cache_w: DashMap::new(),
        }
    }

    /**
     * Load and decode serializing data from KVDB.
     */
    fn load_node_from_store(&self, node_ptr: &Vec<u8>) -> Option<Node> {
        let store_lock = self
            .store
            .lock()
            .expect("Radix Tree: Failed to acquire lock on store");
        let get_result = store_lock.get_one(&node_ptr);

        match get_result {
            Ok(bytes_opt) => match bytes_opt {
                Some(bytes) => {
                    let deserialized = bincode::deserialize(&bytes)
                        .expect("Radix Tree: Failed to deserialize node bytes");
                    deserialized
                }
                None => None,
            },
            Err(err) => {
                println!("Radix Tree: {}", err);
                None
            }
        }
    }

    /**
     * Load one node from [[cacheR]].
     *
     * If there is no such record in cache - load and decode from KVDB, then save to cacheR.
     * If there is no such record in KVDB - execute assert (if set noAssert flag - return emptyNode).
     */
    pub fn load_node(&self, node_ptr: Vec<u8>, no_assert: Option<bool>) -> Node {
        let no_assert = no_assert.unwrap_or(false);

        let error_msg = |node_ptr: &[u8]| {
            assert!(
                no_assert,
                "Missing node in database. ptr={:?}",
                node_ptr
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<_>>()
            );
        };

        let cache_miss = |node_ptr: Vec<u8>| {
            let store_node_opt = self.load_node_from_store(&node_ptr);

            let node_opt = store_node_opt
                .map(|node| self.cache_r.insert(node_ptr.clone(), node))
                .unwrap_or_else(|| {
                    error_msg(&node_ptr);
                    None
                });

            match node_opt {
                Some(node) => node,
                None => EmptyNode::new().node,
            }
        };

        let cache_node_opt = self.cache_r.get(&node_ptr);
        match cache_node_opt {
            Some(node) => node.to_vec(),
            None => cache_miss(node_ptr),
        }
    }

    /**
     * Clear [[cacheR]] (cache for storing read nodes).
     */
    pub fn clear_read_cache(&self) -> () {
        self.cache_r.clear()
    }

    /**
     * Serializing and hashing one [[Node]].
     *
     * Serializing data load in [[cacheW]].
     * If detected collision with older cache data - executing assert
     */
    pub fn save_node(&self, node: &Node) -> Vec<u8> {
        todo!()
    }

    /**
     * Save all [[cacheW]] to [[store]]
     *
     * If detected collision with older KVDB data - execute Exception
     */
    pub fn commit(&self) -> () {
        todo!()
    }

    /**
     * Clear [[cacheW]] (cache for storing data to write in KVDB).
     */
    pub fn clear_write_cache(&self) -> () {
        self.cache_w.clear()
    }

    /**
     * Read leaf data with prefix. If data not found, returned [[None]]
     */
    pub fn read(&self, start_node: Node, start_prefix: Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }

    /**
     * Parallel processing of [[HistoryAction]]s in this part of tree (start from curNode).
     *
     * New data load to [[cacheW]].
     * @return Updated curNode. if no action was taken - return [[None]].
     */
    pub fn make_actions(&self, curr_node: Node, actions: Vec<HistoryAction>) -> Option<Node> {
        todo!()
    }
}
