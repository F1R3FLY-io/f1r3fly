use super::hashing::blake3_hash::Blake3Hash;

// See rspace/src/main/scala/coop/rchain/rspace/Checkpoint.scala
pub struct Checkpoint {
    pub root: Blake3Hash,
    // log: Log
}
