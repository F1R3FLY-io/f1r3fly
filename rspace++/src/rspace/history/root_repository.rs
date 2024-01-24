use super::instances::radix_history::EmptyRootHash;
use crate::rspace::{history::roots_store::RootsStore, shared::key_value_store::KvStoreError};

// See rspace/src/main/scala/coop/rchain/rspace/history/RootRepository.scala
pub struct RootRepository {
    pub roots_store: Box<dyn RootsStore>,
}

impl RootRepository {
    fn commit(&self, root: &blake3::Hash) -> Result<(), KvStoreError> {
        self.roots_store.record_root(root)
    }

    pub fn current_root(&self) -> Result<blake3::Hash, KvStoreError> {
        match self.roots_store.current_root() {
            None => {
                let empty_root_hash = EmptyRootHash::new();
                self.roots_store.record_root(&empty_root_hash.hash)?;
                Ok(empty_root_hash.hash)
            }
            Some(root) => Ok(root),
        }
    }

    fn validate_and_set_current_root(&self, root: &blake3::Hash) -> () {
        match self.roots_store.validate_and_set_current_root(root) {
            Some(_) => (),
            None => panic!("unknown root"),
        }
    }
}
