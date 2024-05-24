mod option;
mod strategy;

use std::{
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use rkyv::Deserialize;

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

  pub fn add(&mut self, files: Vec<PathBuf>) {
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
          let data = rkyv::to_bytes::<_, 1024>(&s).expect("should to bytes success");
          self
            .storage
            .set(SCOPE, path_str.as_bytes().to_vec(), data.into_vec());
        }
      }
      // compiler time
      let data = rkyv::to_bytes::<_, 1024>(&Strategy::CompileTime(compiler_time))
        .expect("should to bytes success");
      self
        .storage
        .set(SCOPE, path_str.as_bytes().to_vec(), data.into_vec());
    }
  }

  pub fn remove(&mut self, files: Vec<PathBuf>) {
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
      let temp = rkyv::check_archived_root::<Strategy>(&value).unwrap();
      let strategy: Strategy = temp.deserialize(&mut rkyv::Infallible).unwrap();
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
