use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    shared::{
        key_value_store::{KeyValueStore, KeyValueStoreOps},
        key_value_typed_store::KeyValueTypedStore,
    },
};
use serde::{Deserialize, Serialize};

// See rspace/src/main/scala/coop/rchain/rspace/history/ColdStore.scala
pub struct ColdStoreInstances;

impl ColdStoreInstances {
    pub fn cold_store(
        store: Box<dyn KeyValueStore>,
    ) -> Box<dyn KeyValueTypedStore<Blake3Hash, PersistedData>> {
        Box::new(KeyValueStoreOps::to_typed_store::<Blake3Hash, PersistedData>(store))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum PersistedData {
    Joins(JoinsLeaf),
    Data(DataLeaf),
    Continuations(ContinuationsLeaf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinsLeaf {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataLeaf {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinuationsLeaf {
    pub bytes: Vec<u8>,
}
