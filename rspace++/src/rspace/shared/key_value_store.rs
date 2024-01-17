use crate::rspace::shared::key_value_typed_store::{
    KeyValueTypedStore, KeyValueTypedStoreInstance,
};
use async_trait::async_trait;
use std::marker::PhantomData;

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    async fn get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>, heed::Error>;

    async fn put(&self, kv_pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), heed::Error>;

    async fn delete(&self, keys: Vec<Vec<u8>>) -> Result<usize, heed::Error>;

    async fn iterate(&self, f: fn(Vec<u8>, Vec<u8>)) -> Result<(), heed::Error>;

    fn clone_box(&self) -> Box<dyn KeyValueStore>;
}

impl Clone for Box<dyn KeyValueStore> {
    fn clone(&self) -> Box<dyn KeyValueStore> {
        self.clone_box()
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
pub struct KeyValueStoreOps;

impl KeyValueStoreOps {
    pub fn to_typed_store<K: Clone, V: Clone>(
        store: Box<dyn KeyValueStore>,
    ) -> impl KeyValueTypedStore<K, V> {
        KeyValueTypedStoreInstance {
            store,
            _marker: PhantomData,
        }
    }
}
