use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item};

pub fn impl_cacheable(_args: TokenStream, tokens: TokenStream) -> TokenStream {
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

  let ident = match &input {
    Item::Enum(input) => &input.ident,
    Item::Struct(input) => &input.ident,
    _ => panic!("expect enum or struct"),
  };

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

fn add_attr_for_field(field: &mut syn::Field) {
  let mut is_box_dyn = false;
  if let syn::Type::Path(ty_path) = &field.ty {
    if let Some(seg) = &ty_path.path.segments.first() {
      // check Box<dyn xxx>
      if seg.ident == "Box" {
        if let syn::PathArguments::AngleBracketed(arg) = &seg.arguments {
          if let Some(syn::GenericArgument::Type(syn::Type::TraitObject(_))) = &arg.args.first() {
            is_box_dyn = true;
          }
        }
      }
    }
  }

  // for Box<dyn xxx>
  if is_box_dyn {
    field.attrs.push(parse_quote! {
        #[with(rspack_cacheable::with::AsCacheable)]
    });
  }
}
