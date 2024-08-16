use crate::ByteString;

// See models/src/main/scala/coop/rchain/models/Validator.scala
pub type Validator = ByteString;

pub const LENGTH: usize = 65;
