use super::super::ArcStorage;

#[derive(Debug)]
pub struct MakeOccasion {
  storage: ArcStorage,
}

impl MakeOccasion {
  pub fn new(storage: ArcStorage) -> Self {
    Self { storage }
  }
}
