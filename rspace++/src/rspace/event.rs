use super::hashing::blake3_hash::Blake3Hash;
use crate::rspace::hashing::stable_hash_provider::*;
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/trace/Event.scala
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Produce {
    pub channel_hash: Blake3Hash,
    pub hash: Blake3Hash,
    pub persistent: bool,
}

impl Produce {
    pub fn create<C: Serialize, A: Serialize>(channel: C, datum: A, persistent: bool) -> Produce {
        let channel_hash = hash(&channel);
        let hash = hash_produce(channel_hash.bytes(), datum, persistent);
        Produce {
            channel_hash,
            hash,
            persistent,
        }
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/trace/Event.scala
#[derive(Clone, Debug)]
pub struct Consume {
    pub channel_hashes: Vec<Blake3Hash>,
    pub hash: Blake3Hash,
    pub persistent: bool,
}

impl Consume {
    pub fn create<C: Serialize, P: Serialize, K: Serialize>(
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
        persistent: bool,
    ) -> Consume {
        let channel_hashes = hash_vec(&channels);
        let channels_encoded_sorted: Vec<Vec<u8>> =
            channel_hashes.iter().map(|hash| hash.bytes()).collect();
        let hash = hash_consume(channels_encoded_sorted, patterns, continuation, persistent);
        Consume {
            channel_hashes,
            hash,
            persistent,
        }
    }
}
