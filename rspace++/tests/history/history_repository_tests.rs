// See rspace/src/test/scala/coop/rchain/rspace/history/HistoryRepositorySpec.scala
#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeSet, HashSet},
        sync::{Arc, Mutex},
    };

    use rspace_plus_plus::rspace::{
        event::{Consume, Produce},
        hashing::blake3_hash::Blake3Hash,
        history::{
            history::HistoryInstances,
            history_repository::HistoryRepository,
            history_repository_impl::HistoryRepositoryImpl,
            instances::radix_history::RadixHistory,
            root_repository::RootRepository,
            roots_store::{RootError, RootsStore},
        },
        hot_store_action::{
            HotStoreAction, InsertAction, InsertContinuations, InsertData, InsertJoins,
        },
        internal::{Datum, WaitingContinuation},
        shared::{
            key_value_store::{KeyValueStore, KvStoreError},
            mem_key_value_store::InMemoryKeyValueStore,
            trie_exporter::{KeyHash, NodePath, TrieExporter, TrieNode, Value},
            trie_importer::TrieImporter,
        },
        state::{rspace_exporter::RSpaceExporter, rspace_importer::RSpaceImporter},
        ByteVector,
    };

    use crate::history::history_action_tests::random_blake;

    #[tokio::test]
    async fn history_repository_should_process_insert_one_datum() {
        let repo = create_empty_repository();
        let test_datum = datum(1);
        let insert_data = InsertData {
            channel: test_channel_data_prefix(),
            data: vec![test_datum.clone()],
        };

        let next_repo = repo
            .checkpoint(&vec![HotStoreAction::Insert(InsertAction::InsertData(insert_data))])
            .await;
        let history_reader = next_repo.get_history_reader(next_repo.root());
        let data = history_reader
            .unwrap()
            .base()
            .get_data(&test_channel_data_prefix());
        let fetched = data.first().unwrap().clone();

        assert_eq!(fetched, test_datum);
    }

    #[tokio::test]
    async fn history_repository_should_allow_insert_of_joins_datum_continuation_on_same_channel() {
        let repo = create_empty_repository();
        let channel = test_channel_continuations_prefix();

        let test_datum = datum(1);
        let data = InsertData {
            channel: channel.clone(),
            data: vec![test_datum.clone()],
        };

        let test_joins = join(1);
        let joins = InsertJoins {
            channel: channel.clone(),
            joins: test_joins,
        };

        let test_continuation = continuation(1);
        let continuations = InsertContinuations {
            channels: vec![channel.clone()],
            continuations: vec![test_continuation.clone()],
        };

        let next_repo = repo
            .checkpoint(&vec![
                HotStoreAction::Insert(InsertAction::InsertData(data)),
                HotStoreAction::Insert(InsertAction::InsertJoins(joins.clone())),
                HotStoreAction::Insert(InsertAction::InsertContinuations(continuations)),
            ])
            .await;
        let history_reader = next_repo.get_history_reader(next_repo.root());
        let reader = history_reader.as_ref().unwrap().base();

        let fetched_data = reader.get_data(&channel);
        let fetched_continuation = reader.get_continuations(&vec![channel.clone()]);
        let fetched_joins = reader.get_joins(&channel);

        assert_eq!(fetched_data.len(), 1);
        assert_eq!(fetched_data.first().unwrap().clone(), test_datum);

        assert_eq!(fetched_continuation.len(), 1);
        assert_eq!(fetched_continuation.first().unwrap().clone(), test_continuation);

        assert_eq!(fetched_joins.len(), 2);
        assert_eq!(
            HashSet::<String>::from_iter(fetched_joins.into_iter().flatten()),
            HashSet::<String>::from_iter(joins.joins.into_iter().flatten())
        );
    }

    fn test_channel_data_prefix() -> String {
        "channel-data".to_string()
    }

    fn test_channel_joins_prefix() -> String {
        "channel-joins".to_string()
    }

    fn test_channel_continuations_prefix() -> String {
        "channel-continuations".to_string()
    }

    fn insert_datum(
        s: i32,
    ) -> (HotStoreAction<String, String, String, String>, InsertData<String, String>) {
        let insert = InsertData {
            channel: format!("{}{}", test_channel_data_prefix(), s),
            data: vec![datum(s)],
        };

        (HotStoreAction::Insert(InsertAction::InsertData(insert.clone())), insert)
    }

    fn insert_join(s: i32) -> InsertJoins<String> {
        InsertJoins {
            channel: format!("{}{}", test_channel_joins_prefix(), s),
            joins: join(s),
        }
    }

    fn insert_continuation(s: i32) -> InsertContinuations<String, String, String> {
        InsertContinuations {
            channels: vec![format!("{}{}", test_channel_continuations_prefix(), s)],
            continuations: vec![continuation(s)],
        }
    }

    fn join(s: i32) -> Vec<Vec<String>> {
        vec![
            vec![format!("abc{}", s), format!("def{}", s)],
            vec![format!("wer{}", s), format!("tre{}", s)],
        ]
    }

    fn continuation(s: i32) -> WaitingContinuation<String, String> {
        WaitingContinuation {
            patterns: vec![format!("pattern-{}", s)],
            continuation: format!("cont-{}", s),
            persist: true,
            peeks: BTreeSet::new(),
            source: Consume {
                channel_hashes: vec![random_blake()],
                hash: random_blake(),
                persistent: true,
            },
        }
    }

    fn datum(s: i32) -> Datum<String> {
        Datum {
            a: format!("data-{}", s),
            persist: false,
            source: Produce {
                channel_hash: random_blake(),
                hash: random_blake(),
                persistent: false,
            },
        }
    }

    fn create_empty_repository() -> HistoryRepositoryImpl<String, String, String, String> {
        let past_roots = root_repository();
        let empty_history = HistoryInstances::create(
            RadixHistory::empty_root_node_hash(),
            Arc::new(Mutex::new(Box::new(InMemoryKeyValueStore::new()))),
        )
        .unwrap();

        let _ = past_roots.commit(&RadixHistory::empty_root_node_hash());

        HistoryRepositoryImpl {
            current_history: Arc::new(Mutex::new(Box::new(empty_history))),
            roots_repository: Arc::new(Mutex::new(past_roots)),
            leaf_store: Arc::new(Mutex::new(create_inmem_cold_store())),
            rspace_exporter: Arc::new(Mutex::new(Box::new(EmptyExporter))),
            rspace_importer: Arc::new(Mutex::new(Box::new(EmptyImporter))),
            _marker: std::marker::PhantomData,
        }
    }

    struct InmemRootsStore {
        roots: Arc<Mutex<HashSet<Blake3Hash>>>,
        maybe_current_root: Arc<Mutex<Option<Blake3Hash>>>,
    }

    impl RootsStore for InmemRootsStore {
        fn current_root(&self) -> Result<Option<Blake3Hash>, RootError> {
            Ok(self.maybe_current_root.lock().unwrap().clone())
        }

        fn validate_and_set_current_root(
            &self,
            key: Blake3Hash,
        ) -> Result<Option<Blake3Hash>, RootError> {
            let roots_lock = self.roots.lock().unwrap();
            let mut maybe_current_root_lock = self.maybe_current_root.lock().unwrap();

            if roots_lock.contains(&key) {
                *maybe_current_root_lock = Some(key);
                Ok(maybe_current_root_lock.clone())
            } else {
                Ok(None)
            }
        }

        fn record_root(&self, key: &Blake3Hash) -> Result<(), RootError> {
            let mut roots_lock = self.roots.lock().unwrap();
            let mut maybe_current_root_lock = self.maybe_current_root.lock().unwrap();

            *maybe_current_root_lock = Some(key.clone());
            let _ = roots_lock.insert(key.clone());
            Ok(())
        }
    }

    impl InmemRootsStore {
        fn new() -> InmemRootsStore {
            InmemRootsStore {
                roots: Arc::new(Mutex::new(HashSet::new())),
                maybe_current_root: Arc::new(Mutex::new(None)),
            }
        }
    }

    fn root_repository() -> RootRepository {
        RootRepository {
            roots_store: Box::new(InmemRootsStore::new()),
        }
    }

    fn create_inmem_cold_store() -> Box<dyn KeyValueStore> {
        Box::new(InMemoryKeyValueStore::new())
    }

    struct EmptyExporter;

    impl RSpaceExporter for EmptyExporter {
        fn get_root(&self) -> Result<KeyHash, RootError> {
            todo!()
        }
    }

    impl TrieExporter for EmptyExporter {
        fn get_nodes(
            &self,
            start_path: NodePath,
            skip: usize,
            take: usize,
        ) -> Vec<TrieNode<KeyHash>> {
            todo!()
        }

        fn get_history_items(
            &self,
            keys: Vec<KeyHash>,
        ) -> Result<Vec<(KeyHash, Value)>, KvStoreError> {
            todo!()
        }

        fn get_data_items(
            &self,
            keys: Vec<KeyHash>,
        ) -> Result<Vec<(KeyHash, Value)>, KvStoreError> {
            todo!()
        }
    }

    struct EmptyImporter;

    impl RSpaceImporter for EmptyImporter {
        fn get_history_item(&self, hash: KeyHash) -> Option<ByteVector> {
            todo!()
        }
    }

    impl TrieImporter for EmptyImporter {
        fn set_history_items(&self, data: Vec<(KeyHash, Value)>) -> () {
            todo!()
        }

        fn set_data_items(&self, data: Vec<(KeyHash, Value)>) -> () {
            todo!()
        }

        fn set_root(&self, key: &KeyHash) -> () {
            todo!()
        }
    }
}
