use super::{from_bytes, to_bytes, Cacheable};

pub struct AsCacheable;

pub struct AsCacheableResolver {
  inner: rkyv::vec::VecResolver,
  len: usize,
}

impl<T, S> rkyv::with::SerializeWith<T, S> for AsCacheable
where
  T: Cacheable,
  S: ?Sized + rkyv::ser::Serializer + rkyv::ser::ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let bytes = &to_bytes(field);
    Ok(AsCacheableResolver {
      inner: rkyv::vec::ArchivedVec::serialize_from_slice(bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl<T> rkyv::with::ArchiveWith<T> for AsCacheable
where
  T: Cacheable,
{
  type Archived = rkyv::Archived<Vec<u8>>;
  type Resolver = AsCacheableResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    rkyv::vec::ArchivedVec::resolve_from_len(resolver.len, pos, resolver.inner, out)
  }
}

impl<T, D> rkyv::with::DeserializeWith<rkyv::vec::ArchivedVec<u8>, T, D> for AsCacheable
where
  T: Sized + Cacheable,
  D: ?Sized + rkyv::Fallible,
{
  #[inline]
  fn deserialize_with(field: &rkyv::vec::ArchivedVec<u8>, _: &mut D) -> Result<T, D::Error> {
    Ok(from_bytes::<T>(field))
  }
}
