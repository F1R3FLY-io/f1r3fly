use blake3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableBlake3Hash(Vec<u8>);

impl SerializableBlake3Hash {
    pub fn new(hash: blake3::Hash) -> Self {
        SerializableBlake3Hash(hash.as_bytes().to_vec())
    }
}
