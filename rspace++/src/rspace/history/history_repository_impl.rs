use super::cold_store::{ContinuationsLeaf, DataLeaf, JoinsLeaf};
use super::history_action::{DeleteAction, HistoryAction, InsertAction};
use super::history_reader::HistoryReader;
use super::history_repository::{PREFIX_DATUM, PREFIX_JOINS, PREFIX_KONT};
use super::instances::rspace_history_reader_impl::RSpaceHistoryReaderImpl;
use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::hashing::stable_hash_provider::{hash, hash_from_vec};
use crate::rspace::history::cold_store::PersistedData;
use crate::rspace::history::history::History;
use crate::rspace::history::history_repository::HistoryRepository;
use crate::rspace::history::root_repository::RootRepository;
use crate::rspace::hot_store_action::DeleteAction::{DeleteContinuations, DeleteData, DeleteJoins};
use crate::rspace::hot_store_action::HotStoreAction;
use crate::rspace::hot_store_action::InsertAction::{InsertContinuations, InsertData, InsertJoins};
use crate::rspace::hot_store_trie_action::{
    HotStoreTrieAction, TrieDeleteAction, TrieDeleteConsume, TrieDeleteJoins, TrieDeleteProduce,
    TrieInsertAction, TrieInsertConsume, TrieInsertJoins, TrieInsertProduce,
};
use crate::rspace::serializers::serializers::{
    encode_binary, encode_continuations, encode_datums, encode_joins,
};
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::state::rspace_exporter::RSpaceExporter;
use crate::rspace::state::rspace_importer::RSpaceImporter;
use bytes::Bytes;
use rayon::prelude::*;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepositoryImpl.scala
pub struct HistoryRepositoryImpl<C, P, A, K> {
    pub current_history: Arc<Mutex<Box<dyn History>>>,
    pub roots_repository: Arc<Mutex<RootRepository>>,
    pub leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<Blake3Hash, PersistedData>>>>,
    pub rspace_exporter: Arc<
        Mutex<
            Box<
                dyn RSpaceExporter<
                    KeyHash = blake3::Hash,
                    NodePath = Vec<(blake3::Hash, Option<u8>)>,
                    Value = Bytes,
                >,
            >,
        >,
    >,
    pub rspace_importer: Arc<Mutex<Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = Bytes>>>>,
    pub _marker: PhantomData<(C, P, A, K)>,
}

type ColdAction = (Blake3Hash, Option<PersistedData>);

