pub use rspack_macros::cacheable;

#[doc(hidden)]
pub mod __private {
  #[doc(hidden)]
  pub extern crate rkyv;
}

pub trait Cacheable {
  fn serialize(&self) -> Vec<u8>;
  fn deserialize(bytes: &[u8]) -> Self;
}

pub fn to_bytes<T: Cacheable>(data: &T) -> Vec<u8> {
  data.serialize()
}

pub fn from_bytes<T: Cacheable>(bytes: &[u8]) -> T {
  T::deserialize(bytes)
}
