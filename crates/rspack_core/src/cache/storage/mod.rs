mod fs;

use std::sync::Arc;

// pub use fs::FsStorage;

pub trait Storage: std::fmt::Debug {
  fn get_all(&self, scope: &str) -> Vec<(Vec<u8>, Vec<u8>)>;
  fn get(&self, scope: &str, key: &[u8]) -> Vec<u8>;
  fn set(&self, scope: &str, key: Vec<u8>, value: Vec<u8>);
  fn remove(&self, scope: &str, key: &[u8]);
  fn idle(&self);
}

pub type ArcStorage = Arc<dyn Storage>;
