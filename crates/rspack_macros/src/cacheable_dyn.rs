use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Ident, Item, ItemImpl, ItemTrait, Type};

pub fn impl_cacheable_dyn(_args: TokenStream, tokens: TokenStream) -> TokenStream {
  let input = parse_macro_input!(tokens as Item);

  match input {
    Item::Trait(input) => impl_trait(input),
    Item::Impl(input) => impl_impl(input),
    _ => panic!("expect Trait or Impl"),
  }
}

fn impl_trait(mut input: ItemTrait) -> TokenStream {
  let trait_ident = &input.ident;
  let data_ident = Ident::new(&format!("{trait_ident}Data"), trait_ident.span());
  let flag_ident = Ident::new(&format!("{trait_ident}Flag"), trait_ident.span());
  let flag_vis = &input.vis;

  input
    .supertraits
    .push(parse_quote!(rspack_cacheable::Cacheable));
  input
    .supertraits
    .push(parse_quote!(rspack_cacheable::CacheableDyn));

  quote! {
      #input

      #[allow(non_upper_case_globals)]
      const _: () = {
          use rspack_cacheable::__private::inventory;
          use rspack_cacheable::__private::once_cell;
          type DeserializeFn = fn(&[u8]) -> Box<dyn #trait_ident>;

          #flag_vis struct #flag_ident {
              name: &'static str,
              deserialize: DeserializeFn
          }
          inventory::collect!(#flag_ident);
          impl dyn #trait_ident {
              #[doc(hidden)]
              #flag_vis const fn cacheable_flag(name: &'static str, deserialize: DeserializeFn) -> #flag_ident {
                  #flag_ident { name, deserialize }
              }
          }

          use std::collections::BTreeMap;
          use std::collections::btree_map::Entry;
          static REGISTRY: once_cell::sync::Lazy<BTreeMap<&str, DeserializeFn>> = once_cell::sync::Lazy::new(|| {
              let mut map = BTreeMap::new();
              for flag in inventory::iter::<#flag_ident> {
                  let name = flag.name;
                  match map.entry(name) {
                      Entry::Vacant(val) => {
                          val.insert(flag.deserialize);
                      },
                      Entry::Occupied(_) => {
                          panic!("cacheable_dyn init global REGISTRY error, duplicate implementation of {name}");
                      }
                  }
              }
              map
          });

          #[rspack_cacheable::cacheable]
          struct #data_ident(String, Vec<u8>);
          impl rspack_cacheable::Cacheable for Box<dyn #trait_ident> {
              fn serialize(&self) -> Vec<u8> {
                  let inner = self.as_ref();
                  let data = #data_ident(inner.type_name(), inner.serialize());
                  rspack_cacheable::to_bytes(&data)
              }
              fn deserialize(bytes: &[u8]) -> Self where Self: Sized {
                  let #data_ident(name, data) = rspack_cacheable::from_bytes::<#data_ident>(bytes);
                  let deserialize_fn = REGISTRY.get(name.as_str()).expect("unsupport data type when deserialize");
                  deserialize_fn(&data)
              }
          }
      };
  }
  .into()
}

fn impl_impl(input: ItemImpl) -> TokenStream {
  let trait_ident = input
    .trait_
    .as_ref()
    .map(|inner| inner.1.get_ident())
    .expect("expect impl trait");
  let target_ident = &input.self_ty;
  let target_ident_string = match &*input.self_ty {
    Type::Path(inner) => {
      let name = &inner.path.segments.last().unwrap().ident.to_string();
      quote! {#name}
    }
    _ => {
      panic!("cacheable_dyn unsupport this target")
    }
  };

  quote! {
      #input

      #[allow(non_upper_case_globals)]
      const _: () = {
          use rspack_cacheable::__private::inventory;
          inventory::submit! {
              <dyn #trait_ident>::cacheable_flag(#target_ident_string, |bytes: &[u8]| {
                  Box::new(rspack_cacheable::from_bytes::<#target_ident>(bytes))
              })
          }

          impl rspack_cacheable::CacheableDyn for #target_ident {
              fn type_name(&self) -> String {
                  String::from(#target_ident_string)
              }
          }
      };
  }
  .into()
}
