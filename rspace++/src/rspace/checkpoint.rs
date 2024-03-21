use super::{event::Produce, hashing::blake2b256_hash::Blake2b256Hash, hot_store::HotStoreState};
use std::collections::HashMap;
use std::hash::Hash;

// See rspace/src/main/scala/coop/rchain/rspace/Checkpoint.scala
pub struct SoftCheckpoint<C: Eq + Hash, P: Clone, A: Clone, K: Clone> {
    pub cache_snapshot: HotStoreState<C, P, A, K>,
    // log: trace.Log,
    pub produce_counter: HashMap<Produce, i32>,
}

pub struct Checkpoint {
    pub root: Blake2b256Hash,
    // log: trace.Log
}
