use crate::rspace::shared::key_value_typed_store::{
    KeyValueTypedStore, KeyValueTypedStoreInstance,
};
use bytes::Bytes;
use std::{io::Cursor, marker::PhantomData};

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
pub trait KeyValueStore {
    fn get<T, F>(&self, keys: Vec<Cursor<Bytes>>, from_buffer: F) -> Vec<Option<T>>
    where
        F: Fn(Cursor<Bytes>) -> T;

    fn put<T, F>(&self, kv_pairs: Vec<(Cursor<Bytes>, T)>, to_buffer: F) -> ()
    where
        F: Fn(T) -> Cursor<Bytes>;

    fn delete(&self, keys: Vec<Cursor<Bytes>>) -> i32;

    fn iterate<T, F>(&self, f: F) -> T
    where
        F: FnOnce(Box<dyn Iterator<Item = (Cursor<Bytes>, Cursor<Bytes>)>>) -> T;
}

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
pub struct KeyValueStoreOps<T: KeyValueStore> {
    store: T,
}

impl<T: KeyValueStore + Clone> KeyValueStoreOps<T> {
    pub fn to_typed_store(store: T) -> impl KeyValueTypedStore<Bytes, Bytes> + Clone {
        KeyValueTypedStoreInstance {
            store,
            _marker: PhantomData,
        }
    }
}
