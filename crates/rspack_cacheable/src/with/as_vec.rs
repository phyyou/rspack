use std::marker::PhantomData;

use rkyv::{
  ser::{ScratchSpace, Serializer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Fallible, Serialize,
};

pub struct AsVec<T> {
  _inner: T,
}

pub trait AsVecConverter {
  type Item;
  fn len(&self) -> usize;
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item>;
  fn from(data: Vec<Self::Item>) -> Self;
}

impl<T, O, A> ArchiveWith<T> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  A: ArchiveWith<O>,
{
  type Archived = ArchivedVec<<A as ArchiveWith<O>>::Archived>;
  type Resolver = VecResolver;

  unsafe fn resolve_with(
    field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedVec::resolve_from_len(field.len(), pos, resolver, out)
  }
}

impl<T, A, O, S> SerializeWith<T, S> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  S: Fallible + ScratchSpace + Serializer + ?Sized,
  A: ArchiveWith<O> + SerializeWith<O, S>,
{
  fn serialize_with(field: &T, s: &mut S) -> Result<Self::Resolver, S::Error> {
    // Wrapper for O so that we have an Archive and Serialize implementation
    // and ArchivedVec::serialize_from_* is happy about the bound constraints
    struct RefWrapper<'o, A, O>(&'o O, PhantomData<A>);

    impl<A: ArchiveWith<O>, O> Archive for RefWrapper<'_, A, O> {
      type Archived = <A as ArchiveWith<O>>::Archived;
      type Resolver = <A as ArchiveWith<O>>::Resolver;

      unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        A::resolve_with(self.0, pos, resolver, out)
      }
    }

    impl<A, O, S> Serialize<S> for RefWrapper<'_, A, O>
    where
      A: ArchiveWith<O> + SerializeWith<O, S>,
      S: Fallible + Serializer + ?Sized,
    {
      fn serialize(&self, s: &mut S) -> Result<Self::Resolver, S::Error> {
        A::serialize_with(self.0, s)
      }
    }

    let iter = field
      .iter()
      .map(|value| RefWrapper::<'_, A, O>(value, PhantomData));

    ArchivedVec::serialize_from_iter(iter, s)
  }
}

impl<T, A, O, D> DeserializeWith<ArchivedVec<<A as ArchiveWith<O>>::Archived>, T, D> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  A: ArchiveWith<O> + DeserializeWith<<A as ArchiveWith<O>>::Archived, O, D>,
  D: Fallible + ?Sized,
{
  fn deserialize_with(
    field: &ArchivedVec<<A as ArchiveWith<O>>::Archived>,
    d: &mut D,
  ) -> Result<T, D::Error> {
    let mut res = Vec::with_capacity(field.len());
    for item in field.iter() {
      res.push(<A as DeserializeWith<_, _, D>>::deserialize_with(item, d)?);
    }
    Ok(<T as AsVecConverter>::from(res))
  }
}

// for rustc_hash::FxHashSet
impl<T> AsVecConverter for rustc_hash::FxHashSet<T>
where
  T: std::cmp::Eq + std::hash::Hash,
{
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(data: Vec<Self::Item>) -> Self {
    rustc_hash::FxHashSet::from_iter(data.into_iter())
  }
}
