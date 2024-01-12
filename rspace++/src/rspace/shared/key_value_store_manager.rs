use super::{
    key_value_store::KeyValueStore,
    lmdb_dir_store_manager::{Db, LmdbDirStoreManagerInstances, LmdbEnvConfig},
};
use crate::rspace::rspace::RSpaceStore;
use std::{collections::BTreeMap, path::PathBuf};

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreManager.scala
pub trait KeyValueStoreManager {
    fn store(&self, name: String) -> Box<dyn KeyValueStore>;

    fn shutdown(&self) -> ();
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/RholangCLI.scala
fn mk_rspace_store_manager(dir_path: PathBuf, map_size: u64) -> impl KeyValueStoreManager {
    let rspace_history_env_config = LmdbEnvConfig::new("history".to_owned(), map_size);
    let rspace_cold_env_config = LmdbEnvConfig::new("cold".to_owned(), map_size);
    let channel_env_config = LmdbEnvConfig::new("channels".to_owned(), map_size);

    let mut db_mapping = BTreeMap::new();
    db_mapping
        .insert(Db::new("rspace-history".to_string(), None), rspace_history_env_config.clone());
    db_mapping.insert(Db::new("rspace-roots".to_string(), None), rspace_history_env_config);
    db_mapping.insert(Db::new("rspace-cold".to_string(), None), rspace_cold_env_config);
    db_mapping.insert(Db::new("rspace-channels".to_string(), None), channel_env_config);

    LmdbDirStoreManagerInstances::create(dir_path, db_mapping)
}
