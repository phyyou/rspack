use rkyv::{
  ser::{ScratchSpace, Serializer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};

use crate::{from_bytes, r#dyn::CacheableDynData, to_bytes, Cacheable};

pub struct AsCacheable;

pub struct AsCacheableResolver {
  inner: VecResolver,
  len: usize,
}

impl<T> ArchiveWith<T> for AsCacheable
where
  T: Cacheable,
{
  type Archived = ArchivedVec<u8>;
  type Resolver = AsCacheableResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedVec::resolve_from_len(resolver.len, pos, resolver.inner, out)
  }
}

impl<T, S> SerializeWith<T, S> for AsCacheable
where
  T: Cacheable,
  S: ?Sized + Serializer + ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let bytes = &field.serialize();
    Ok(AsCacheableResolver {
      inner: ArchivedVec::serialize_from_slice(bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl<T, D> DeserializeWith<ArchivedVec<u8>, T, D> for AsCacheable
where
  T: Cacheable,
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedVec<u8>, _: &mut D) -> Result<T, D::Error> {
    Ok(Cacheable::deserialize(field))
  }
}

// for rspack_source
use std::sync::Arc;

use rspack_sources::RawSource;
impl Cacheable for rspack_sources::BoxSource {
  fn serialize(&self) -> Vec<u8> {
    let inner = self.as_ref().as_any();
    let mut data: Option<CacheableDynData> = None;
    if let Some(raw_source) = inner.downcast_ref::<rspack_sources::RawSource>() {
      match raw_source {
        RawSource::Buffer(buf) => {
          // TODO try avoid clone
          data = Some(CacheableDynData(
            String::from("RawSource::Buffer"),
            buf.clone(),
          ));
        }
        RawSource::Source(source) => {
          data = Some(CacheableDynData(
            String::from("RawSource::Source"),
            source.as_bytes().to_vec(),
          ));
        }
      }
      //    } else if let Some() = inner.downcast_ref::<rspack_sources::RawSource>() {
    }

    if let Some(data) = data {
      to_bytes(&data)
    } else {
      panic!("unsupport box source")
    }
  }
  fn deserialize(bytes: &[u8]) -> Self
  where
    Self: Sized,
  {
    let CacheableDynData(type_name, data) = from_bytes(bytes);
    match type_name.as_str() {
      "RawSource::Buffer" => Arc::new(RawSource::Buffer(data)),
      "RawSource::Source" => Arc::new(RawSource::Source(
        String::from_utf8(data).expect("convert to string failed"),
      )),
      _ => {
        panic!("unsupport box source")
      }
    }
  }
}
