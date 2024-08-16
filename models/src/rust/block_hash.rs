use crate::ByteString;

// See models/src/main/scala/coop/rchain/models/BlockHash.scala
pub type BlockHash = ByteString;

pub const LENGTH: usize = 32;
