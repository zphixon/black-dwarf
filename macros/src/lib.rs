use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Fields, ItemStruct, Meta};

#[proc_macro_derive(UnusedKeys, attributes(unused))]
pub fn derive_unused_keys(item: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields: Fields::Named(fields),
        ..
    } = syn::parse_macro_input!(item)
    else {
        return quote::quote!(compile_error!("not implemented for tuple structs"))
            .into_token_stream()
            .into();
    };

    let [rest] = &fields
        .named
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| match &attr.meta {
                Meta::Path(path) => {
                    path.segments
                        .iter()
                        .map(|seg| format!("{}", seg.ident))
                        .collect::<Vec<_>>()
                        == vec![String::from("unused")]
                }
                _ => false,
            })
        })
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>()[..]
    else {
        return quote::quote!(compile_error!("one field must have #[unused] attribute"))
            .into_token_stream()
            .into();
    };

    let other_fields = fields
        .named
        .iter()
        .filter(|field| field.ident.as_ref() != Some(rest))
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    quote::quote!(
        impl UnusedKeys for #ident {
            fn unused_keys(&self) -> Vec<String> {
                let map: &::std::collections::HashMap<String, _> = &self.#rest;
                map.keys().map(|key| key.clone())
                #(.chain(
                    self
                        .#other_fields
                        .unused_keys()
                        .into_iter()
                        .map(|key| format!("{}.{}", stringify!(#other_fields), key))
                ))*
                .collect()
            }
        }
    )
    .into_token_stream()
    .into()
}
