use super::{key_value_store::KeyValueStore, key_value_store_manager::KeyValueStoreManager};
use std::{collections::BTreeMap, path::PathBuf};

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

struct LmdbDirStoreManager {
    dir_path: PathBuf,
    db_instance_mapping: BTreeMap<Db, LmdbEnvConfig>,
}

impl KeyValueStoreManager for LmdbDirStoreManager {
    fn store(&self, name: String) -> Box<dyn KeyValueStore> {
        todo!()
    }

    fn shutdown(&self) -> () {
        todo!()
    }
}
