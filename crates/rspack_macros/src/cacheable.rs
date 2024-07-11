use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, Item, Result,
};

mod kw {
  syn::custom_keyword!(with);
}
pub struct CacheableArgs {
  pub with: syn::Path,
}
impl Parse for CacheableArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<kw::with>()?;
    input.parse::<syn::Token![=]>()?;
    let with = input.parse::<syn::Path>()?;
    Ok(Self { with })
  }
}

pub fn impl_cacheable(tokens: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  // add attr for some field
  match &mut input {
    Item::Enum(input) => {
      for v in input.variants.iter_mut() {
        for f in v.fields.iter_mut() {
          add_attr_for_field(f);
        }
      }
    }
    Item::Struct(input) => {
      for f in input.fields.iter_mut() {
        add_attr_for_field(f);
      }
    }
    _ => panic!("expect enum or struct"),
  }

  let ident = get_ident(&input);

  quote! {
      #[derive(
          rspack_cacheable::__private::rkyv::Archive,
          rspack_cacheable::__private::rkyv::Deserialize,
          rspack_cacheable::__private::rkyv::Serialize
      )]
      #[archive(check_bytes, crate="rspack_cacheable::__private::rkyv")]
      #input
      impl rspack_cacheable::Cacheable for #ident {
          #[inline]
          fn serialize(&self) -> Vec<u8> {
              rspack_cacheable::__private::rkyv::to_bytes::<_, 1024>(self).expect("serialize #ident failed").to_vec()
          }
          #[inline]
          fn deserialize(bytes: &[u8]) -> Self where Self: Sized {
              rspack_cacheable::__private::rkyv::from_bytes::<Self>(bytes).expect("deserialize #ident failed")
          }
      }
  }
  .into()
}

pub fn impl_cacheable_with(tokens: TokenStream, with: syn::Path) -> TokenStream {
  let input = parse_macro_input!(tokens as Item);
  let ident = get_ident(&input);
  let archived = quote! {<#with as rkyv::with::ArchiveWith<#ident>>::Archived};
  let resolver = quote! {<#with as rkyv::with::ArchiveWith<#ident>>::Resolver};
  let rkyv_with = quote! {rkyv::with::With<#ident, #with>};
  quote! {
      #input
      #[allow(non_upper_case_globals)]
      const _: () = {
          use rspack_cacheable::__private::rkyv;
          impl rkyv::Archive for #ident {
              type Archived = #archived;
              type Resolver = #resolver;
              #[inline]
              unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
                  <#rkyv_with>::cast(self).resolve(pos, resolver, out)
              }
          }
          impl<S> rkyv::Serialize<S> for #ident
          where
              #rkyv_with: rkyv::Serialize<S>,
              S: rkyv::Fallible + ?Sized,
          {
              #[inline]
              fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                  <#rkyv_with>::cast(self).serialize(serializer)
              }
          }
          impl<D: rkyv::Fallible + ?Sized> rkyv::Deserialize<#ident, D> for #archived
          where
              #rkyv_with: rkyv::Archive,
              rkyv::Archived<#rkyv_with>: rkyv::Deserialize<#rkyv_with, D>,
          {
              #[inline]
              fn deserialize(&self, _deserializer: &mut D) -> Result<#ident, D::Error> {
                  Ok(
                      rkyv::Deserialize::<#rkyv_with, D>::deserialize(
                          self,
                          _deserializer,
                      )?.into_inner()
                  )
              }
          }
          impl rspack_cacheable::Cacheable for #ident {
              #[inline]
              fn serialize(&self) -> Vec<u8> {
                  rkyv::to_bytes::<_, 1024>(self).expect("serialize #ident failed").to_vec()
              }
              #[inline]
              fn deserialize(bytes: &[u8]) -> Self where Self: Sized {
                  rkyv::from_bytes::<Self>(bytes).expect("deserialize #ident failed")
              }
          }
      };
  }
  .into()
}

fn get_ident(input: &Item) -> &syn::Ident {
  match &input {
    Item::Enum(input) => &input.ident,
    Item::Struct(input) => &input.ident,
    _ => panic!("expect enum or struct"),
  }
}

fn add_attr_for_field(field: &mut syn::Field) {
  if let syn::Type::Path(ty_path) = &field.ty {
    if let Some(seg) = &ty_path.path.segments.first() {
      // check Box<dyn xxx>
      if seg.ident == "Box" {
        if let syn::PathArguments::AngleBracketed(arg) = &seg.arguments {
          if let Some(syn::GenericArgument::Type(syn::Type::TraitObject(_))) = &arg.args.first() {
            field.attrs.push(syn::parse_quote! {
                #[with(rspack_cacheable::with::AsCacheable)]
            });
            return;
          }
        }
      }

      // check Option<JsonValue>
      if seg.ident == "Option" {
        if let syn::PathArguments::AngleBracketed(arg) = &seg.arguments {
          if let Some(syn::GenericArgument::Type(syn::Type::Path(sub_path))) = &arg.args.first() {
            if sub_path.path.is_ident("JsonValue") {
              field.attrs.push(syn::parse_quote! {
              #[with(rspack_cacheable::with::AsOption<rspack_cacheable::with::AsString>)]
                            });
              return;
            }
          }
        }
      }

      if seg.ident == "HashSet" {
        if let syn::PathArguments::AngleBracketed(arg) = &seg.arguments {
          if let Some(syn::GenericArgument::Type(syn::Type::Path(sub_path))) = &arg.args.first() {
            if sub_path.path.is_ident("PathBuf") {
              field.attrs.push(syn::parse_quote! {
                  #[with(rspack_cacheable::with::AsVec<rspack_cacheable::with::AsString>)]
              });
              return;
            }
          }
        }
      }
    }
  }
}
