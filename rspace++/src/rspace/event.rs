use crate::rspace::hashing::stable_hash_provider::*;
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/trace/Event.scala
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Produce {
    pub channel_hash: blake3::Hash,
    pub hash: blake3::Hash,
    pub persistent: bool,
}

impl Produce {
    pub fn create<C: Serialize, A: Serialize>(channel: C, datum: A, persistent: bool) -> Produce {
        let channel_hash = hash_channel(channel);
        let hash = hash_produce(channel_hash.as_bytes().to_vec(), datum, persistent);
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
    pub channel_hashes: Vec<blake3::Hash>,
    pub hash: blake3::Hash,
    pub persistent: bool,
}

impl Consume {
    pub fn create<C: Serialize, P: Serialize, K: Serialize>(
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
        persistent: bool,
    ) -> Consume {
        let channel_hashes = hash_channels(channels);
        let channels_encoded_sorted: Vec<Vec<u8>> = channel_hashes
            .iter()
            .map(|hash| hash.as_bytes().to_vec())
            .collect();
        let hash = hash_consume(channels_encoded_sorted, patterns, continuation, persistent);
        Consume {
            channel_hashes,
            hash,
            persistent,
        }
    }
}
