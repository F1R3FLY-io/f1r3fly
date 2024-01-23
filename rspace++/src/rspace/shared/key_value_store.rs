use crate::rspace::shared::key_value_typed_store::{
    KeyValueTypedStore, KeyValueTypedStoreInstance,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    async fn get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>, heed::Error>;

    async fn put(&self, kv_pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), heed::Error>;

    async fn delete(&self, keys: Vec<Vec<u8>>) -> Result<usize, heed::Error>;

    async fn iterate(&self, f: fn(Vec<u8>, Vec<u8>)) -> Result<(), heed::Error>;

    fn clone_box(&self) -> Box<dyn KeyValueStore>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
    async fn get_one(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let values = self.get(vec![key]).await;
        let first_value = values.map(|mut v| v.remove(0));

        match first_value {
            Ok(value) => value,
            Err(err) => {
                println!("Key_Value_Store: {}", err);
                None
            }
        }
    }

    async fn put_one(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), KvStoreError> {
        self.put(vec![(key, value)]).await?;
        Ok(())
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
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    DeserializationError(serde_json::Error),
    HeedError(heed::Error),
    BoxBincodeError(Box<bincode::ErrorKind>),
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KvStoreError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            KvStoreError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            KvStoreError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            KvStoreError::IoError(e) => write!(f, "I/O error: {}", e),
            KvStoreError::HeedError(e) => write!(f, "Heed error: {}", e),
            KvStoreError::BoxBincodeError(e) => write!(f, "Bincode error: {}", e),
        }
    }
}

impl Error for KvStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            KvStoreError::SerializationError(e) => Some(e),
            KvStoreError::DeserializationError(e) => Some(e),
            KvStoreError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<heed::Error> for KvStoreError {
    fn from(error: heed::Error) -> Self {
        KvStoreError::HeedError(error)
    }
}

impl From<Box<bincode::ErrorKind>> for KvStoreError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        KvStoreError::BoxBincodeError(error)
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
pub struct KeyValueStoreOps;

impl KeyValueStoreOps {
    pub fn to_typed_store<K, V>(store: Box<dyn KeyValueStore>) -> impl KeyValueTypedStore<K, V>
    where
        K: Clone + Debug + Send + Sync + Serialize + 'static,
        V: Clone + Send + Sync + for<'a> Deserialize<'a> + 'static,
    {
        KeyValueTypedStoreInstance {
            store,
            _marker: PhantomData,
        }
    }
}
