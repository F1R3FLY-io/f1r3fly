use super::key_value_store::{KeyValueStore, KvStoreError};
use crate::rspace::ByteBuffer;
use heed::types::SerdeBincode;
use heed::{Database, Env};
use std::sync::{Arc, Mutex};

pub struct LmdbKeyValueStore {
    env: Arc<Env>,
    db: Arc<Mutex<Database<SerdeBincode<ByteBuffer>, SerdeBincode<ByteBuffer>>>>,
}

impl KeyValueStore for LmdbKeyValueStore {
    fn get(&self, keys: Vec<ByteBuffer>) -> Result<Vec<Option<ByteBuffer>>, KvStoreError> {
        let db = self
            .db
            .lock()
            .expect("LMDB Key Value Store: Failed to acquire lock on db");
        let reader = self.env.read_txn()?;
        let results = keys
            .into_iter()
            .map(|key| db.get(&reader, &key).map_err(|e| e.into()))
            .collect();
        reader.commit()?;
        results
    }

    fn put(&self, kv_pairs: Vec<(ByteBuffer, ByteBuffer)>) -> Result<(), KvStoreError> {
        let db = self
            .db
            .lock()
            .expect("LMDB Key Value Store: Failed to acquire lock on db");
        let mut writer = self.env.write_txn()?;
        for (key, value) in kv_pairs {
            db.put(&mut writer, &key, &value)?;
        }
        writer.commit()?;
        Ok(())
    }

    fn delete(&self, keys: Vec<ByteBuffer>) -> Result<usize, KvStoreError> {
        let db = self
            .db
            .lock()
            .expect("LMDB Key Value Store: Failed to acquire lock on db");
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

    fn iterate(&self, f: fn(ByteBuffer, ByteBuffer)) -> Result<(), KvStoreError> {
        let db = self
            .db
            .lock()
            .expect("LMDB Key Value Store: Failed to acquire lock on db");
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
        Box::new(self.clone())
    }
}

impl LmdbKeyValueStore {
    pub fn new(env: Env, db: Database<SerdeBincode<ByteBuffer>, SerdeBincode<ByteBuffer>>) -> Self {
        let env_arc = Arc::new(env);
        let db_arc = Arc::new(Mutex::new(db));

        LmdbKeyValueStore {
            env: env_arc,
            db: db_arc,
        }
    }
}

impl Clone for LmdbKeyValueStore {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            env: self.env.clone(),
        }
    }
}
