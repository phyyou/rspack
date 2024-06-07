mod occasion;
mod snapshot;
mod storage;

use std::{path::PathBuf, sync::Arc};

pub use self::snapshot::SnapshotOption;
use self::{
  snapshot::Snapshot,
  storage::{ArcStorage, FsStorage},
};

// TODO call write storage only build success
#[derive(Debug)]
pub struct Cache {
  pub storage: ArcStorage,
  pub snapshot: Snapshot,
}

// TODO conside multi compiler
impl Cache {
  pub fn new(snapshot_option: SnapshotOption) -> Self {
    let storage = Arc::new(FsStorage::new(PathBuf::from(
      "/Users/bytedance/project/rspack/node_modules/.cache/rspack/compiler-id-version",
    )));
    Self {
      snapshot: Snapshot::new(storage.clone(), snapshot_option),
      storage,
    }
  }

  pub fn idle(&self) {
    self.storage.idle();
  }
}
