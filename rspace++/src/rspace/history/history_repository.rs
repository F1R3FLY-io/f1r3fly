use crate::rspace::history::cold_store::ColdStoreInstances;
use crate::rspace::history::history::{History, HistoryInstances};
use crate::rspace::history::history_reader::HistoryReader;
use crate::rspace::history::history_repository_impl::HistoryRepositoryImpl;
use crate::rspace::history::root_repository::RootRepository;
use crate::rspace::history::roots_store::RootsStoreInstances;
use crate::rspace::hot_store_action::HotStoreAction;
use crate::rspace::hot_store_trie_action::HotStoreTrieAction;
use crate::rspace::shared::key_value_store::{KeyValueStore, KvStoreError};
use crate::rspace::state::instances::rspace_exporter_store::RSpaceExporterStore;
use crate::rspace::state::instances::rspace_importer_store::RSpaceImporterStore;
use crate::rspace::state::rspace_exporter::RSpaceExporter;
use crate::rspace::state::rspace_importer::RSpaceImporter;
use bytes::Bytes;
use std::marker::PhantomData;

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepository.scala
pub trait HistoryRepository<C, P, A, K> {
    fn checkpoint(
        &self,
        actions: Vec<HotStoreAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>>;

    fn do_checkpoint(
        &self,
        actions: Vec<HotStoreTrieAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>>;

    fn reset(&self, root: blake3::Hash) -> Box<dyn HistoryRepository<C, P, A, K>>;

    fn history(&self) -> Box<dyn History>;

    fn exporter(
        &self,
    ) -> Box<
        dyn RSpaceExporter<
            KeyHash = blake3::Hash,
            NodePath = Vec<(blake3::Hash, Option<u8>)>,
            Value = Bytes,
        >,
    >;

    fn importer(&self) -> Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = Bytes>>;

    fn get_history_reader(
        &self,
        state_hash: blake3::Hash,
    ) -> Box<dyn HistoryReader<blake3::Hash, C, P, A, K>>;

    fn root(&self) -> blake3::Hash;
}

pub struct HistoryRepositoryInstances<C, P, A, K> {
    _marker: PhantomData<(C, P, A, K)>,
}

impl<C, P, A, K> HistoryRepositoryInstances<C, P, A, K> {
    pub async fn lmdb_repository(
        history_key_value_store: Box<dyn KeyValueStore>,
        roots_key_value_store: Box<dyn KeyValueStore>,
        cold_key_value_store: Box<dyn KeyValueStore>,
    ) -> Result<impl HistoryRepository<C, P, A, K>, KvStoreError> {
        // Roots store
        let roots_repository = RootRepository {
            roots_store: Box::new(RootsStoreInstances::roots_store(roots_key_value_store.clone())),
        };

        let current_root = roots_repository.current_root().await?;

        // History store
        let history = HistoryInstances::create(current_root, history_key_value_store.clone());

        // Cold store
        let cold_store = ColdStoreInstances::cold_store(cold_key_value_store.clone());

        // RSpace importer/exporter / directly operates on Store (lmdb)
        let exporter = RSpaceExporterStore::create(
            history_key_value_store.clone(),
            cold_key_value_store.clone(),
            roots_key_value_store.clone(),
        );
        let importer = RSpaceImporterStore::create(
            history_key_value_store,
            cold_key_value_store,
            roots_key_value_store,
        );

        Ok(HistoryRepositoryImpl {
            current_history: Box::new(history),
            roots_repository,
            leaf_store: cold_store,
            rspace_exporter: Box::new(exporter),
            rspace_importer: Box::new(importer),
            _marker: PhantomData,
        })
    }
}
