// See casper/src/test/scala/coop/rchain/casper/util/rholang/Resources.scala

use std::path::PathBuf;

use casper::rust::storage::rnode_key_value_store_manager::rnode_db_mapping;
use rspace_plus_plus::rspace::shared::{
    key_value_store_manager::KeyValueStoreManager,
    lmdb_dir_store_manager::{Db, LmdbDirStoreManager, LmdbEnvConfig, MB},
};

pub fn mk_test_rnode_store_manager(dir_path: PathBuf) -> impl KeyValueStoreManager {
    // Limit maximum environment (file) size for LMDB in tests
    let limit_size = 100 * MB;

    let db_mappings: Vec<(Db, LmdbEnvConfig)> = rnode_db_mapping(None)
        .into_iter()
        .map(|(db, mut conf)| {
            let new_conf = if conf.max_env_size > limit_size {
                conf.max_env_size = limit_size;
                conf
            } else {
                conf
            };

            (db, new_conf)
        })
        .collect();

    LmdbDirStoreManager::new(dir_path, db_mappings.into_iter().collect())
}
