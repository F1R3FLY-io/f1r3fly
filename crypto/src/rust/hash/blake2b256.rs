use blake2::{digest::consts::U32, Blake2b, Digest};

// See crypto/src/main/scala/coop/rchain/crypto/hash/Blake2b256.scala
pub const HASH_LENGTH: usize = 32;

pub struct Blake2b256;

impl Blake2b256 {
    pub fn hash(input: Vec<u8>) -> Vec<u8> {
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(input);
        let hash = hasher.finalize();
        hash.to_vec()
    }
}
