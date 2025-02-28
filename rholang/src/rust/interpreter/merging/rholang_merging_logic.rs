// See rholang/src/main/scala/coop/rchain/rholang/interpreter/merging/RholangMergingLogic.scala

use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;

pub struct DeployMergeableData {
    pub channels: Vec<NumberChannel>,
}

pub struct NumberChannel {
    pub hash: Blake2b256Hash,
    pub diff: i64,
}
