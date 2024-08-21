use tiny_keccak::{Hasher, Keccak};

// See crypto/src/main/scala/coop/rchain/crypto/hash/Keccak256.scala
pub struct Keccak256;

impl Keccak256 {
    pub fn hash(input: Vec<u8>) -> Vec<u8> {
        let mut hasher = Keccak::v256();
        let mut result = vec![0u8; 32];
        hasher.update(&input);
        hasher.finalize(&mut result);
        result
    }
}
