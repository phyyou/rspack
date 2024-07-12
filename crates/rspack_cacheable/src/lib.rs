pub use rspack_macros::{cacheable, cacheable_dyn};
pub mod r#dyn;
pub mod with;

#[doc(hidden)]
pub mod __private {
  #[doc(hidden)]
  pub extern crate inventory;
  #[doc(hidden)]
  pub extern crate once_cell;
  #[doc(hidden)]
  pub extern crate rkyv;
}

pub trait Cacheable {
  fn serialize(&self) -> Vec<u8>;
  fn deserialize(bytes: &[u8]) -> Self
  where
    Self: Sized;
}

#[inline]
pub fn to_bytes<T: Cacheable>(data: &T) -> Vec<u8> {
  data.serialize()
}

#[inline]
pub fn from_bytes<T: Cacheable>(bytes: &[u8]) -> T {
  T::deserialize(bytes)
}
