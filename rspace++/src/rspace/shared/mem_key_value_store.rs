use std::collections::{BTreeMap, HashMap};

use super::key_value_store::{KeyValueStore, KvStoreError};
use crate::rspace::{ByteBuffer, ByteVector};
use bincode;
use dashmap::DashMap;

// See shared/src/main/scala/coop/rchain/store/InMemoryKeyValueStore.scala
#[derive(Clone)]
pub struct InMemoryKeyValueStore {
    state: BTreeMap<ByteBuffer, ByteVector>,
}

impl KeyValueStore for InMemoryKeyValueStore {
    fn get(&self, keys: Vec<ByteBuffer>) -> Result<Vec<Option<ByteBuffer>>, KvStoreError> {
        // println!("\nin_mem_state get: {:?}", self.state);
        // println!("\nin_mem_state get keys: {:?}", keys);
        let result = keys
            .into_iter()
            .map(|key| {
                self.state.get(&key).map(|value| {
                    // println!(
                    //     "\nRetrieved value for key {:?}: {:?}",
                    //     key_value.key(),
                    //     key_value.value()
                    // );
                    bincode::deserialize(&value)
                        .expect("Mem Key Value Store: Failed to deserialize")
                })
            })
            .collect::<Vec<Option<_>>>();

        // println!("\nresults in get: {:?}", result);

        Ok(result)
    }

    fn put(&mut self, kv_pairs: Vec<(ByteBuffer, ByteBuffer)>) -> Result<(), KvStoreError> {
        // println!("\nhit put in mem_kv");
        // println!("\nin_mem_state before put: {:?}", self.state);
        for (key, value) in kv_pairs {
            let encoded_value = bincode::serialize(&value).unwrap();
            self.state.insert(key, encoded_value);
        }

        // println!("\nin_mem_state after put: {:?}", self.state);

        Ok(())
    }

    fn delete(&mut self, keys: Vec<ByteBuffer>) -> Result<usize, KvStoreError> {
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
            let (key, value) = entry;
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
                let (key, value) = ref_entry;
                key.len() + value.len()
            })
            .sum()
    }
}

impl InMemoryKeyValueStore {
    pub fn new() -> Self {
        InMemoryKeyValueStore {
            state: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.state.clear();
    }

    pub fn num_records(&self) -> usize {
        self.state.len()
    }
}
