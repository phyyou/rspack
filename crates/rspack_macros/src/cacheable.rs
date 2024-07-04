use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

pub fn impl_cacheable(_args: TokenStream, tokens: TokenStream) -> TokenStream {
  let input = parse_macro_input!(tokens as Item);

  let ident = match &input {
    Item::Enum(input) => &input.ident,
    Item::Struct(input) => &input.ident,
    _ => panic!("expect enum or struct"),
  };

  quote! {
      #[derive(rspack_cache::__private::rkyv::Archive, rspack_cache::__private::rkyv::Deserialize, rspack_cache::__private::rkyv::Serialize)]
      #[archive(check_bytes, crate="rspack_cache::__private::rkyv")]
      #input

      impl rspack_cache::Cacheable for #ident {
          fn serialize(&self) -> Vec<u8> {
              rspack_cache::__private::rkyv::to_bytes::<_, 1024>(self).expect("serialize #ident failed").to_vec()
          }
          fn deserialize(bytes: &[u8]) -> Self {
              rspack_cache::__private::rkyv::from_bytes::<Self>(bytes).expect("deserialize #ident failed")
          }
      }
  }
  .into()
}
