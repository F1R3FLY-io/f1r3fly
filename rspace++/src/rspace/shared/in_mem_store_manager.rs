use super::{
    in_mem_key_value_store::InMemoryKeyValueStore, key_value_store::KeyValueStore,
    key_value_store_manager::KeyValueStoreManager,
};
use async_trait::async_trait;
use dashmap::DashMap;

// See shared/src/main/scala/coop/rchain/store/InMemoryStoreManager.scala
// Simple in-memory key value store manager
pub struct InMemoryStoreManager {
    state: DashMap<String, InMemoryKeyValueStore>,
}

#[async_trait]
impl KeyValueStoreManager for InMemoryStoreManager {
    async fn store(&mut self, name: String) -> Result<Box<dyn KeyValueStore>, heed::Error> {
        let kv_store = self
            .state
            .entry(name)
            .or_insert_with(|| InMemoryKeyValueStore::new());

        Ok(Box::new(kv_store.value().clone()))
    }

    async fn shutdown(&mut self) -> Result<(), heed::Error> {
        self.state.clear();
        Ok(())
    }
}

impl InMemoryStoreManager {
    pub fn new() -> Self {
        InMemoryStoreManager {
            state: DashMap::new(),
        }
    }
}
