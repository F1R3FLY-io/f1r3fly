// See rspace/src/main/scala/coop/rchain/rspace/merger/MergingLogic.scala

use std::collections::HashMap;

use crate::rspace::hashing::blake2b256_hash::Blake2b256Hash;

pub type NumberChannelsEndVal = HashMap<Blake2b256Hash, i64>;

pub type NumberChannelsDiff = HashMap<Blake2b256Hash, i64>;
