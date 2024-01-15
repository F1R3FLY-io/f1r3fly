use super::{key_value_store::KeyValueStore, key_value_store_manager::KeyValueStoreManager};
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
    // fn store(&self, name: String) -> Box<dyn KeyValueStore> {
    //     let mapping = &self.db_instance_mapping;
    //     let db_instance_mapping: BTreeMap<&String, (&Db, &LmdbEnvConfig)> = mapping
    //         .into_iter()
    //         .map(|(db, cfg)| (&db.id, (db, cfg)))
    //         .collect();

    //     let (sender, receiver) = oneshot::channel::<Box<dyn KeyValueStoreManager>>();

    //     todo!()
    // }

    async fn store(&self, db_name: String) -> Box<dyn KeyValueStore> {
        let mapping = &self.db_instance_mapping;
        let db_instance_mapping: BTreeMap<&String, (&Db, &LmdbEnvConfig)> = mapping
            .into_iter()
            .map(|(db, cfg)| (&db.id, (db, cfg)))
            .collect();

        let (sender, receiver) = oneshot::channel::<Box<dyn KeyValueStoreManager>>();
        let mut state = self.managers_state.lock().await;

        let (db, cfg) = self.db_instance_mapping.get(&db_name).unwrap(); // Simplified for example purposes
        let man_name = &cfg.name;

        let (is_new, receiver) = match state.envs.get(man_name) {
            Some(receiver) => (false, receiver.clone()),
            None => {
                state.envs.insert(man_name.clone(), new_receiver);
                (true, new_receiver)
            }
        };

        drop(state); // Explicitly drop the lock

        if is_new {
            create_lmdb_manager(cfg.clone(), new_sender).await;
        }

        let manager = receiver.await.unwrap(); // Simplified error handling
        let database_name = db.name_override.unwrap_or_else(|| db.id.clone());
        let database = manager.store(database_name).await; // Assuming store is an async method

        database
    }

    fn shutdown(&self) -> () {
        todo!()
    }
}
