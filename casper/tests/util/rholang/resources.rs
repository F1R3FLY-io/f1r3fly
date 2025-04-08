// See casper/src/test/scala/coop/rchain/casper/util/rholang/Resources.scala

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::Builder;

use casper::rust::{
    genesis::genesis::Genesis, storage::rnode_key_value_store_manager::rnode_db_mapping,
    util::rholang::runtime_manager::RuntimeManager,
};
use models::rhoapi::Par;
use rholang::rust::interpreter::{
    rho_runtime::RhoHistoryRepository, test_utils::resources::mk_temp_dir,
};
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

pub async fn mk_runtime_manager(prefix: &str, mergeable_tag_name: Option<Par>) -> RuntimeManager {
    let dir_path = mk_temp_dir(prefix);
    let kvm = mk_test_rnode_store_manager(dir_path);

    mk_runtime_manager_at(kvm, mergeable_tag_name).await
}

pub async fn mk_runtime_manager_at(
    mut kvm: impl KeyValueStoreManager,
    mergeable_tag_name: Option<Par>,
) -> RuntimeManager {
    let mergeable_tag_name =
        mergeable_tag_name.unwrap_or(Genesis::non_negative_mergeable_tag_name());

    let r_store = kvm.r_space_stores().await.unwrap();
    let m_store = RuntimeManager::mergeable_store(&mut kvm).await.unwrap();
    RuntimeManager::create_with_store(r_store, m_store, mergeable_tag_name)
}

pub async fn mk_runtime_manager_with_history_at(
    mut kvm: impl KeyValueStoreManager,
) -> (RuntimeManager, RhoHistoryRepository) {
    let r_store = kvm.r_space_stores().await.unwrap();
    let m_store = RuntimeManager::mergeable_store(&mut kvm).await.unwrap();
    let (rt_manager, history_repo) = RuntimeManager::create_with_history(
        r_store,
        m_store,
        Genesis::non_negative_mergeable_tag_name(),
    );
    (rt_manager, history_repo)
}

/// Creates a managed temporary directory that will be automatically removed when the TempDir is dropped
pub fn with_temp_dir<F, R>(prefix: &str, f: F) -> R
where
    F: FnOnce(&Path) -> R,
{
    let temp_dir = Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("Failed to create temp dir");

    // Run the function with the temp_dir path
    let result = f(temp_dir.path());

    // TempDir will be dropped here and automatically cleaned up
    // unless we've decided to manually persist it
    result
}

/// Creates a temporary directory that will be persisted (not automatically cleaned up)
pub fn create_persisted_temp_dir(prefix: &str) -> PathBuf {
    let temp_dir = Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("Failed to create temp dir");

    // Convert to PathBuf which will persist even after TempDir is dropped
    let path = temp_dir.into_path();
    path
}

/// Copy a template storage directory to a new temporary directory that is persisted
/// If the source directory doesn't exist, it creates an empty directory instead
pub fn copy_storage(storage_template_path: PathBuf) -> PathBuf {
    // Create a persistent temporary directory instead of using with_temp_dir
    let temp_path_buf = create_persisted_temp_dir("casper-test-");
    copy_dir(storage_template_path, temp_path_buf.clone()).expect("Failed to copy directory");
    temp_path_buf
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> io::Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry_path.strip_prefix(src).unwrap());

        if entry_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}
