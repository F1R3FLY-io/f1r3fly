use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::RadixHistory;
use crate::rspace::shared::key_value_store::KeyValueStore;
use crate::rspace::{hashing::blake2b256_hash::Blake2b256Hash, shared::key_value_store::KvStoreError};
use std::sync::{Arc, Mutex};

use super::radix_tree::RadixTreeError;
use super::roots_store::RootError;

// See rspace/src/main/scala/coop/rchain/rspace/history/History.scala
pub trait History: Send + Sync {
    fn read(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, HistoryError>;

    fn process(&self, actions: Vec<HistoryAction>) -> Result<Box<dyn History>, HistoryError>;

    fn root(&self) -> Blake2b256Hash;

    fn reset(&self, root: &Blake2b256Hash) -> Result<Box<dyn History>, HistoryError>;
}

pub struct HistoryInstances;

impl HistoryInstances {
    pub fn create(
        root: Blake2b256Hash,
        store: Arc<Mutex<Box<dyn KeyValueStore>>>,
    ) -> Result<RadixHistory, HistoryError> {
        let typed_store = RadixHistory::create_store(store.to_owned());
        RadixHistory::create(root, typed_store)
    }
}

#[derive(Debug)]
pub enum HistoryError {
    ActionError(String),
    RadixTreeError(RadixTreeError),
    KvStoreError(KvStoreError),
    RootError(RootError),
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::ActionError(err) => write!(f, "Actions Error: {}", err),
            HistoryError::RadixTreeError(err) => write!(f, "Radix Tree Error: {}", err),
            HistoryError::KvStoreError(err) => write!(f, "Key Value Store Error: {}", err),
            HistoryError::RootError(err) => write!(f, "Root Error: {}", err),
        }
    }
}

impl From<RadixTreeError> for HistoryError {
    fn from(error: RadixTreeError) -> Self {
        HistoryError::RadixTreeError(error)
    }
}

impl From<KvStoreError> for HistoryError {
    fn from(error: KvStoreError) -> Self {
        HistoryError::KvStoreError(error)
    }
}

impl From<RootError> for HistoryError {
    fn from(error: RootError) -> Self {
        HistoryError::RootError(error)
    }
}
