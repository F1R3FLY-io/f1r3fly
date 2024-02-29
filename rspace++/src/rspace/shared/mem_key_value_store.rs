use std::collections::HashMap;

use super::key_value_store::{KeyValueStore, KvStoreError};
use crate::rspace::{ByteBuffer, ByteVector};
use bincode;
use dashmap::DashMap;

// See shared/src/main/scala/coop/rchain/store/InMemoryKeyValueStore.scala
#[derive(Clone)]
pub struct InMemoryKeyValueStore {
    state: DashMap<ByteBuffer, ByteVector>,
}

impl KeyValueStore for InMemoryKeyValueStore {
    fn get(&self, keys: Vec<ByteBuffer>) -> Result<Vec<Option<ByteBuffer>>, KvStoreError> {
        println!("\nin_mem_state get: {:?}", self.state);
        let result = keys
            .into_iter()
            .map(|key| {
                self.state
                    .get(&key)
                    .map(|value| bincode::deserialize(&value).ok())
                    .flatten()
            })
            .collect();
        Ok(result)
    }

    fn put(&self, kv_pairs: Vec<(ByteBuffer, ByteBuffer)>) -> Result<(), KvStoreError> {
        println!("\nhit put in mem_kv");
        println!("\nin_mem_state before put: {:?}", self.state);
        for (key, value) in kv_pairs {
            let encoded = bincode::serialize(&value).unwrap();
            self.state.insert(key, encoded);
        }

        println!("\nin_mem_state after put: {:?}", self.state);

        Ok(())
    }

    fn delete(&self, keys: Vec<ByteBuffer>) -> Result<usize, KvStoreError> {
        Ok(keys
            .into_iter()
            .filter_map(|key| self.state.remove(&key))
            .count())
    }

    fn iterate(&self, _f: fn(ByteBuffer, ByteBuffer)) -> Result<(), KvStoreError> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn KeyValueStore> {
        Box::new(self.clone())
    }

    fn to_map(&self) -> Result<HashMap<ByteBuffer, ByteBuffer>, KvStoreError> {
        let mut map = HashMap::new();
        for entry in self.state.iter() {
            let (key, value) = entry.pair();
            map.insert(
                key.clone(),
                bincode::deserialize(value).expect("Mem Key Value Store: Unable to deserialize"),
            );
        }
        Ok(map)
    }

    fn size_bytes(&self) -> usize {
        self.state
            .iter()
            .map(|ref_entry| {
                let (key, value) = ref_entry.pair();
                key.len() + value.len()
            })
            .sum()
    }
}

impl InMemoryKeyValueStore {
    pub fn new() -> Self {
        InMemoryKeyValueStore {
            state: DashMap::new(),
        }
    }

    pub fn clear(&self) {
        self.state.clear();
    }

    pub fn num_records(&self) -> usize {
        self.state.len()
    }
}
