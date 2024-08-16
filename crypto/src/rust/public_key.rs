use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

// See crypto/src/main/scala/coop/rchain/crypto/PublicKey.scala
#[derive(Debug, Clone)]
pub struct PublicKey {
    bytes: Vec<u8>,
}

impl PublicKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        PublicKey { bytes }
    }

    pub fn from_bytes(bs: &[u8]) -> Self {
        PublicKey::new(bs.to_vec())
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}
