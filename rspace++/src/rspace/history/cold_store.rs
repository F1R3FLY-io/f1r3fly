use std::sync::{Arc, Mutex};

use crate::rspace::shared::key_value_store::KeyValueStore;
use serde::{Deserialize, Serialize};

// See rspace/src/main/scala/coop/rchain/rspace/history/ColdStore.scala
pub struct ColdStoreInstances;

impl ColdStoreInstances {
    pub fn cold_store(store: Arc<Mutex<Box<dyn KeyValueStore>>>) -> Box<dyn KeyValueStore> {
        let store_lock = store
            .lock()
            .expect("Radix History: Failed to acquire lock on store");
        store_lock.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
