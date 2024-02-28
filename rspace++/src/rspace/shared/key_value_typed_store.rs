use super::key_value_store::KvStoreError;
use crate::rspace::shared::key_value_store::KeyValueStore;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{collections::BTreeMap, marker::PhantomData};

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
#[async_trait]
pub trait KeyValueTypedStore<K, V>: Send + Sync
where
    K: Debug + Send + Sync + Clone,
    V: Debug + Send + Sync,
{
    fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError>;

    fn delete(&self, keys: Vec<K>) -> Result<usize, KvStoreError>;

    fn contains(&self, keys: &Vec<K>) -> Result<Vec<bool>, KvStoreError>;

    fn to_map(&self) -> Result<BTreeMap<K, V>, KvStoreError>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreSyntax.scala
    fn get_one(&self, key: &K) -> Result<Option<V>, KvStoreError> {
        let mut values = self.get(vec![key.clone()])?;
        let first_value = values.remove(0);

        match first_value {
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    fn put_if_absent(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError> {
        let keys: Vec<K> = kv_pairs.iter().map(|(k, _)| k.clone()).collect();
        let if_absent = self.contains(&keys)?;
        let kv_if_absent: Vec<_> = kv_pairs.into_iter().zip(if_absent).collect();
        let kv_absent: Vec<_> = kv_if_absent
            .into_iter()
            .filter(|(_, is_present)| !is_present)
            .map(|(kv, _)| kv)
            .collect();

        self.put(kv_absent)
    }
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
#[derive(Clone)]
pub struct KeyValueTypedStoreInstance<K, V> {
    pub store: Box<dyn KeyValueStore>,
    pub _marker: PhantomData<(K, V)>,
}

impl<K, V> KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<K, V>
where
    K: Debug + Send + Sync + Serialize + 'static + Clone + for<'a> Deserialize<'a> + Ord,
    V: Debug + Send + Sync + Serialize + 'static + for<'a> Deserialize<'a>,
{
    fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError> {
        let keys_bytes = keys
            .into_iter()
            .map(|key| bincode::serialize(&key))
            .collect::<Result<Vec<_>, _>>()?;

        let values_bytes = self.store.get(keys_bytes)?;
        let values: Vec<Option<V>> = values_bytes
            .into_iter()
            .map(|value_bytes_opt| match value_bytes_opt {
                Some(bytes) => {
                    let decoded: V = bincode::deserialize(&bytes)
                        .expect("Key Value Typed Store: Failed to deserialize value bytes");
                    Some(decoded)
                }
                None => None,
            })
            .collect();

        Ok(values)
    }

    fn put(&self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError> {
        let pairs_bytes: Vec<(Vec<u8>, Vec<u8>)> = kv_pairs
            .iter()
            .map(|(k, v)| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                let serialized_value = bincode::serialize(v)
                    .expect("Key Value Typed Store: Failed to serialize value");
                (serialized_key, serialized_value)
            })
            .collect();

        Ok(self.store.put(pairs_bytes)?)
    }

    fn delete(&self, keys: Vec<K>) -> Result<usize, KvStoreError> {
        let keys_bytes: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                serialized_key
            })
            .collect();

        let deleted_count = self.store.delete(keys_bytes);
        Ok(deleted_count?)
    }

    fn contains(&self, keys: &Vec<K>) -> Result<Vec<bool>, KvStoreError> {
        let keys_bytes: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| {
                let serialized_key =
                    bincode::serialize(k).expect("Key Value Typed Store: Failed to serialize key");
                serialized_key
            })
            .collect();

        let results = self.store.get(keys_bytes)?;
        Ok(results
            .into_iter()
            .map(|result| !result.is_none())
            .collect())
    }

    fn to_map(&self) -> Result<BTreeMap<K, V>, KvStoreError> {
        let map_bytes = self.store.to_map()?;
        let mut map = BTreeMap::new();
        for (k_bytes, v_bytes) in map_bytes {
            let k: K = bincode::deserialize(&k_bytes)
                .expect("Key Value Typed Store: Failed to deserialize key bytes");
            let v: V = bincode::deserialize(&v_bytes)
                .expect("Key Value Typed Store: Failed to value key bytes");
            map.insert(k, v);
        }
        Ok(map)
    }
}
