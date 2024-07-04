mod option;
mod strategy;

use std::{
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use rustc_hash::FxHashSet as HashSet;

pub use self::option::{PathMatcher, SnapshotOption};
use self::strategy::{Strategy, StrategyHelper, ValidateResult};
use super::storage::ArcStorage;

const SCOPE: &'static str = "snapshot";

#[derive(Debug)]
pub struct Snapshot {
  storage: ArcStorage,
  option: SnapshotOption,
}

impl Snapshot {
  pub fn new(storage: ArcStorage, option: SnapshotOption) -> Self {
    Self { storage, option }
  }

  pub fn add(&self, files: &HashSet<PathBuf>) {
    let compiler_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let mut helper = StrategyHelper::default();
    for path in files {
      let path_str = path.to_str().expect("should can convert to string");
      if self.option.is_immutable_path(path_str) {
        continue;
      }
      if self.option.is_managed_path(path_str) {
        if let Some(s) = helper.lib_version(&path) {
          self.storage.set(
            SCOPE,
            path_str.as_bytes().to_vec(),
            rspack_cache::to_bytes(&Strategy::LibVersion(s)),
          );
        }
      }
      // compiler time
      self.storage.set(
        SCOPE,
        path_str.as_bytes().to_vec(),
        rspack_cache::to_bytes(&Strategy::CompileTime(compiler_time)),
      );
    }
  }

  pub fn remove(&self, files: &HashSet<PathBuf>) {
    for item in files {
      self
        .storage
        .remove(SCOPE, item.to_str().expect("should have str").as_bytes())
    }
  }

  pub fn calc_modified_files(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut helper = StrategyHelper::default();
    let mut modified_files = vec![];
    let mut deleted_files = vec![];

    for (key, value) in self.storage.get_all(SCOPE) {
      let path = PathBuf::from(String::from_utf8(key).unwrap());
      let strategy: Strategy = rspack_cache::from_bytes::<Strategy>(&value);
      match helper.validate(&path, &strategy) {
        ValidateResult::Modified => {
          modified_files.push(path);
        }
        ValidateResult::Deleted => {
          deleted_files.push(path);
        }
        _ => {}
      }
    }
    (modified_files, deleted_files)
  }
}
