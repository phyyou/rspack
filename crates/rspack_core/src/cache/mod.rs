mod occasion;
mod snapshot;
mod storage;

use std::sync::Arc;

use self::{
  snapshot::{Snapshot, SnapshotOption},
  storage::{ArcStorage, FsStorage},
};

// call write storage only build success
pub struct Cache {
  storage: ArcStorage,
  snapshot: Snapshot,
}

// TODO conside multi compiler
impl Cache {
  pub fn new(snapshot_option: SnapshotOption) -> Self {
    let storage = Arc::new(FsStorage {});
    Self {
      snapshot: Snapshot::new(storage.clone(), snapshot_option),
      storage,
    }
  }
}
