// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala

use std::{collections::HashMap, marker::PhantomData};

use crate::rust::{store::key_value_store::KeyValueStore, BitVector};

use super::{key_value_store::KvStoreError, key_value_typed_store::KeyValueTypedStore};

pub struct KeyValueTypedStoreImpl<K, V> {
    store: Box<dyn KeyValueStore>,
    phantom_data: PhantomData<(K, V)>,
}

impl<K, V> KeyValueTypedStoreImpl<K, V>
where
    K: serde::Serialize + for<'a> serde::Deserialize<'a>,
    V: serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    pub fn new(store: Box<dyn KeyValueStore>) -> Self {
        Self {
            store,
            phantom_data: PhantomData,
        }
    }

    pub fn encode_key(&self, key: &K) -> Result<BitVector, KvStoreError> {
        Ok(bincode::serialize(key)?)
    }

    pub fn decode_key(&self, encoded_key: &BitVector) -> Result<K, KvStoreError> {
        Ok(bincode::deserialize(encoded_key)?)
    }

    pub fn encode_value(&self, value: &V) -> Result<BitVector, KvStoreError> {
        Ok(bincode::serialize(value)?)
    }

    pub fn decode_value(&self, encoded_value: &BitVector) -> Result<V, KvStoreError> {
        Ok(bincode::deserialize(encoded_value)?)
    }
}

impl<K, V> KeyValueTypedStore<K, V> for KeyValueTypedStoreImpl<K, V>
where
    K: serde::Serialize + for<'a> serde::Deserialize<'a>,
    V: serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    fn get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>, KvStoreError> {
        let keys_bit_vector = keys
            .iter()
            .map(|key| self.encode_key(key))
            .collect::<Result<Vec<_>, _>>()?;
        let values_bytes = self.store.get(&keys_bit_vector)?;

        let values = values_bytes
            .iter()
            .map(|value_opt| {
                value_opt
                    .as_ref()
                    .map(|value| self.decode_value(value))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(values)
    }

    fn put(&mut self, kv_pairs: Vec<(K, V)>) -> Result<(), KvStoreError> {
        let pairs_bit_vector = kv_pairs
            .iter()
            .map(|(key, value)| {
                let encoded_key = self.encode_key(key)?;
                let encoded_value = self.encode_value(value)?;
                Ok((encoded_key, encoded_value))
            })
            .collect::<Result<Vec<(BitVector, BitVector)>, KvStoreError>>()?;

        self.store.put(pairs_bit_vector)?;
        Ok(())
    }

    fn delete(&mut self, keys: Vec<K>) -> Result<(), KvStoreError> {
        let keys_bit_vector = keys
            .iter()
            .map(|key| self.encode_key(key))
            .collect::<Result<Vec<_>, _>>()?;
        self.store.delete(keys_bit_vector)?;
        Ok(())
    }

    fn contains(&self, keys: Vec<K>) -> Result<Vec<bool>, KvStoreError> {
        let keys_bit_vector = keys
            .iter()
            .map(|key| self.encode_key(key))
            .collect::<Result<Vec<_>, _>>()?;

        let results = self.store.get(&keys_bit_vector)?;
        Ok(results.iter().map(|result| result.is_some()).collect())
    }

    fn collect<F, T>(&self, _f: F) -> Result<Vec<T>, KvStoreError>
    where
        F: FnMut((&K, Box<dyn FnOnce() -> V>)) -> Option<T>,
    {
        todo!()
    }

    fn to_map(&self) -> Result<HashMap<K, V>, KvStoreError> {
        todo!()
    }
}
