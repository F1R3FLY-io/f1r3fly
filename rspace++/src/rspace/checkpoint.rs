use super::{event::Produce, hashing::blake3_hash::Blake3Hash, hot_store::HotStoreState};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

// See rspace/src/main/scala/coop/rchain/rspace/Checkpoint.scala
pub struct SoftCheckpoint<C: Eq + Hash, P: Clone, A: Clone, K: Clone> {
    pub cache_snapshot: Arc<Mutex<HotStoreState<C, P, A, K>>>,
    // log: trace.Log,
    pub produce_counter: HashMap<Produce, i32>,
}

pub struct Checkpoint {
    pub root: Blake3Hash,
    // log: trace.Log
}
