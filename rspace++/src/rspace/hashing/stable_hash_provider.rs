use bincode;
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/hashing/StableHashProvider.scala
pub fn hash_channels<C: Serialize>(channels: Vec<C>) -> Vec<blake3::Hash> {
    let mut hashes = channels
        .into_iter()
        .map(|channel| {
            let bytes = bincode::serialize(&channel).unwrap();
            blake3::hash(&bytes)
        })
        .collect::<Vec<_>>();
    hashes.sort_by(|a, b| a.as_bytes().cmp(&b.as_bytes()));
    hashes
}

pub fn hash_channel<C: Serialize>(channel: C) -> blake3::Hash {
    let bytes = bincode::serialize(&channel).unwrap();
    blake3::hash(&bytes)
}

// See rspace/src/main/scala/coop/rchain/rspace/hashing/StableHashProvider.scala
pub fn hash_consume<P: Serialize, K: Serialize>(
    encoded_channels: Vec<Vec<u8>>,
    patterns: Vec<P>,
    continuation: K,
    persist: bool,
) -> blake3::Hash {
    let mut encoded_patterns = patterns
        .into_iter()
        .map(|pattern| bincode::serialize(&pattern).unwrap())
        .collect::<Vec<_>>();
    encoded_patterns.sort();

    let encoded_continuation = bincode::serialize(&continuation).unwrap();
    let encoded_persist = bincode::serialize(&persist).unwrap();

    let mut encoded_vec = encoded_channels;
    encoded_vec.extend(encoded_patterns);
    encoded_vec.push(encoded_continuation);
    encoded_vec.push(encoded_persist);

    let encoded = bincode::serialize(&encoded_vec).unwrap();
    blake3::hash(&encoded)
}

pub fn hash_produce<A: Serialize>(
    encoded_channel: Vec<u8>,
    datum: A,
    persist: bool,
) -> blake3::Hash {
    let encoded_datum = bincode::serialize(&datum).unwrap();
    let encoded_persist = bincode::serialize(&persist).unwrap();

    let encoded_vec = vec![encoded_channel, encoded_datum, encoded_persist];

    let encoded = bincode::serialize(&encoded_vec).unwrap();
    blake3::hash(&encoded)
}
