use crate::rspace::{
    hashing::serializable_blake3_hash::SerializableBlake3Hash,
    shared::{
        key_value_store::{KeyValueStore, KeyValueStoreOps},
        key_value_typed_store::KeyValueTypedStore,
    },
};
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/ColdStore.scala
pub struct ColdStoreInstances;

impl ColdStoreInstances {
    pub fn cold_store(
        store: Box<dyn KeyValueStore>,
    ) -> Box<dyn KeyValueTypedStore<SerializableBlake3Hash, PersistedData>> {
        Box::new(KeyValueStoreOps::to_typed_store::<SerializableBlake3Hash, PersistedData>(store))
    }
}

#[derive(Clone)]
pub enum PersistedData {
    Joins(JoinsLeaf),
    Data(DataLeaf),
    Continuations(ContinuationsLeaf),
}

#[derive(Clone)]
struct JoinsLeaf {
    bytes: Bytes,
}

#[derive(Clone)]
struct DataLeaf {
    bytes: Bytes,
}

#[derive(Clone)]
struct ContinuationsLeaf {
    bytes: Bytes,
}
