// See models/src/main/scala/coop/rchain/models/BlockHash.scala
pub type BlockHash = shared::rust::ByteString;
pub type BlockHashWrapper = shared::rust::BytesWrapper;

pub const LENGTH: usize = 32;
