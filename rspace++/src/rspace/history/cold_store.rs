use crate::rspace::{
    hashing::serializable_blake3_hash::SerializableBlake3Hash,
    shared::{
        key_value_store::{KeyValueStore, KeyValueStoreOps},
        key_value_typed_store::KeyValueTypedStore,
    },
};
use serde::Deserialize;

// See rspace/src/main/scala/coop/rchain/rspace/history/ColdStore.scala
pub struct ColdStoreInstances;

impl ColdStoreInstances {
    pub fn cold_store(
        store: Box<dyn KeyValueStore>,
    ) -> Box<dyn KeyValueTypedStore<SerializableBlake3Hash, PersistedData>> {
        Box::new(KeyValueStoreOps::to_typed_store::<SerializableBlake3Hash, PersistedData>(store))
    }
}

#[derive(Clone, Deserialize)]
pub enum PersistedData {
    Joins(JoinsLeaf),
    Data(DataLeaf),
    Continuations(ContinuationsLeaf),
}

#[derive(Clone, Deserialize)]
struct JoinsLeaf {
    bytes: Vec<u8>,
}

#[derive(Clone, Deserialize)]
struct DataLeaf {
    bytes: Vec<u8>,
}

#[derive(Clone, Deserialize)]
struct ContinuationsLeaf {
    bytes: Vec<u8>,
}
