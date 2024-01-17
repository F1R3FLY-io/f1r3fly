use crate::rspace::shared::key_value_typed_store::{
    KeyValueTypedStore, KeyValueTypedStoreInstance,
};
use bytes::Bytes;
use std::{any::Any, io::Cursor, marker::PhantomData};

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
pub trait KeyValueStore: Send + Sync {
    fn get(
        &self,
        keys: Vec<Cursor<Bytes>>,
        from_buffer: fn(Cursor<Bytes>) -> Box<dyn Any>,
    ) -> Vec<Option<Box<dyn Any>>>;

    fn put(
        &self,
        kv_pairs: Vec<(Cursor<Bytes>, Box<dyn Any>)>,
        to_buffer: fn(Box<dyn Any>) -> Cursor<Bytes>,
    ) -> ();

    fn delete(&self, keys: Vec<Cursor<Bytes>>) -> i32;

    fn iterate(
        &self,
        f: fn(Box<dyn Iterator<Item = (Cursor<Bytes>, Cursor<Bytes>)>>) -> Box<dyn Any>,
    ) -> Box<dyn Any>;

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
