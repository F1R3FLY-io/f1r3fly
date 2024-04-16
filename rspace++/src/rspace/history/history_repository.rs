use crate::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use crate::rspace::history::cold_store::ColdStoreInstances;
use crate::rspace::history::history::{History, HistoryInstances};
use crate::rspace::history::history_reader::HistoryReader;
use crate::rspace::history::history_repository_impl::HistoryRepositoryImpl;
use crate::rspace::history::root_repository::RootRepository;
use crate::rspace::history::roots_store::RootsStoreInstances;
use crate::rspace::hot_store_action::HotStoreAction;
use crate::rspace::hot_store_trie_action::HotStoreTrieAction;
use crate::rspace::shared::key_value_store::KeyValueStore;
use crate::rspace::state::instances::rspace_exporter_store::RSpaceExporterStore;
use crate::rspace::state::instances::rspace_importer_store::RSpaceImporterStore;
use crate::rspace::state::rspace_exporter::RSpaceExporter;
use crate::rspace::state::rspace_importer::RSpaceImporter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::history::HistoryError;
use super::roots_store::RootError;

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepository.scala
#[async_trait]
pub trait HistoryRepository<C: Clone, P: Clone, A: Clone, K: Clone>: Send + Sync {
    async fn checkpoint(
        &self,
        actions: &Vec<HotStoreAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>>;

    async fn do_checkpoint(
        &self,
        actions: Vec<HotStoreTrieAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>>;

    fn reset(
        &self,
        root: &Blake2b256Hash,
    ) -> Result<Box<dyn HistoryRepository<C, P, A, K>>, HistoryError>;

    fn history(&self) -> Arc<Mutex<Box<dyn History>>>;

    fn exporter(&self) -> Arc<Mutex<Box<dyn RSpaceExporter>>>;

    fn importer(&self) -> Arc<Mutex<Box<dyn RSpaceImporter>>>;

    fn get_history_reader(
        &self,
        state_hash: Blake2b256Hash,
    ) -> Result<Box<dyn HistoryReader<Blake2b256Hash, C, P, A, K>>, HistoryError>;

    fn root(&self) -> Blake2b256Hash;
}

pub struct HistoryRepositoryInstances<C, P, A, K> {
    _marker: PhantomData<(C, P, A, K)>,
}

pub const PREFIX_DATUM: u8 = 0x00;
pub const PREFIX_KONT: u8 = 0x01;
pub const PREFIX_JOINS: u8 = 0x02;

impl<C, P, A, K> HistoryRepositoryInstances<C, P, A, K>
where
    C: Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    P: Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    A: Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    K: Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
{
    pub fn lmdb_repository(
        history_key_value_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
        roots_key_value_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
        cold_key_value_store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    ) -> Result<impl HistoryRepository<C, P, A, K>, HistoryRepositoryError> {
        // Roots store
        let roots_repository = RootRepository {
            roots_store: Box::new(RootsStoreInstances::roots_store(roots_key_value_store.clone())),
        };

        let current_root = roots_repository.current_root()?;

        // History store
        let history = HistoryInstances::create(current_root, history_key_value_store.clone())?;

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
            current_history: Arc::new(Mutex::new(Box::new(history))),
            roots_repository: Arc::new(Mutex::new(roots_repository)),
            leaf_store: Arc::new(Mutex::new(cold_store)),
            rspace_exporter: Arc::new(Mutex::new(Box::new(exporter))),
            rspace_importer: Arc::new(Mutex::new(Box::new(importer))),
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub enum HistoryRepositoryError {
    RootError(RootError),
    HistoryError(HistoryError),
}

impl std::fmt::Display for HistoryRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryRepositoryError::RootError(err) => write!(f, "Root Error: {}", err),
            HistoryRepositoryError::HistoryError(err) => write!(f, "History Error: {}", err),
        }
    }
}

impl From<RootError> for HistoryRepositoryError {
    fn from(error: RootError) -> Self {
        HistoryRepositoryError::RootError(error)
    }
}

impl From<HistoryError> for HistoryRepositoryError {
    fn from(error: HistoryError) -> Self {
        HistoryRepositoryError::HistoryError(error)
    }
}
