use rkyv::{
  ser::{ScratchSpace, Serializer},
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};

pub struct AsString;

pub trait AsStringConverter {
  fn to_string(&self) -> String;
  fn from_str(s: &str) -> Self
  where
    Self: Sized;
}

pub struct AsStringResolver {
  inner: StringResolver,
  value: String,
}

impl<T> ArchiveWith<T> for AsString
where
  T: AsStringConverter,
{
  type Archived = ArchivedString;
  type Resolver = AsStringResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let AsStringResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, pos, inner, out);
  }
}

impl<T, S> SerializeWith<T, S> for AsString
where
  T: AsStringConverter,
  S: ?Sized + Serializer + ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let value = field.to_string();
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(AsStringResolver { value, inner })
  }
}

impl<T, D> DeserializeWith<ArchivedString, T, D> for AsString
where
  T: AsStringConverter,
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<T, D::Error> {
    Ok(AsStringConverter::from_str(field.as_str()))
  }
}

// for pathbuf
use std::path::PathBuf;
impl AsStringConverter for PathBuf {
  fn to_string(&self) -> String {
    self.to_string_lossy().to_string()
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    PathBuf::from(s)
  }
}

// for json value
impl AsStringConverter for json::JsonValue {
  fn to_string(&self) -> String {
    json::stringify(self.clone())
  }
  fn from_str(s: &str) -> Self
  where
    Self: Sized,
  {
    json::parse(s).expect("parse json failed")
  }
}
