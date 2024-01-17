use super::{
    key_value_store::KeyValueStore,
    key_value_store_manager::{KVSManagerError, KeyValueStoreManager},
};
use async_trait::async_trait;
use futures::channel::oneshot;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

// See shared/src/main/scala/coop/rchain/store/LmdbStoreManager.scala
pub struct LmdbStoreManager {
    dir_path: PathBuf,
    max_env_size: i64,
    var_state: Arc<Mutex<DbState>>,
}

enum EnvRefStatus {
    EnvClosed,
    EnvStarting,
    EnvOpen,
    EnvClosing,
}

struct DbState {
    status: EnvRefStatus,
    in_progress: i32,
    env_rec: oneshot::Receiver<heed::Env>,
    dbs: BTreeMap<String, oneshot::Receiver<heed::Env>>,
}

impl LmdbStoreManager {
    pub fn new(dir_path: PathBuf, max_env_size: i64) -> Box<dyn KeyValueStoreManager> {
        let (_, receiver) = oneshot::channel::<heed::Env>();
        Box::new(LmdbStoreManager {
            dir_path,
            max_env_size,
            var_state: Arc::new(Mutex::new(DbState {
                status: EnvRefStatus::EnvClosed,
                in_progress: 0,
                env_rec: receiver,
                dbs: BTreeMap::default(),
            })),
        })
    }
}

#[async_trait]
impl KeyValueStoreManager for LmdbStoreManager {
    async fn store(&self, name: String) -> Result<Box<dyn KeyValueStore>, KVSManagerError> {
        todo!()
    }

    async fn shutdown(&self) -> () {
        todo!()
    }
}
