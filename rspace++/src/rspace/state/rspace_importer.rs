// use rayon::{
//     iter::{IntoParallelRefIterator, ParallelIterator},
//     ThreadPoolBuilder,
// };
// use std::{collections::HashMap, sync::Arc};

use crate::rspace::{
    // hashing::blake2b256_hash::Blake2b256Hash,
    shared::{trie_exporter::KeyHash, trie_importer::TrieImporter},
    // state::rspace_exporter::RSpaceExporterInstance,
    ByteVector,
};

// See rspace/src/main/scala/coop/rchain/rspace/state/RSpaceImporter.scala
pub trait RSpaceImporter: TrieImporter + Send + Sync {
    fn get_history_item(&self, hash: KeyHash) -> Option<ByteVector>;
}

// struct RSpaceImporterInstance;

// impl RSpaceImporterInstance {
//     async fn validate_state_items(
//         history_items: Vec<(Blake2b256Hash, ByteVector)>,
//         data_items: Vec<(Blake2b256Hash, ByteVector)>,
//         start_path: Vec<(Blake2b256Hash, Option<u8>)>,
//         chunk_size: usize,
//         skip: usize,
//         get_from_history: fn(Blake2b256Hash) -> Option<ByteVector>,
//     ) -> () {
//         let received_history_size = history_items.len();
//         let is_end = || received_history_size < chunk_size;

//         // Validate history items size
//         let validate_history_size = || {
//             let size_is_valid = || received_history_size == chunk_size || is_end();
//             if !size_is_valid() {
//                 panic!("RSpace Importer: Input size of history items is not valid. Expected chunk size {}, received {}.", chunk_size, received_history_size)
//             }
//         };

//         // Validate history hashes
//         let get_and_validate_history_items = || {
//             let mut validated_items = Vec::new();
//             for (hash, trie_bytes) in history_items.clone() {
//                 let trie_hash = Blake2b256Hash::new(&trie_bytes);
//                 if hash == trie_hash {
//                     validated_items.push((trie_hash.bytes(), trie_bytes));
//                 } else {
//                     panic!("RSpace Importer: Trie hash does not match decoded trie, key: {}, decoded: {}", hex::encode(hash.bytes()), hex::encode(trie_hash.bytes()));
//                 }
//             }
//             validated_items
//         };

//         // Validate data hashes
//         let validate_data_items_hashes = async {
//             let pool = ThreadPoolBuilder::new().num_threads(64).build().unwrap();
//             pool.install(|| {
//                 data_items.par_iter().try_for_each(|(hash, value_bytes)| {
//                     let data_hash = Blake2b256Hash::new(value_bytes);
//                     if *hash != data_hash {
//                         Err(format!(
//                             "Data hash does not match decoded data, key: {}, decoded: {}.",
//                             hex::encode(hash.bytes()),
//                             hex::encode(data_hash.bytes())
//                         ))
//                     } else {
//                         Ok(())
//                     }
//                 })
//             })
//         };

//         fn node_hash_not_found_error(h: Blake2b256Hash) {
//             panic!("RSpace Importer: Trie hash not found in received items or in history store, hash: {}", hex::encode(h.bytes()))
//         }

//         let get_node = |st: HashMap<ByteVector, ByteVector>| {
//             Arc::new(move |hash: &ByteVector| {
//                 match st.get(hash) {
//                 Some(value) => Some(value.clone()),
//                 None => match get_from_history(Blake2b256Hash::new(hash)) {
//                     Some(bytes) => Some(bytes),
//                     None => panic!("RSpace Importer: Trie hash not found in received items or in history store, hash: {}", hex::encode(Blake2b256Hash::new(&hash).bytes())),
//                 },
//             }
//             })
//         };

//         // Validate chunk size.
//         validate_history_size();

//         // Validate tries from received history items.
//         let trie_map: HashMap<ByteVector, ByteVector> =
//             get_and_validate_history_items().into_iter().collect();

//         let get_node_function = get_node(trie_map);
//         // Traverse trie and extract nodes / the same as in export. Nodes must match hashed keys.
//         let nodes = RSpaceExporterInstance::traverse_history(
//             start_path,
//             skip,
//             chunk_size,
//             get_node_function,
//         );

//         // Extract history and data keys.
//         let (leafs, non_leafs): (Vec<_>, Vec<_>) = nodes.into_iter().partition(|node| node.is_leaf);
//         let history_keys: Vec<Blake2b256Hash> = non_leafs
//             .into_iter()
//             .map(|non_leaf| non_leaf.hash)
//             .collect();
//         let data_keys: Vec<Blake2b256Hash> = leafs.into_iter().map(|leaf| leaf.hash).collect();

//         // Validate keys / cryptographic proof that store chunk is not corrupted or modified.
//         let history_item_keys: Vec<Blake2b256Hash> =
//             history_items.into_iter().map(|item| item.0).collect();
//         if !(history_item_keys == history_keys) {
//             panic!("RSpace Importer: History items are corrupted")
//         }

//         let data_item_keys: Vec<Blake2b256Hash> =
//             data_items.clone().into_iter().map(|item| item.0).collect();
//         if !(data_item_keys == data_keys) {
//             panic!("RSpace Importer: Data items are corrupted")
//         }

//         validate_data_items_hashes
//             .await
//             .expect("RSpace Importer: Unable to validate data items hashes");
//     }
// }
