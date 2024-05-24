mod option;
mod strategy;

use std::{
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use json::parse;
use rkyv::Deserialize;

pub use self::option::{PathMatcher, SnapshotOption};
use self::strategy::Strategy;
use super::storage::ArcStorage;

const SCOPE: &'static str = "snapshot";

pub struct Snapshot {
  storage: ArcStorage,
  option: SnapshotOption,
}

impl Snapshot {
  pub fn new(storage: ArcStorage, option: SnapshotOption) -> Self {
    Self { storage, option }
  }

  pub fn add(&mut self, files: Vec<PathBuf>) {
    for item in files {
      let path = item.to_str().expect("should can convert to string");
      if self.option.is_immutable_path(path) {
        continue;
      }
      if self.option.is_managed_path(path) {
        if let Some(s) = Strategy::lib_version(item.clone()) {
          let data = rkyv::to_bytes::<_, 1024>(&s).expect("should to bytes success");
          self
            .storage
            .set(SCOPE, path.as_bytes().to_vec(), data.into_vec());
        }
      }
      // update time
      let data =
        rkyv::to_bytes::<_, 1024>(&Strategy::UpdateTime()).expect("should to bytes success");
      self
        .storage
        .set(SCOPE, path.as_bytes().to_vec(), data.into_vec());
    }

    self.storage.set(
      "meta",
      "update_time".as_bytes().to_vec(),
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
        .into_bytes(),
    )
  }

  pub fn remove(&mut self, files: Vec<PathBuf>) {
    for item in files {
      self
        .storage
        .remove(SCOPE, item.to_str().expect("should have str").as_bytes())
    }
  }

  pub fn calc_modified_files(&self) -> Vec<PathBuf> {
    let Ok(str) = String::from_utf8(self.storage.get("meta", "update_time".as_bytes())) else {
      return vec![];
    };
    let Ok(update_time) = str.parse::<u64>() else {
      return vec![];
    };

    let mut modified_files = vec![];

    for (key, value) in self.storage.get_all(SCOPE) {
      let path = PathBuf::from(String::from_utf8(key).unwrap());
      let temp = rkyv::check_archived_root::<Strategy>(&value).unwrap();
      let res: Strategy = temp.deserialize(&mut rkyv::Infallible).unwrap();
      if let Some(true) = res.check(&path, update_time) {
        modified_files.push(path);
      }
    }
    modified_files
  }
}
