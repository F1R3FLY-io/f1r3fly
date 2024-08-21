use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

// See crypto/src/main/scala/coop/rchain/crypto/PrivateKey.scala
#[derive(Debug, Clone)]
pub struct PrivateKey {
    pub bytes: Vec<u8>,
}

impl PrivateKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        PrivateKey { bytes }
    }

    pub fn from_bytes(bs: &[u8]) -> Self {
        PrivateKey::new(bs.to_vec())
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Hash for PrivateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}
