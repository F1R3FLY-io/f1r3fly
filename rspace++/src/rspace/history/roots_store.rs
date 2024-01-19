use crate::rspace::shared::key_value_store::KeyValueStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootsStore.scala
pub trait RootsStore {
    fn current_root(&self) -> Option<blake3::Hash>;

    fn validate_and_set_current_root(&self, key: &blake3::Hash) -> Option<blake3::Hash>;

    fn record_root(&self, key: &blake3::Hash) -> ();
}

pub struct RootsStoreInstances;

impl RootsStoreInstances {
    pub fn roots_store(store: &Box<dyn KeyValueStore>) -> impl RootsStore {
        struct RootsStoreInstance;

        impl RootsStore for RootsStoreInstance {
            fn current_root(&self) -> Option<blake3::Hash> {
                todo!()
            }

            fn validate_and_set_current_root(&self, key: &blake3::Hash) -> Option<blake3::Hash> {
                todo!()
            }

            fn record_root(&self, key: &blake3::Hash) -> () {
                todo!()
            }
        }

        RootsStoreInstance
    }
}
