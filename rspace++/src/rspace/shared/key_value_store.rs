use crate::rspace::{ByteBuffer, ByteVector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// See shared/src/main/scala/coop/rchain/store/KeyValueStore.scala
pub trait KeyValueStore: Send + Sync {
    fn get(&self, keys: &Vec<ByteBuffer>) -> Result<Vec<Option<ByteBuffer>>, KvStoreError>;

    fn put(&mut self, kv_pairs: Vec<(ByteBuffer, ByteBuffer)>) -> Result<(), KvStoreError>;

    fn delete(&mut self, keys: Vec<ByteBuffer>) -> Result<usize, KvStoreError>;

    fn iterate(&self, f: fn(ByteBuffer, ByteBuffer)) -> Result<(), KvStoreError>;

    fn clone_box(&self) -> Box<dyn KeyValueStore>;

    fn to_map(&self) -> Result<HashMap<ByteBuffer, ByteBuffer>, KvStoreError>;

    fn print_store(&self) -> ();

    fn contains(&self, keys: &Vec<ByteBuffer>) -> Result<Vec<bool>, KvStoreError> {
        // println!("\nkeys in contains: {:?}", keys);
        // let serialized_keys: Vec<ByteVector> = keys
        //     .iter()
        //     .map(|k| {
        //         let serialized_key =
        //             bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
        //         serialized_key
        //     })
        //     .collect();

        // println!("\nkeys_bytes in contains: {:?}", keys_bytes);

        let results = self.get(keys)?;
        // println!("\nresults in contains: {:?}", results);
        Ok(results
            .into_iter()
            .map(|result| !result.is_none())
            .collect())
    }

    // See shared/src/main/scala/coop/rchain/store/KeyValueStoreSyntax.scala
    fn get_one(&self, key: &ByteBuffer) -> Result<Option<ByteBuffer>, KvStoreError> {
        let values = self.get(&vec![key.to_vec()])?;

        // println!("\nget_values in get_one: {:?}", values);

        match values.split_first() {
            Some((first_value, _)) => Ok(first_value.clone()),
            None => Ok(None),
        }
    }

    fn put_one(&mut self, key: ByteBuffer, value: ByteBuffer) -> Result<(), KvStoreError> {
        self.put(vec![(key, value)])
    }

    fn put_if_absent(
        &mut self,
        kv_pairs: Vec<(ByteBuffer, ByteBuffer)>,
    ) -> Result<(), KvStoreError> {
        let keys: Vec<ByteBuffer> = kv_pairs.iter().map(|(k, _)| k.clone()).collect();
        let if_absent = self.contains(&keys)?;
        let kv_if_absent: Vec<_> = kv_pairs.into_iter().zip(if_absent).collect();
        let kv_absent: Vec<_> = kv_if_absent
            .into_iter()
            .filter(|(_, is_present)| !is_present)
            .map(|(kv, _)| kv)
            .collect();

        self.put(kv_absent)
    }

    fn size_bytes(&self) -> usize;
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
// pub struct KeyValueStoreOps;

// impl KeyValueStoreOps {
//     pub fn to_typed_store<K, V>(
//         store: Arc<Mutex<Box<dyn KeyValueStore>>>,
//     ) -> impl KeyValueTypedStore<K, V>
//     where
//         K: Clone + Debug + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a> + Ord,
//         V: Clone + Debug + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>,
//     {
//         KeyValueTypedStoreInstance {
//             store,
//             _marker: PhantomData,
//         }
//     }
// }
