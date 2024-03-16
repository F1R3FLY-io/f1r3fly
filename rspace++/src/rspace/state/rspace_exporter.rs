use crate::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    history::{
        radix_tree::{sequential_export, ExportData, ExportDataSettings},
        roots_store::RootError,
    },
    shared::trie_exporter::{KeyHash, TrieExporter, TrieNode},
    ByteVector,
};
use std::sync::Arc;

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceExporter.scala
pub trait RSpaceExporter: TrieExporter + Send + Sync {
    // Get current root
    fn get_root(&self) -> Result<KeyHash, RootError>;
}

pub struct RSpaceExporterInstance;

impl RSpaceExporterInstance {
    pub fn traverse_history(
        start_path: Vec<(Blake2b256Hash, Option<u8>)>,
        skip: usize,
        take: usize,
        get_from_history: Arc<dyn Fn(&ByteVector) -> Option<ByteVector>>,
    ) -> Vec<TrieNode<Blake2b256Hash>> {
        let settings = ExportDataSettings {
            flag_node_prefixes: false,
            flag_node_keys: true,
            flag_node_values: false,
            flag_leaf_prefixes: false,
            flag_leaf_values: true,
        };

        fn create_last_prefix(prefix_vec: Vec<Blake2b256Hash>) -> Option<Vec<u8>> {
            if prefix_vec.is_empty() {
                None // Start from root
            } else {
                // Max prefix length = 127 bytes.
                // Prefix coded 5 Blake256 elements (0 - size, 1..4 - value of prefix).
                assert!(prefix_vec.len() >= 5, "RSpace Exporter: Invalid path during export");

                let (size_prefix, vec): (usize, Vec<Blake2b256Hash>) = (
                    prefix_vec[0].bytes()[0] as usize & 0xff,
                    prefix_vec.iter().map(|path| path.clone()).collect(),
                );

                let mut prefix128 = vec[0].bytes();
                prefix128.extend(vec[1].bytes());
                prefix128.extend(vec[2].bytes());
                prefix128.extend(vec[3].bytes());

                Some(prefix128[..size_prefix].to_vec())
            }
        }

        fn construct_nodes(
            leaf_keys: Vec<Vec<u8>>,
            node_keys: Vec<Vec<u8>>,
        ) -> Vec<TrieNode<Blake2b256Hash>> {
            let mut data_keys: Vec<TrieNode<Blake2b256Hash>> = leaf_keys
                .iter()
                .map(|key| {
                    let hash = Blake2b256Hash::new(&key);
                    TrieNode::new(hash, true, Vec::new())
                })
                .collect();

            let history_keys: Vec<TrieNode<Blake2b256Hash>> = node_keys
                .iter()
                .map(|key| {
                    let hash = Blake2b256Hash::new(&key);
                    TrieNode::new(hash, false, Vec::new())
                })
                .collect();

            data_keys.extend(history_keys);
            data_keys
        }

        fn construct_last_path(
            last_prefix: Vec<u8>,
            root_hash: Blake2b256Hash,
        ) -> Vec<(Blake2b256Hash, Option<u8>)> {
            let prefix_size = last_prefix.len();
            let mut size_array: Vec<u8> = vec![prefix_size as u8];
            size_array.extend(std::iter::repeat(0x00).take(31));

            let prefix_zeros: Vec<u8> = std::iter::repeat(0x00).take(128 - prefix_size).collect();
            let mut prefix128_array = last_prefix.to_vec();
            prefix128_array.extend(prefix_zeros);

            let prefix_blake0 = Blake2b256Hash::new(&size_array);
            let prefix_blake1 = Blake2b256Hash::new(&prefix128_array[0..32]);
            let prefix_blake2 = Blake2b256Hash::new(&prefix128_array[32..64]);
            let prefix_blake3 = Blake2b256Hash::new(&prefix128_array[64..96]);
            let prefix_blake4 = Blake2b256Hash::new(&prefix128_array[96..128]);

            vec![
                (root_hash, None),
                (prefix_blake0, None),
                (prefix_blake1, None),
                (prefix_blake2, None),
                (prefix_blake3, None),
                (prefix_blake4, None),
            ]
        }

        fn construct_last_node(
            last_key: Vec<u8>,
            last_path: Vec<(Blake2b256Hash, Option<u8>)>,
        ) -> Vec<TrieNode<Blake2b256Hash>> {
            let hash = Blake2b256Hash::new(&last_key);
            vec![TrieNode::new(hash, false, last_path)]
        }

        if start_path.is_empty() {
            Vec::new()
        } else {
            let path_vec: Vec<Blake2b256Hash> = start_path.iter().map(|path| path.0.clone()).collect();
            let (root_hash, prefix_vec) =
                (path_vec.first().unwrap(), path_vec.split_first().map(|(_, tail)| tail).unwrap());

            let last_prefix = create_last_prefix(prefix_vec.to_vec());

            let exp_res = sequential_export(
                root_hash.bytes(),
                last_prefix,
                skip,
                take,
                get_from_history,
                settings,
            )
            .expect("RSpace Exporter: Failed to export");

            let (data, new_last_prefix_opt) = exp_res;
            let ExportData {
                node_prefixes: _,
                node_keys,
                node_values: _,
                leaf_prefixes: _,
                leaf_values: leaf_keys,
            } = data;

            let nodes = construct_nodes(leaf_keys, node_keys.clone());
            let mut nodes_without_last = if nodes.len() > 0 {
                nodes[..nodes.len() - 1].to_vec()
            } else {
                Vec::new()
            };

            let last_path = match new_last_prefix_opt {
                Some(bytes) => construct_last_path(bytes, root_hash.clone()),
                None => Vec::new(),
            };

            let last_history_node = match node_keys.last() {
                Some(bytes) => construct_last_node(bytes.clone(), last_path),
                None => Vec::new(),
            };

            nodes_without_last.extend(last_history_node);
            nodes_without_last
        }
    }
}
