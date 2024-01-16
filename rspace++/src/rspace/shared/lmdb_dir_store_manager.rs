use super::{
    key_value_store::KeyValueStore, key_value_store_manager::KVSManagerError,
    key_value_store_manager::KeyValueStoreManager,
};
use async_trait::async_trait;
use futures::channel::oneshot;
use std::sync::Arc;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::sync::Mutex;

// See shared/src/main/scala/coop/rchain/store/LmdbDirStoreManager.scala
pub struct LmdbDirStoreManagerInstances;

impl LmdbDirStoreManagerInstances {
    pub fn create(
        dir_path: PathBuf,
        db_instance_mapping: BTreeMap<Db, LmdbEnvConfig>,
    ) -> impl KeyValueStoreManager {
        LmdbDirStoreManager {
            dir_path,
            db_instance_mapping,
            managers_state: Arc::new(Mutex::new(StoreState {
                envs: BTreeMap::new(),
            })),
        }
    }
}

/**
 * Specification for LMDB database: unique identifier and database name
 *
 * @param id unique identifier
 * @param nameOverride name to use as database name instead of [[id]]
 */
#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Db {
    id: String,
    name_override: Option<String>,
}

impl Db {
    pub fn new(id: String, name_override: Option<String>) -> Self {
        Db { id, name_override }
    }
}

// Mega, giga and tera bytes
const MB: u64 = 1024 * 1024;
const GB: u64 = 1024 * MB;
const TB: u64 = 1024 * GB;

#[derive(Clone)]
pub struct LmdbEnvConfig {
    pub name: String,
    pub max_env_size: u64,
}

impl LmdbEnvConfig {
    pub fn new(name: String, max_env_size: u64) -> Self {
        LmdbEnvConfig { name, max_env_size }
    }
}

// The idea for this class is to manage multiple of key-value lmdb databases.
// For LMDB this allows control which databases are part of the same environment (file).
struct LmdbDirStoreManager {
    dir_path: PathBuf,
    db_instance_mapping: BTreeMap<Db, LmdbEnvConfig>,
    managers_state: Arc<Mutex<StoreState>>,
}

struct StoreState {
    envs: BTreeMap<String, oneshot::Receiver<Box<dyn KeyValueStoreManager>>>,
}

#[async_trait]
impl KeyValueStoreManager for LmdbDirStoreManager {
    async fn store(&self, db_name: String) -> Result<Box<dyn KeyValueStore>, KVSManagerError> {
        todo!()
    }

    fn shutdown(&self) -> () {
        todo!()
    }
}
