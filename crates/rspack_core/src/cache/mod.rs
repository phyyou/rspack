mod occasion;
mod snapshot;
mod storage;

use self::storage::ArcStorage;

// call write storage only build success
pub struct Cache {
  storage: ArcStorage,
}

impl Cache {}
