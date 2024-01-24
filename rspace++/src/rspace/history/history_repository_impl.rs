use super::history_reader::HistoryReader;
use super::instances::rspace_history_reader_impl::RSpaceHistoryReaderImpl;
use crate::rspace::hashing::serializable_blake3_hash::SerializableBlake3Hash;
use crate::rspace::history::cold_store::PersistedData;
use crate::rspace::history::history::History;
use crate::rspace::history::history_repository::HistoryRepository;
use crate::rspace::history::root_repository::RootRepository;
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::state::rspace_exporter::RSpaceExporter;
use crate::rspace::state::rspace_importer::RSpaceImporter;
use bytes::Bytes;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepositoryImpl.scala
pub struct HistoryRepositoryImpl<C, P, A, K> {
    pub current_history: Arc<Mutex<Box<dyn History>>>,
    pub roots_repository: RootRepository,
    pub leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<SerializableBlake3Hash, PersistedData>>>>,
    pub rspace_exporter: Box<
        dyn RSpaceExporter<
            KeyHash = blake3::Hash,
            NodePath = Vec<(blake3::Hash, Option<u8>)>,
            Value = Bytes,
        >,
    >,
    pub rspace_importer: Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = Bytes>>,
    pub _marker: PhantomData<(C, P, A, K)>,
}

impl<C: 'static, P: 'static, A: 'static, K: 'static> HistoryRepository<C, P, A, K>
    for HistoryRepositoryImpl<C, P, A, K>
{
    fn checkpoint(
        &self,
        actions: Vec<crate::rspace::hot_store_action::HotStoreAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn do_checkpoint(
        &self,
        actions: Vec<crate::rspace::hot_store_trie_action::HotStoreTrieAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn reset(&self, root: blake3::Hash) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn history(&self) -> Arc<Mutex<Box<dyn History>>> {
        self.current_history.clone()
    }

    fn exporter(
        &self,
    ) -> Box<
        dyn RSpaceExporter<
            KeyHash = blake3::Hash,
            NodePath = Vec<(blake3::Hash, Option<u8>)>,
            Value = bytes::Bytes,
        >,
    > {
        todo!()
    }

    fn importer(&self) -> Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = bytes::Bytes>> {
        todo!()
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
