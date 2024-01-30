use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    shared::key_value_store::{KeyValueStore, KvStoreError},
};

// See rspace/src/main/scala/coop/rchain/rspace/history/RootsStore.scala
pub trait RootsStore: Send + Sync {
    fn current_root(&self) -> Option<Blake3Hash>;

    fn validate_and_set_current_root(&self, key: Blake3Hash) -> Option<Blake3Hash>;

    fn record_root(&self, key: &Blake3Hash) -> Result<(), KvStoreError>;
}

pub struct RootsStoreInstances;

impl RootsStoreInstances {
    pub fn roots_store(store: Box<dyn KeyValueStore>) -> impl RootsStore {
        struct RootsStoreInstance {
            store: Box<dyn KeyValueStore>,
        }

        impl RootsStore for RootsStoreInstance {
            fn current_root(&self) -> Option<Blake3Hash> {
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();
                let bytes = self.store.get_one(current_root_name);

                let maybe_decoded = match bytes {
                    Some(b) => {
                        let hash_array: [u8; 32] = match b.try_into() {
                            Ok(array) => array,
                            Err(_) => panic!("Roots_Store: Expected a Blake3 hash of length 32"),
                        };
                        Some(Blake3Hash::new(&hash_array))
                    }
                    None => None,
                };

                maybe_decoded
            }

            fn validate_and_set_current_root(&self, key: Blake3Hash) -> Option<Blake3Hash> {
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();
                let bytes = key.bytes();

                if let Some(_) = self.store.get_one(bytes.clone()) {
                    self.store.put_one(current_root_name.to_vec(), bytes).ok()?;
                    Some(key)
                } else {
                    None
                }
            }

            fn record_root(&self, key: &Blake3Hash) -> Result<(), KvStoreError> {
                let tag: Vec<u8> = "tag".as_bytes().to_vec();
                let current_root_name: Vec<u8> = "current-root".as_bytes().to_vec();
                let bytes = key.bytes();

                self.store.put_one(bytes.to_vec(), tag)?;
                self.store.put_one(current_root_name, bytes.to_vec())?;

                Ok(())
            }
        }

        RootsStoreInstance { store }
    }
}
