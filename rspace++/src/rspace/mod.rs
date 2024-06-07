pub mod event;
pub mod hashing;
pub mod history;
pub mod hot_store;
pub mod internal;
pub mod matcher;
pub mod rspace;
mod space_matcher;
pub mod checkpoint;
pub mod hot_store_action;
pub mod shared;
mod hot_store_trie_action;
pub mod state;
mod serializers;

pub type ByteVector = Vec<u8>;
pub type ByteBuffer = Vec<u8>;
pub type Byte = u8;
