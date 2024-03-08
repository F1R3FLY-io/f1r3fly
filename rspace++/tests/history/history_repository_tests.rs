// See src/firefly/f1r3fly/rspace/src/test/scala/coop/rchain/rspace/history/HistoryRepositorySpec.scala
#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        sync::{Arc, Mutex},
    };

    use rspace_plus_plus::rspace::{
        hashing::blake3_hash::Blake3Hash,
        history::{
            history::HistoryInstances,
            history_repository_impl::HistoryRepositoryImpl,
            instances::radix_history::RadixHistory,
            root_repository::RootRepository,
            roots_store::{RootError, RootsStore},
        },
        shared::mem_key_value_store::InMemoryKeyValueStore,
    };

    #[test]
    fn history_repository_should_process_insert_one_datum() {}

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
            leaf_store: todo!(),
            rspace_exporter: todo!(),
            rspace_importer: todo!(),
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

    // fn create_inmem_cold_store() -> ColdK
}
