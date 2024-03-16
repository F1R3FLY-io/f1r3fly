use super::{
    key_value_store::KeyValueStore, key_value_store_manager::KeyValueStoreManager,
    lmdb_key_value_store::LmdbKeyValueStore,
};
use async_trait::async_trait;
use futures::channel::oneshot;
use heed::{types::SerdeBincode, Database, Env, EnvOpenOptions};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

// See shared/src/main/scala/coop/rchain/store/LmdbStoreManager.scala
pub struct LmdbStoreManager {
    dir_path: PathBuf,
    max_env_size: usize,
    env_sender: Option<oneshot::Sender<Env>>,
    env_receiver: Option<oneshot::Receiver<Env>>,
    dbs: Arc<Mutex<HashMap<String, DbEnv>>>,
}

#[derive(Clone)]
struct DbEnv {
    env: Env,
    db: Database<SerdeBincode<Vec<u8>>, SerdeBincode<Vec<u8>>>,
}

impl LmdbStoreManager {
    pub fn new(dir_path: PathBuf, max_env_size: usize) -> Box<dyn KeyValueStoreManager> {
        let (sender, receiver) = oneshot::channel::<Env>();
        Box::new(LmdbStoreManager {
            dir_path,
            max_env_size,
            env_sender: Some(sender),
            env_receiver: Some(receiver),
            dbs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn get_current_env(&mut self, db_name: &str) -> Result<DbEnv, heed::Error> {
        let dbs = self.dbs.lock().await;
        if let Some(db_env) = dbs.get(db_name) {
            return Ok(db_env.clone());
        }
        drop(dbs);

        // Create and open the environment if it doesn't exist
        if self.env_sender.is_some() {
            let env = self.create_env().await?;
            let sender = self.env_sender.take().ok_or_else(|| {
                heed::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "LMDB_Store_Manager: LMDB environment sender unavailable",
                ))
            })?;
            let _ = sender.send(env); // Send the environment to the receiver
        }

        // Await the environment from the receiver
        let receiver = self.env_receiver.take().ok_or_else(|| {
            heed::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "LMDB_Store_Manager: LMDB environment receiver unavailable",
            ))
        })?;
        let env = receiver.await.map_err(|_| {
            heed::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "LMDB_Store_Manager: LMDB environment was not received",
            ))
        })?;

        // Open or create the database
        let db = env.create_database(Some(db_name))?;

        let mut dbs = self.dbs.lock().await;
        let db_env = DbEnv { env, db };
        dbs.insert(db_name.to_string(), db_env.clone());
        Ok(db_env)
    }

    async fn create_env(&self) -> Result<Env, heed::Error> {
        // println!("Creating LMDB environment: {:?}", self.dir_path);
        fs::create_dir_all(&self.dir_path)?;

        let mut env_builder = EnvOpenOptions::new();
        env_builder.map_size(self.max_env_size);
        env_builder.max_dbs(20);
        env_builder.max_readers(2048);

        let env = env_builder.open(&self.dir_path)?;
        Ok(env)
    }
}

#[async_trait]
impl KeyValueStoreManager for LmdbStoreManager {
    async fn store(&mut self, name: String) -> Result<Box<dyn KeyValueStore>, heed::Error> {
        let db_env = self.get_current_env(&name).await?;
        Ok(Box::new(LmdbKeyValueStore::new(db_env.env, db_env.db)))
    }

    async fn shutdown(&mut self) -> Result<(), heed::Error> {
        // Clear the databases HashMap to drop all DbEnv references
        let mut dbs = self.dbs.lock().await;
        dbs.clear();

        // If there is an active receiver awaiting the environment, receive it and drop it
        if let Some(receiver) = self.env_receiver.take() {
            let env = receiver.await.map_err(|_| {
                heed::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "LMDB_Store_Manager: Failed to receive LMDB environment for shutdown",
                ))
            })?;
            drop(env);
        }

        Ok(())
    }
}
