use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    shared::key_value_store::{KeyValueStore, KvStoreError},
    ByteBuffer,
};

// See rspace/src/main/scala/coop/rchain/rspace/history/RootsStore.scala
pub trait RootsStore: Send + Sync {
    fn current_root(&self) -> Result<Option<Blake3Hash>, RootError>;

    fn validate_and_set_current_root(
        &self,
        key: Blake3Hash,
    ) -> Result<Option<Blake3Hash>, RootError>;

    fn record_root(&self, key: &Blake3Hash) -> Result<(), RootError>;
}

pub struct RootsStoreInstances;

impl RootsStoreInstances {
    pub fn roots_store(store: Box<dyn KeyValueStore>) -> impl RootsStore {
        struct RootsStoreInstance {
            store: Box<dyn KeyValueStore>,
        }

        impl RootsStore for RootsStoreInstance {
            fn current_root(&self) -> Result<Option<Blake3Hash>, RootError> {
                let current_root_name: ByteBuffer = "current-root".as_bytes().to_vec();
                let bytes = self.store.get_one(current_root_name)?;

                let maybe_decoded = match bytes {
                    Some(b) => {
                        let hash_array: [u8; 32] = match b.try_into() {
                            Ok(array) => array,
                            Err(_) => panic!("Roots Store: Expected a Blake3 hash of length 32"),
                        };
                        Some(Blake3Hash::new(&hash_array))
                    }
                    None => None,
                };

                Ok(maybe_decoded)
            }

            fn validate_and_set_current_root(
                &self,
                key: Blake3Hash,
            ) -> Result<Option<Blake3Hash>, RootError> {
                let current_root_name: ByteBuffer = "current-root".as_bytes().to_vec();
                let bytes = key.bytes();

                if let Some(_) = self.store.get_one(bytes.clone())? {
                    self.store.put_one(current_root_name, bytes)?;
                    Ok(Some(key))
                } else {
                    Ok(None)
                }
            }

            fn record_root(&self, key: &Blake3Hash) -> Result<(), RootError> {
                let tag: ByteBuffer = "tag".as_bytes().to_vec();
                let current_root_name: ByteBuffer = "current-root".as_bytes().to_vec();
                let bytes = key.bytes();

                self.store.put_one(bytes.to_vec(), tag)?;
                self.store.put_one(current_root_name, bytes.to_vec())?;

                Ok(())
            }
        }

        RootsStoreInstance { store }
    }
}

#[derive(Debug)]
pub enum RootError {
    KvStoreError(String),
    UnknownRootError(String),
}

impl std::fmt::Display for RootError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RootError::KvStoreError(err) => write!(f, "Key Value Store Error: {}", err),
            RootError::UnknownRootError(err) => write!(f, "Unknown root: {}", err),
        }
    }
}

impl From<KvStoreError> for RootError {
    fn from(error: KvStoreError) -> Self {
        RootError::KvStoreError(error.to_string())
    }
}
