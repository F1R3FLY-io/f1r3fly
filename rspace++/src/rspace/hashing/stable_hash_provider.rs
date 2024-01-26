use super::blake3_hash::Blake3Hash;
use bincode;
use serde::Serialize;

// See rspace/src/main/scala/coop/rchain/rspace/hashing/StableHashProvider.scala
pub fn hash<C: Serialize>(channel: &C) -> Blake3Hash {
    let bytes = bincode::serialize(channel).unwrap();
    Blake3Hash::new(&bytes)
}

// TODO: Double check the sorting here against scala side
pub fn hash_vec<C: Serialize>(channels: &Vec<C>) -> Vec<Blake3Hash> {
    let mut hashes = channels
        .into_iter()
        .map(|channel| {
            let bytes = bincode::serialize(&channel).unwrap();
            Blake3Hash::new(&bytes)
        })
        .collect::<Vec<_>>();
    hashes.sort();
    hashes
}

pub fn hash_from_vec<C: Serialize>(channels: &Vec<C>) -> Blake3Hash {
    hash_from_hashes(hash_vec(channels))
}

// TODO: Double check the sorting here against scala side
pub fn hash_from_hashes(channels_hashes: Vec<Blake3Hash>) -> Blake3Hash {
    let mut ord_hashes = channels_hashes;
    ord_hashes.sort();
    let concatenated: Vec<u8> = ord_hashes.into_iter().flat_map(|h| h.0.clone()).collect();
    Blake3Hash::new(&concatenated)
}

// See rspace/src/main/scala/coop/rchain/rspace/hashing/StableHashProvider.scala
pub fn hash_consume<P: Serialize, K: Serialize>(
    encoded_channels: Vec<Vec<u8>>,
    patterns: Vec<P>,
    continuation: K,
    persist: bool,
) -> Blake3Hash {
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
    Blake3Hash::new(&encoded)
}

pub fn hash_produce<A: Serialize>(encoded_channel: Vec<u8>, datum: A, persist: bool) -> Blake3Hash {
    let encoded_datum = bincode::serialize(&datum).unwrap();
    let encoded_persist = bincode::serialize(&persist).unwrap();

    let encoded_vec = vec![encoded_channel, encoded_datum, encoded_persist];

    let encoded = bincode::serialize(&encoded_vec).unwrap();
    Blake3Hash::new(&encoded)
}