impl<C, P, A, K> HistoryRepositoryImpl<C, P, A, K>
where
    C: Clone + Send + Sync + Serialize,
    P: Clone + Send + Sync + Serialize,
    A: Clone + Send + Sync + Serialize,
    K: Clone + Send + Sync + Serialize,
{
    fn measure(&self, actions: Vec<HotStoreAction<C, P, A, K>>) -> () {
        todo!()
    }

    fn compute_measure(&self, actions: Vec<HotStoreAction<C, P, A, K>>) -> Vec<String> {
        actions
            .into_par_iter()
            .map(|action| match action {
                HotStoreAction::Insert(InsertData(i)) => {
                    let key = hash(&i.channel).bytes();
                    let data = encode_datums(&i.data);
                    format!("{};insert-data;{};{}", hex::encode(key), data.len(), i.data.len())
                }
                HotStoreAction::Insert(InsertContinuations(i)) => {
                    let key = hash_from_vec(&i.channels).bytes();
                    let data = encode_continuations(&i.continuations);
                    format!(
                        "{};insert-continuation;{};{}",
                        hex::encode(key),
                        data.len(),
                        i.continuations.len()
                    )
                }
                HotStoreAction::Insert(InsertJoins(i)) => {
                    let key = hash(&i.channel).bytes();
                    let data = encode_joins(&i.joins);
                    format!("{};insert-join;{};", hex::encode(key), data.len())
                }
                HotStoreAction::Delete(DeleteData(d)) => {
                    let key = hash(&d.channel).bytes();
                    format!("{};delete-data;0", hex::encode(key))
                }
                HotStoreAction::Delete(DeleteContinuations(d)) => {
                    let key = hash_from_vec(&d.channels).bytes();
                    format!("{};delete-continuation;0", hex::encode(key))
                }
                HotStoreAction::Delete(DeleteJoins(d)) => {
                    let key = hash(&d.channel).bytes();
                    format!("{};delete-join;0", hex::encode(key))
                }
            })
            .collect()
    }

    fn calculate_storage_actions(
        &self,
        action: HotStoreTrieAction<C, P, A, K>,
    ) -> (ColdAction, HistoryAction) {
        match action {
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertProduce(i)) => {
                let data = encode_datums(&i.data);
                let data_leaf = DataLeaf { bytes: data };
                let data_leaf_encoded = bincode::serialize(&data_leaf)
                    .expect("History Repository Impl: Unable to serialize DataLeaf");
                let data_hash = Blake3Hash::new(&data_leaf_encoded);

                (
                    (data_hash.clone(), Some(PersistedData::Data(data_leaf))),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_DATUM, &i.hash.bytes()),
                        hash: data_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertConsume(i)) => {
                let data = encode_continuations(&i.continuations);
                let continuations_leaf = ContinuationsLeaf { bytes: data };
                let continuations_leaf_encoded = bincode::serialize(&continuations_leaf)
                    .expect("History Repository Impl: Unable to serialize ContinuationsLeaf");
                let continuations_hash = Blake3Hash::new(&continuations_leaf_encoded);

                (
                    (
                        continuations_hash.clone(),
                        Some(PersistedData::Continuations(continuations_leaf)),
                    ),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_KONT, &i.hash.bytes()),
                        hash: continuations_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertJoins(i)) => {
                let data = encode_joins(&i.joins);
                let joins_leaf = JoinsLeaf { bytes: data };
                let joins_leaf_encoded = bincode::serialize(&joins_leaf)
                    .expect("History Repository Impl: Unable to serialize JoinsLeaf");
                let joins_hash = Blake3Hash::new(&joins_leaf_encoded);

                (
                    (joins_hash.clone(), Some(PersistedData::Joins(joins_leaf))),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_JOINS, &i.hash.bytes()),
                        hash: joins_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertBinaryProduce(i)) => {
                let data = encode_binary(&i.data);
                let data_leaf = DataLeaf { bytes: data };
                let data_leaf_encoded = bincode::serialize(&data_leaf)
                    .expect("History Repository Impl: Unable to serialize DataLeaf");
                let data_hash = Blake3Hash::new(&data_leaf_encoded);

                (
                    (data_hash.clone(), Some(PersistedData::Data(data_leaf))),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_DATUM, &i.hash.bytes()),
                        hash: data_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertBinaryConsume(i)) => {
                let data = encode_binary(&i.continuations);
                let continuations_leaf = ContinuationsLeaf { bytes: data };
                let continuations_leaf_encoded = bincode::serialize(&continuations_leaf)
                    .expect("History Repository Impl: Unable to serialize ContinuationsLeaf");
                let continuations_hash = Blake3Hash::new(&continuations_leaf_encoded);

                (
                    (
                        continuations_hash.clone(),
                        Some(PersistedData::Continuations(continuations_leaf)),
                    ),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_KONT, &i.hash.bytes()),
                        hash: continuations_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieInsertAction(TrieInsertAction::TrieInsertBinaryJoins(i)) => {
                let data = encode_binary(&i.joins);
                let joins_leaf = JoinsLeaf { bytes: data };
                let joins_leaf_encoded = bincode::serialize(&joins_leaf)
                    .expect("History Repository Impl: Unable to serialize JoinsLeaf");
                let joins_hash = Blake3Hash::new(&joins_leaf_encoded);

                (
                    (joins_hash.clone(), Some(PersistedData::Joins(joins_leaf))),
                    HistoryAction::Insert(InsertAction {
                        key: prepend_bytes(PREFIX_JOINS, &i.hash.bytes()),
                        hash: joins_hash,
                    }),
                )
            }
            HotStoreTrieAction::TrieDeleteAction(TrieDeleteAction::TrieDeleteProduce(d)) => (
                (d.hash.clone(), None),
                HistoryAction::Delete(DeleteAction {
                    key: prepend_bytes(PREFIX_DATUM, &d.hash.bytes()),
                }),
            ),
            HotStoreTrieAction::TrieDeleteAction(TrieDeleteAction::TrieDeleteConsume(d)) => (
                (d.hash.clone(), None),
                HistoryAction::Delete(DeleteAction {
                    key: prepend_bytes(PREFIX_KONT, &d.hash.bytes()),
                }),
            ),
            HotStoreTrieAction::TrieDeleteAction(TrieDeleteAction::TrieDeleteJoins(d)) => (
                (d.hash.clone(), None),
                HistoryAction::Delete(DeleteAction {
                    key: prepend_bytes(PREFIX_JOINS, &d.hash.bytes()),
                }),
            ),
        }
    }

    fn transform(
        &self,
        hot_store_action: &HotStoreAction<C, P, A, K>,
    ) -> HotStoreTrieAction<C, P, A, K> {
        match hot_store_action {
            HotStoreAction::Insert(InsertData(i)) => {
                let key = hash(&i.channel);
                let trie_insert_action = TrieInsertAction::TrieInsertProduce(
                    TrieInsertProduce::new(key, i.data.clone()),
                );
                HotStoreTrieAction::TrieInsertAction(trie_insert_action)
            }
            HotStoreAction::Insert(InsertContinuations(i)) => {
                let key = hash_from_vec(&i.channels);
                let trie_insert_action = TrieInsertAction::TrieInsertConsume(
                    TrieInsertConsume::new(key, i.continuations.clone()),
                );
                HotStoreTrieAction::TrieInsertAction(trie_insert_action)
            }
            HotStoreAction::Insert(InsertJoins(i)) => {
                let key = hash(&i.channel);
                let trie_insert_action =
                    TrieInsertAction::TrieInsertJoins(TrieInsertJoins::new(key, i.joins.clone()));
                HotStoreTrieAction::TrieInsertAction(trie_insert_action)
            }
            HotStoreAction::Delete(DeleteData(d)) => {
                let key = hash(&d.channel);
                let trie_delete_action =
                    TrieDeleteAction::TrieDeleteProduce(TrieDeleteProduce::new(key));
                HotStoreTrieAction::TrieDeleteAction(trie_delete_action)
            }
            HotStoreAction::Delete(DeleteContinuations(d)) => {
                let key = hash_from_vec(&d.channels);
                let trie_delete_action =
                    TrieDeleteAction::TrieDeleteConsume(TrieDeleteConsume::new(key));
                HotStoreTrieAction::TrieDeleteAction(trie_delete_action)
            }
            HotStoreAction::Delete(DeleteJoins(d)) => {
                let key = hash(&d.channel);
                let trie_delete_action =
                    TrieDeleteAction::TrieDeleteJoins(TrieDeleteJoins::new(key));
                HotStoreTrieAction::TrieDeleteAction(trie_delete_action)
            }
        }
    }
}

