use std::{
    cell::OnceCell,
    collections::HashSet,
    io::{Read, Write},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseBuffer, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprLit, Fields, ItemStruct, Lit, LitStr, Meta, Token,
};

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

#[proc_macro]
pub fn env_var(tokens: TokenStream) -> TokenStream {
    struct FuckInner {
        fuck: Punctuated<Expr, Token![,]>,
    }

    impl Parse for FuckInner {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(FuckInner {
                fuck: Punctuated::parse_separated_nonempty(input)?,
            })
        }
    }

    struct FuckOuter {
        fuck: Punctuated<FuckInner, Token![;]>,
    }

    impl Parse for FuckOuter {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(FuckOuter {
                fuck: Punctuated::parse_separated_nonempty(input)?,
            })
        }
    }

    let mut var_exprs = Vec::new();
    let mut var_names = Vec::new();
    let outer = parse_macro_input!(tokens as FuckOuter);
    let num = outer.fuck.len();
    for (i, inner) in outer.fuck.into_iter().enumerate() {
        if i + 1 == num {
            let ts = inner.fuck;
            var_exprs.push(quote::quote!(#ts));
            break;
        }

        let mut var_expr = quote::quote!("CR");
        let mut var_name = String::from("CR");
        for expr in inner.fuck {
            var_expr = quote::quote!(#var_expr , "_");
            var_name.push('_');
            match expr {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Str(litstr) => {
                        let name = litstr.value().to_uppercase();
                        var_expr = quote::quote!(#var_expr , #name);
                        var_name.push_str(&name);
                    }
                    lit => {
                        var_expr = quote::quote!(#var_expr , stringify!(#lit));
                        var_name.push('[');
                        var_name.push_str(&lit.to_token_stream().to_string());
                        var_name.push(']');
                    }
                },
                expr => {
                    var_expr = quote::quote!(#var_expr , &(#expr).to_uppercase());
                    // SAFETY: we convert the string to ascii
                    let as_ascii = unsafe {
                        let mut s = expr.to_token_stream().to_string();
                        s.as_bytes_mut().iter_mut().for_each(|byte| {
                            if !byte.is_ascii_alphabetic() {
                                *byte = b'_';
                            }
                        });
                        s
                    };
                    var_name.push('[');
                    var_name.push_str(&as_ascii);
                    var_name.push(']');
                }
            }
        }
        var_names.push(var_name);
        var_exprs.push(var_expr);
    }

    let env_var_dir = PathBuf::from("dist").join("env_vars");
    for var in var_names.iter() {
        std::fs::write(env_var_dir.join(var), "").expect("create file");
    }

    let Some(or) = var_exprs.pop() else {
        return quote::quote!(compile_error!("need at least one item"))
            .into_token_stream()
            .into();
    };

    quote::quote!(
        crate::get_env_or(&[#(&[#var_exprs]),*], #or)
    )
    .into_token_stream()
    .into()
}
