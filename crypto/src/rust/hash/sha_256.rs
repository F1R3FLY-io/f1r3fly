use sha2::{Digest, Sha256};

/**
* Sha256 hashing algorithm
*/
// See crypto/src/main/scala/coop/rchain/crypto/hash/Sha256.scala
pub struct Sha256Hasher;

impl Sha256Hasher {
    pub fn hash(input: Vec<u8>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.finalize().to_vec()
    }
}
