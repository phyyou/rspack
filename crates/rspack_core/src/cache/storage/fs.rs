use super::Storage;

//use rustc_hash::FxHashMap as HashMap;

#[derive(Debug)]
pub struct FsStorage {
  //    inner: HashMap<Vec<u8>, Vec<u8>>
}

impl Storage for FsStorage {
  fn get_all(&self, _scope: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    vec![]
  }
  //  fn get(&self, _scope: &str, _key: &[u8]) -> Option<Vec<u8>> {
  //    None
  //  }
  fn set(&self, _scope: &str, _key: Vec<u8>, _value: Vec<u8>) {}
  fn remove(&self, _scope: &str, _key: &[u8]) {}
  fn idle(&self) {}
}
