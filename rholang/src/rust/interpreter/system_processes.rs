use std::sync::{Arc, RwLock};

use crypto::rust::public_key::PublicKey;
use models::rhoapi::Par;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/SystemProcesses.scala
pub struct InvalidBlocks {
    invalid_blocks: Arc<RwLock<Par>>,
}
impl InvalidBlocks {
    pub fn set_params(&self, invalid_blocks: Par) -> () {
        let mut lock = self.invalid_blocks.write().unwrap();

        *lock = invalid_blocks;
    }
}

pub struct BlockData {
    time_stamp: i64,
    block_number: i64,
    sender: PublicKey,
    seq_num: i32,
}
