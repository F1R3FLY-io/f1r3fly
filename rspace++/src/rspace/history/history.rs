use crate::rspace::hashing::blake3_hash::Blake3Hash;
use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::RadixHistory;
use crate::rspace::shared::key_value_store::KeyValueStore;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/History.scala
pub trait History: Send + Sync {
    fn read(&self, key: Bytes) -> Option<Bytes>;

    fn process(&self, actions: Vec<HistoryAction>) -> Box<dyn History>;

    fn root(&self) -> Blake3Hash;

    fn reset(&self, root: &Blake3Hash) -> Box<dyn History>;
}

pub struct HistoryInstances;

impl HistoryInstances {
    pub fn create(root: Blake3Hash, store: Box<dyn KeyValueStore>) -> RadixHistory {
        let typed_store = RadixHistory::create_store(store.to_owned());
        RadixHistory::create(root, typed_store)
    }
}
