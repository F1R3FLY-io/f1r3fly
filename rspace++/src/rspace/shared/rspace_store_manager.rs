use heed::EnvOpenOptions;

use super::key_value_store_manager::KeyValueStoreManager;
use crate::rspace::rspace::RSpaceStore;
use crate::rspace::shared::lmdb_dir_store_manager::{Db, LmdbDirStoreManager, LmdbEnvConfig};
use crate::rspace::shared::lmdb_key_value_store::LmdbKeyValueStore;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/RholangCLI.scala
pub fn mk_rspace_store_manager(dir_path: PathBuf, map_size: usize) -> impl KeyValueStoreManager {
    let rspace_history_env_config = LmdbEnvConfig::new("history".to_owned(), map_size);
    let rspace_cold_env_config = LmdbEnvConfig::new("cold".to_owned(), map_size);
    let channel_env_config = LmdbEnvConfig::new("channels".to_owned(), map_size);

    let mut db_mapping = HashMap::new();
    db_mapping
        .insert(Db::new("rspace-history".to_string(), None), rspace_history_env_config.clone());
    db_mapping.insert(Db::new("rspace-roots".to_string(), None), rspace_history_env_config);
    db_mapping.insert(Db::new("rspace-cold".to_string(), None), rspace_cold_env_config);
    db_mapping.insert(Db::new("rspace-channels".to_string(), None), channel_env_config);

    LmdbDirStoreManager::new(dir_path, db_mapping)
}

pub fn get_or_create_rspace_store(lmdb_path: String, map_size: usize) -> RSpaceStore {
    if Path::new(&lmdb_path).exists() {
        println!("RSpace++ storage path {} already exists.", lmdb_path);

        let history_store =
            open_lmdb_store(lmdb_path.clone(), "rspace++_history".to_string()).unwrap();
        let roots_store = open_lmdb_store(lmdb_path.clone(), "rspace++_roots".to_string()).unwrap();
        let cold_store = open_lmdb_store(lmdb_path, "rspace++_cold".to_string()).unwrap();

        let rspace_store = RSpaceStore {
            history: Arc::new(Mutex::new(Box::new(history_store))),
            roots: Arc::new(Mutex::new(Box::new(roots_store))),
            cold: Arc::new(Mutex::new(Box::new(cold_store))),
        };

        rspace_store
    } else {
        println!("RSpace++ storage path: {} does not exist, creating a new one.", lmdb_path);
        create_dir_all(lmdb_path.clone()).expect("Failed to create RSpace++ storage directory");

        let history_store =
            create_lmdb_store(lmdb_path.clone(), "rspace++_history".to_string(), map_size).unwrap();
        let roots_store =
            create_lmdb_store(lmdb_path.clone(), "rspace++_roots".to_string(), map_size).unwrap();
        let cold_store =
            create_lmdb_store(lmdb_path, "rspace++_cold".to_string(), map_size).unwrap();

        let rspace_store = RSpaceStore {
            history: Arc::new(Mutex::new(Box::new(history_store))),
            roots: Arc::new(Mutex::new(Box::new(roots_store))),
            cold: Arc::new(Mutex::new(Box::new(cold_store))),
        };

        rspace_store
    }
}

pub fn close_rspace_store(rspace_store: RSpaceStore) {
    drop(rspace_store);
}

fn create_lmdb_store(
    lmdb_path: String,
    db_name: String,
    max_env_size: usize,
) -> Result<LmdbKeyValueStore, heed::Error> {
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(max_env_size);
    env_builder.max_dbs(20);
    env_builder.max_readers(2048);

    let env = env_builder.open(&lmdb_path)?;
    let db = env.create_database(Some(&db_name))?;

    Ok(LmdbKeyValueStore::new(env, db))
}

fn open_lmdb_store(lmdb_path: String, db_name: String) -> Result<LmdbKeyValueStore, heed::Error> {
    let env = EnvOpenOptions::new().open(lmdb_path)?;
    let db = env.open_database(Some(&db_name))?;
    match db {
        Some(open_db) => Ok(LmdbKeyValueStore {
            env: env.into(),
            db: Arc::new(Mutex::new(open_db)),
        }),
        None => panic!("\nUnable to open database."),
    }
}
