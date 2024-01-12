use super::{key_value_store::KeyValueStore, key_value_store_manager::KeyValueStoreManager};
use crate::rspace::rspace::RSpaceStore;
use crate::rspace::shared::lmdb_dir_store_manager::{
    Db, LmdbDirStoreManagerInstances, LmdbEnvConfig,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

// See rspace/src/main/scala/coop/rchain/rspace/store/RSpaceStoreManagerSyntax.scala
struct RSpaceStoreManagerOps;

impl RSpaceStoreManagerOps {
    fn r_space_stores(&self) -> RSpaceStore {
        self.get_stores("rspace")
    }

    fn eval_stores(&self) -> RSpaceStore {
        self.get_stores("eval")
    }

    // TODO: This function should be private
    fn get_stores(&self, db_prefix: &str) -> RSpaceStore {
        let history = self.store(format!("{}-history", db_prefix));
        let roots = self.store(format!("{}-roots", db_prefix));
        let cold = self.store(format!("{}-cold", db_prefix));

        RSpaceStore {
            history,
            roots,
            cold,
        }
    }
}

impl KeyValueStoreManager for RSpaceStoreManagerOps {
    fn store(&self, name: String) -> Box<dyn KeyValueStore> {
        todo!()
    }

    fn shutdown(&self) -> () {
        todo!()
    }
}

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/RholangCLI.scala
pub fn mk_rspace_store_manager(dir_path: PathBuf, map_size: u64) -> impl KeyValueStoreManager {
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
