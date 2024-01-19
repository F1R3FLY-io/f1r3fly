use super::instances::radix_history::EmptyRootHash;
use crate::rspace::history::roots_store::RootsStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootRepository.scala
pub struct RootRepository {
    pub roots_store: Box<dyn RootsStore>,
}

impl RootRepository {
    fn commit(&self, root: &blake3::Hash) -> () {
        self.roots_store.record_root(root)
    }

    pub fn current_root(&self) -> blake3::Hash {
        match self.roots_store.current_root() {
            None => {
                let empty_root_hash = EmptyRootHash::new();
                self.roots_store.record_root(&empty_root_hash.hash);
                empty_root_hash.hash
            }
            Some(root) => root,
        }
    }

    fn validate_and_set_current_root(&self, root: &blake3::Hash) -> () {
        match self.roots_store.validate_and_set_current_root(root) {
            Some(_) => (),
            None => panic!("unknown root"),
        }
    }
}
