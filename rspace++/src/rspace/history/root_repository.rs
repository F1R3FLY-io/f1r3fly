use crate::rspace::history::roots_store::RootsStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootRepository.scala
pub struct RootRepository {
    pub roots_store: Box<dyn RootsStore>,
}

impl RootRepository {
    fn commit(&self, root: blake3::Hash) -> () {
        todo!()
    }

    pub fn current_root(&self) -> blake3::Hash {
        todo!()
    }

    fn validate_and_set_current_root(&self, root: blake3::Hash) -> () {
        todo!()
    }
}
