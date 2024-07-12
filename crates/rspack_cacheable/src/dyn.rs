use crate::Cacheable;

// TODO try avoid clone
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct CacheableDynData(pub String, pub Vec<u8>);

impl Cacheable for CacheableDynData {
  #[inline]
  fn serialize(&self) -> Vec<u8> {
    rkyv::to_bytes::<_, 1024>(self)
      .expect("serialize #ident failed")
      .to_vec()
  }
  #[inline]
  fn deserialize(bytes: &[u8]) -> Self
  where
    Self: Sized,
  {
    rkyv::from_bytes::<Self>(bytes).expect("deserialize #ident failed")
  }
}

pub trait CacheableDyn {
  fn type_name(&self) -> String;
}
