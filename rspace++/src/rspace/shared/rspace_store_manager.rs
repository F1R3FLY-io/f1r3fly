use super::key_value_store_manager::KeyValueStoreManager;
use crate::rspace::shared::lmdb_dir_store_manager::{Db, LmdbDirStoreManager, LmdbEnvConfig};
use std::collections::HashMap;
use std::path::PathBuf;

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