impl<C, P, A, K> HistoryRepository<C, P, A, K> for HistoryRepositoryImpl<C, P, A, K>
where
    C: Clone + Send + Sync + Serialize + 'static,
    P: Clone + Send + Sync + Serialize + 'static,
    A: Clone + Send + Sync + Serialize + 'static,
    K: Clone + Send + Sync + Serialize + 'static,
{
    fn checkpoint(
        &self,
        actions: Vec<HotStoreAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        let trie_actions: Vec<_> = actions
            .par_iter()
            .map(|action| self.transform(action))
            .collect();

        todo!()
    }

    fn do_checkpoint(
        &self,
        actions: Vec<HotStoreTrieAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn reset(&self, root: blake3::Hash) -> Box<dyn HistoryRepository<C, P, A, K>> {
        let roots_lock = self
            .roots_repository
            .lock()
            .expect("History Repository Impl: Unable to acquire roots repository lock");
        let _ = roots_lock.validate_and_set_current_root(&root);

        let history_lock = self
            .current_history
            .lock()
            .expect("History Repository Impl: Unable to acquire history lock");
        let next = history_lock.reset(root);

        Box::new(HistoryRepositoryImpl {
            current_history: Arc::new(Mutex::new(next)),
            roots_repository: self.roots_repository.clone(),
            leaf_store: self.leaf_store.clone(),
            rspace_exporter: self.rspace_exporter.clone(),
            rspace_importer: self.rspace_importer.clone(),
            _marker: PhantomData,
        })
    }

    fn history(&self) -> Arc<Mutex<Box<dyn History>>> {
        self.current_history.clone()
    }

    fn exporter(
        &self,
    ) -> Arc<
        Mutex<
            Box<
                dyn RSpaceExporter<
                    KeyHash = blake3::Hash,
                    NodePath = Vec<(blake3::Hash, Option<u8>)>,
                    Value = bytes::Bytes,
                >,
            >,
        >,
    > {
        self.rspace_exporter.clone()
    }

    fn importer(
        &self,
    ) -> Arc<Mutex<Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = bytes::Bytes>>>> {
        self.rspace_importer.clone()
    }

    fn get_history_reader(
        &self,
        state_hash: blake3::Hash,
    ) -> Box<dyn HistoryReader<blake3::Hash, C, P, A, K>> {
        let history_lock = self
            .current_history
            .lock()
            .expect("History Repository Impl: Unable to acquire history lock");
        let history_repo = history_lock.reset(state_hash);
        Box::new(RSpaceHistoryReaderImpl::new(history_repo, self.leaf_store.clone()))
    }

    fn root(&self) -> blake3::Hash {
        let history_lock = self
            .current_history
            .lock()
            .expect("History Repository Impl: Unable to acquire history lock");
        history_lock.root()
    }
}

fn prepend_bytes(element: u8, _bytes: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(element);
    bytes.extend(_bytes.iter().cloned());
    bytes
}
