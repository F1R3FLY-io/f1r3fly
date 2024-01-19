use crate::rspace::shared::key_value_store::{KeyValueStore, KvStoreError};
use async_trait::async_trait;

// See rspace/src/main/scala/coop/rchain/rspace/history/RootsStore.scala
#[async_trait]
pub trait RootsStore {
    async fn current_root(&self) -> Option<blake3::Hash>;

    fn validate_and_set_current_root(&self, key: &blake3::Hash) -> Option<blake3::Hash>;

    fn record_root(&self, key: &blake3::Hash) -> ();
}

pub struct RootsStoreInstances;

impl RootsStoreInstances {
    pub fn roots_store(store: Box<dyn KeyValueStore>) -> impl RootsStore {
        struct RootsStoreInstance {
            store: Box<dyn KeyValueStore>,
        }

        #[async_trait]
        impl RootsStore for RootsStoreInstance {
            async fn current_root(&self) -> Option<blake3::Hash> {
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();
                let bytes = self.store.get_one(current_root_name).await;

                let maybe_decoded = match bytes {
                    Some(b) => {
                        let hash_array: [u8; 32] = match b.try_into() {
                            Ok(array) => array,
                            Err(_) => panic!("Expected a Blake3 hash of length 32"),
                        };
                        Some(blake3::Hash::from(hash_array))
                    }
                    None => None,
                };

                maybe_decoded
            }

            fn validate_and_set_current_root(&self, key: &blake3::Hash) -> Option<blake3::Hash> {
                todo!()
            }

            fn record_root(&self, key: &blake3::Hash) -> () {
                todo!()
            }
        }

        RootsStoreInstance { store }
    }
}
