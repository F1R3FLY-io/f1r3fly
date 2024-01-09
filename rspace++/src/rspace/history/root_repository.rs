use crate::rspace::history::roots_store::RootsStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootRepository.scala
pub struct RootRepository<T: RootsStore> {
    pub roots_store: T,
}

impl<T: RootsStore> RootRepository<T> {
    fn commit(&self, root: blake3::Hash) -> () {
        todo!()
    }

    fn current_root(&self) -> blake3::Hash {
        todo!()
    }

    fn validate_and_set_current_root(&self, root: blake3::Hash) -> () {
        todo!()
    }
}
