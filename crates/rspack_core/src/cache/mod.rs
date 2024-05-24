mod occasion;
mod snapshot;
mod storage;

use std::sync::Arc;

pub use self::snapshot::SnapshotOption;
use self::{
  snapshot::Snapshot,
  storage::{ArcStorage, FsStorage},
};

// TODO call write storage only build success
#[derive(Debug)]
pub struct Cache {
  snapshot: Snapshot,
}

// TODO conside multi compiler
impl Cache {
  pub fn new(snapshot_option: SnapshotOption) -> Self {
    let storage = Arc::new(FsStorage {});
    Self {
      snapshot: Snapshot::new(storage, snapshot_option),
    }
  }
}
