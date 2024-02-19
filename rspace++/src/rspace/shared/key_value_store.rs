use crate::rspace::shared::key_value_typed_store::{
    KeyValueTypedStore, KeyValueTypedStoreInstance,
};
use crate::rspace::ByteBuffer;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
pub trait KeyValueStore: Send + Sync {
    fn get(&self, keys: Vec<ByteBuffer>) -> Result<Vec<Option<ByteBuffer>>, KvStoreError>;

    fn put(&self, kv_pairs: Vec<(ByteBuffer, ByteBuffer)>) -> Result<(), KvStoreError>;

    fn delete(&self, keys: Vec<ByteBuffer>) -> Result<usize, KvStoreError>;

    fn iterate(&self, f: fn(ByteBuffer, ByteBuffer)) -> Result<(), KvStoreError>;

    fn clone_box(&self) -> Box<dyn KeyValueStore>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
    fn get_one(&self, key: ByteBuffer) -> Result<Option<ByteBuffer>, KvStoreError> {
        let values = self.get(vec![key])?;

        match values.split_first() {
            Some((first_value, _)) => Ok(first_value.clone()),
            None => Ok(None),
        }
    }

    fn put_one(&self, key: ByteBuffer, value: ByteBuffer) -> Result<(), KvStoreError> {
        self.put(vec![(key, value)])
    }
}

impl Clone for Box<dyn KeyValueStore> {
    fn clone(&self) -> Box<dyn KeyValueStore> {
        self.clone_box()
    }
}

#[derive(Debug)]
pub enum KvStoreError {
    KeyNotFound(String),
    IoError(heed::Error),
    SerializationError(Box<bincode::ErrorKind>),
}

impl std::fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KvStoreError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            KvStoreError::IoError(e) => write!(f, "I/O error: {}", e),
            KvStoreError::SerializationError(e) => write!(f, "SerializationError error: {}", e),
        }
    }
}

impl From<heed::Error> for KvStoreError {
    fn from(error: heed::Error) -> Self {
        KvStoreError::IoError(error)
    }
}

impl From<Box<bincode::ErrorKind>> for KvStoreError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        KvStoreError::SerializationError(error)
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
pub struct KeyValueStoreOps;

impl KeyValueStoreOps {
    pub fn to_typed_store<K, V>(store: Box<dyn KeyValueStore>) -> impl KeyValueTypedStore<K, V>
    where
        K: Clone + Debug + Send + Sync + Serialize + 'static,
        V: Clone + Debug + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>,
    {
        KeyValueTypedStoreInstance {
            store,
            _marker: PhantomData,
        }
    }
}
