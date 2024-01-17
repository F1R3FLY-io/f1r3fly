use super::key_value_store::KeyValueStore;
use async_trait::async_trait;
use heed::types::SerdeBincode;
use heed::{Database, Env};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LmdbKeyValueStore {
    env: Arc<Env>,
    db: Arc<Mutex<Database<SerdeBincode<Vec<u8>>, SerdeBincode<Vec<u8>>>>>,
}

#[async_trait]
impl KeyValueStore for LmdbKeyValueStore {
    async fn get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>, heed::Error> {
        let db = self.db.lock().await;
        let reader = self.env.read_txn()?;
        let results = keys
            .into_iter()
            .map(|key| db.get(&reader, &key).map_err(|e| e.into()))
            .collect();
        reader.commit()?;
        results
    }

    async fn put(&self, kv_pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), heed::Error> {
        let db = self.db.lock().await;
        let mut writer = self.env.write_txn()?;
        for (key, value) in kv_pairs {
            db.put(&mut writer, &key, &value)?;
        }
        writer.commit()?;
        Ok(())
    }

    async fn delete(&self, keys: Vec<Vec<u8>>) -> Result<usize, heed::Error> {
        let db = self.db.lock().await;
        let mut writer = self.env.write_txn()?;
        let mut delete_count = 0;
        for key in &keys {
            if db.delete(&mut writer, key)? {
                delete_count += 1;
            }
        }
        writer.commit()?;
        Ok(delete_count)
    }

    async fn iterate(&self, f: fn(Vec<u8>, Vec<u8>)) -> Result<(), heed::Error> {
        let db = self.db.lock().await;
        let reader = self.env.read_txn()?;
        let iter = db.iter(&reader)?;
        for result in iter {
            let (key, value) = result?;
            f(key.to_vec(), value);
        }
        reader.commit()?;
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn KeyValueStore> {
        todo!()
    }
}

impl LmdbKeyValueStore {
    pub fn new(env: Env, db: Database<SerdeBincode<Vec<u8>>, SerdeBincode<Vec<u8>>>) -> Self {
        let env_arc = Arc::new(env);
        let db_arc = Arc::new(Mutex::new(db));

        LmdbKeyValueStore {
            env: env_arc,
            db: db_arc,
        }
    }
}
