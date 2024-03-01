use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::RadixHistory;
use crate::rspace::shared::key_value_store::KeyValueStore;
use std::sync::{Arc, Mutex};

use super::radix_tree::RadixTreeError;

// See rspace/src/main/scala/coop/rchain/rspace/history/History.scala
pub trait History: Send + Sync {
    fn read(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, HistoryError>;

    fn process(&self, actions: Vec<HistoryAction>) -> Result<Box<dyn History>, HistoryError>;

    fn root(&self) -> Blake3Hash;

    fn reset(&self, root: &Blake3Hash) -> Box<dyn History>;
}

pub struct HistoryInstances;

impl HistoryInstances {
    pub fn create(
        root: Blake3Hash,
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
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HistoryError::ActionError(err) => write!(f, "Actions Error: {}", err),
            HistoryError::RadixTreeError(err) => write!(f, "Radix Tree Error: {}", err),
        }
    }
}

impl From<RadixTreeError> for HistoryError {
    fn from(error: RadixTreeError) -> Self {
        HistoryError::RadixTreeError(error)
    }
}
