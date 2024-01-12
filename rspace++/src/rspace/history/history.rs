use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::{RadixHistory, RadixHistoryInstances};
use crate::rspace::shared::key_value_store::KeyValueStore;
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/History.scala
pub trait History {
    fn read(&self, key: Bytes) -> Option<Bytes>;

    fn process(&self, actions: Vec<HistoryAction>) -> Box<dyn History>;

    fn root(&self) -> blake3::Hash;

    fn reset(&self, root: blake3::Hash) -> Box<dyn History>;
}

pub struct HistoryInstances;

impl HistoryInstances {
    pub fn create(
        root: blake3::Hash,
        store: impl KeyValueStore + Clone,
    ) -> RadixHistory<impl KeyValueTypedStore<Bytes, Bytes>> {
        let typed_store = RadixHistoryInstances::create_store(store);
        RadixHistoryInstances::create(root, typed_store)
    }
}
