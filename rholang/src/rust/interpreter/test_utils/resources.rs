// See rholang/src/test/scala/coop/rchain/rholang/Resources.scala

use std::path::PathBuf;
use tempfile::Builder;

pub fn mk_temp_dir(prefix: &str) -> PathBuf {
    let temp_dir = Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("Failed to create temp dir");
    temp_dir.into_path()
}
