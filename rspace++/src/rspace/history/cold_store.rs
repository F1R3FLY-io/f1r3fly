use crate::rspace::shared::{
    key_value_store::{KeyValueStore, KeyValueStoreOps},
    key_value_typed_store::KeyValueTypedStore,
};
use bytes::Bytes;

// See rspace/src/main/scala/coop/rchain/rspace/history/ColdStore.scala
pub struct ColdStoreInstances;

impl ColdStoreInstances {
    pub fn cold_store(
        store: impl KeyValueStore + Clone,
    ) -> Box<dyn KeyValueTypedStore<blake3::Hash, PersistedData>> {
        Box::new(KeyValueStoreOps::to_typed_store::<blake3::Hash, PersistedData>(store))
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
