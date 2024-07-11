use rkyv::{
  ser::{ScratchSpace, Serializer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};

use crate::Cacheable;

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
