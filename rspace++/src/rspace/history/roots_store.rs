use crate::rspace::shared::key_value_store::{KeyValueStore, KvStoreError};

// See rspace/src/main/scala/coop/rchain/rspace/history/RootsStore.scala
pub trait RootsStore {
    fn current_root(&self) -> Option<blake3::Hash>;

    fn validate_and_set_current_root(&self, key: &blake3::Hash) -> Option<blake3::Hash>;

    fn record_root(&self, key: &blake3::Hash) -> Result<(), KvStoreError>;
}

pub struct RootsStoreInstances;

impl RootsStoreInstances {
    pub fn roots_store(store: Box<dyn KeyValueStore>) -> impl RootsStore {
        struct RootsStoreInstance {
            store: Box<dyn KeyValueStore>,
        }

        impl RootsStore for RootsStoreInstance {
            fn current_root(&self) -> Option<blake3::Hash> {
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();
                let bytes = self.store.get_one(current_root_name);

                let maybe_decoded = match bytes {
                    Some(b) => {
                        let hash_array: [u8; 32] = match b.try_into() {
                            Ok(array) => array,
                            Err(_) => panic!("Roots_Store: Expected a Blake3 hash of length 32"),
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

            fn record_root(&self, key: &blake3::Hash) -> Result<(), KvStoreError> {
                let tag: Vec<u8> = "tag".as_bytes().to_vec();
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();

                let bytes = blake3::Hash::as_bytes(key);

                self.store.put_one(bytes.to_vec(), tag)?;
                self.store.put_one(current_root_name, bytes.to_vec())?;

                todo!()
            }
        }

        RootsStoreInstance { store }
    }
}
