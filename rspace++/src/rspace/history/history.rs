use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::{RadixHistory, RadixHistoryInstance};
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

pub struct HistoryInstance;

impl HistoryInstance {
    pub fn create<U: KeyValueStore>(
        root: blake3::Hash,
        store: U,
    ) -> RadixHistory<impl KeyValueTypedStore<Bytes, Bytes>> {
        RadixHistory::create(root, RadixHistoryInstance::create_store(store))
    }
}
