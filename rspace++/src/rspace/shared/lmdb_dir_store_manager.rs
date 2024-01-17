use super::lmdb_store_manager::LmdbStoreManager;
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
            db_mapping: db_instance_mapping,
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
const MB: i64 = 1024 * 1024;
const GB: i64 = 1024 * MB;
const TB: i64 = 1024 * GB;

#[derive(Clone)]
pub struct LmdbEnvConfig {
    pub name: String,
    pub max_env_size: i64,
}

impl LmdbEnvConfig {
    pub fn new(name: String, max_env_size: i64) -> Self {
        LmdbEnvConfig { name, max_env_size }
    }
}

// The idea for this class is to manage multiple of key-value lmdb databases.
// For LMDB this allows control which databases are part of the same environment (file).
struct LmdbDirStoreManager {
    dir_path: PathBuf,
    db_mapping: BTreeMap<Db, LmdbEnvConfig>,
    managers_state: Arc<Mutex<StoreState>>,
}

struct StoreState {
    envs: BTreeMap<String, oneshot::Receiver<Box<dyn KeyValueStoreManager>>>,
}

#[async_trait]
impl KeyValueStoreManager for LmdbDirStoreManager {
    async fn store(&self, db_name: String) -> Result<Box<dyn KeyValueStore>, KVSManagerError> {
        let db_instance_mapping: BTreeMap<&String, (&Db, &LmdbEnvConfig)> = self
            .db_mapping
            .iter()
            .map(|(db, cfg)| (&db.id, (db, cfg)))
            .collect();

        let (sender, receiver) = oneshot::channel::<Box<dyn KeyValueStoreManager>>();

        let action = {
            let mut state = self.managers_state.lock().await;

            let (db, cfg) = db_instance_mapping.get(&db_name).ok_or(KVSManagerError {
                message: format!("LMDB_Dir_Store_Manager: Key {} was not found", db_name),
            })?;
            let man_name = cfg.name.to_string();

            let is_new = !state.envs.contains_key(&man_name);
            if is_new {
                state.envs.insert(man_name.to_string(), receiver);
            }

            (is_new, db, cfg)
        };
        let (is_new, db, man_cfg) = action;

        if is_new {
            self.create_lmdb_manager(man_cfg, sender)?;
        }

        let manager = {
            let mut state = self.managers_state.lock().await;
            let receiver = state.envs.get_mut(&man_cfg.name).ok_or(KVSManagerError {
                message: format!("LMDB_Dir_Store_Manager: Receiver not found"),
            })?;
            receiver.await.map_err(|_| KVSManagerError {
                message: format!("LMDB_Dir_Store_Manager: Failed to receive manager"),
            })?
        };

        let database_name = db.name_override.clone().unwrap_or(db.id.clone());
        let database = manager.store(database_name);

        database.await
    }

    async fn shutdown(&self) -> () {
        let mut state = self.managers_state.lock().await;
        for manager_future in state.envs.values_mut() {
            if let Ok(manager) = manager_future.await {
                manager.shutdown().await;
            }
        }
    }
}

impl LmdbDirStoreManager {
    fn create_lmdb_manager(
        &self,
        config: &LmdbEnvConfig,
        sender: oneshot::Sender<Box<dyn KeyValueStoreManager>>,
    ) -> Result<(), KVSManagerError> {
        let manager: Box<dyn KeyValueStoreManager> =
            LmdbStoreManager::new(self.dir_path.join(&config.name), config.max_env_size);
        sender.send(manager).map_err(|_| KVSManagerError {
            message: "LMDB_Dir_Store_Manager: Failed to send managerd".to_string(),
        })?;
        Ok(())
    }
}
