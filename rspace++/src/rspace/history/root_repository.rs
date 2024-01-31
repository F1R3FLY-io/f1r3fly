use crate::rspace::{
    hashing::blake3_hash::Blake3Hash, history::roots_store::RootsStore,
    shared::key_value_store::KvStoreError,
};

use super::instances::radix_history::RadixHistory;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootRepository.scala
pub struct RootRepository {
    pub roots_store: Box<dyn RootsStore>,
}

impl RootRepository {
    pub fn commit(&self, root: &Blake3Hash) -> Result<(), KvStoreError> {
        self.roots_store.record_root(root)
    }

    pub fn current_root(&self) -> Result<Blake3Hash, KvStoreError> {
        match self.roots_store.current_root() {
            None => {
                let empty_root_hash = RadixHistory::empty_root_hash();
                self.roots_store.record_root(&empty_root_hash)?;
                Ok(empty_root_hash)
            }
            Some(root) => Ok(root),
        }
    }

    pub fn validate_and_set_current_root(&self, root: Blake3Hash) -> () {
        match self.roots_store.validate_and_set_current_root(root) {
            Some(_) => (),
            None => panic!("unknown root"),
        }
    }
}
