use blake2::{digest::consts::U32, Blake2b, Digest};
use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

// See src/firefly/f1r3fly/rspace/src/main/scala/coop/rchain/rspace/hashing/Blake2b256Hash.scala
// The 'Hash' macro is needed here for test util function 'check_same_elements'
#[derive(Eq, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Blake2b256Hash(pub Vec<u8>);

impl Blake2b256Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Blake2b256Hash(hash.to_vec())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Blake2b256Hash(bytes)
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl Ord for Blake2b256Hash {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Blake2b256Hash {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Blake2b256Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for Blake2b256Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Blake2b256Hash({})", self.0.encode_hex::<String>())
    }
}
