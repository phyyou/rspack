pub struct AsString;

pub trait AsStringConverter {
  fn as_str(&self) -> &str;
  fn from_str(s: &str) -> Self
  where
    Self: Sized;
}

impl<T, S> rkyv::with::SerializeWith<T, S> for AsString
where
  T: AsStringConverter,
  S: ?Sized + rkyv::ser::Serializer + rkyv::ser::ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    rkyv::string::ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<T> rkyv::with::ArchiveWith<T> for AsString
where
  T: AsStringConverter,
{
  type Archived = rkyv::string::ArchivedString;
  type Resolver = rkyv::string::StringResolver;

  #[inline]
  unsafe fn resolve_with(
    field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    rkyv::string::ArchivedString::resolve_from_str(field.as_str(), pos, resolver, out);
  }
}

// TODO change to FromStr
impl<T, D> rkyv::with::DeserializeWith<rkyv::string::ArchivedString, T, D> for AsString
where
  T: AsStringConverter,
  D: ?Sized + rkyv::Fallible,
{
  #[inline]
  fn deserialize_with(field: &rkyv::string::ArchivedString, _: &mut D) -> Result<T, D::Error> {
    Ok(AsStringConverter::from_str(field.as_str()))
  }
}
