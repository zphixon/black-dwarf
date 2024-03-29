use proc_macro::TokenStream;
use quote::ToTokens;
use std::path::PathBuf;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, Expr, ExprLit, Fields, ItemStruct,
    Lit, LitStr, Meta, Token,
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

#[derive(Clone)]
struct EnvVar {
    parts: Punctuated<Expr, Token![,]>,
}

impl Parse for EnvVar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(EnvVar {
            parts: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

struct EnvVarCall {
    doc: Option<String>,
    vars: Vec<EnvVar>,
    default: Expr,
}

mod kw {
    syn::custom_keyword!(doc);
}

impl Parse for EnvVarCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let doc;
        if lookahead.peek(kw::doc) {
            let _become: kw::doc = input.parse()?;
            let lit: LitStr = input.parse()?;
            doc = Some(lit.value());
        } else {
            doc = None;
        }

        let mut vars = Punctuated::<EnvVar, Token![;]>::parse_separated_nonempty(input)?
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let Some(EnvVar { parts }) = vars.pop() else {
            return Err(syn::Error::new(
                input.span(),
                "need at least a default value",
            ));
        };

        let [default] = &parts.into_iter().collect::<Vec<_>>()[..] else {
            return Err(syn::Error::new(
                input.span(),
                "expected a single expression for default value, got multiple",
            ));
        };
        let default = default.clone();

        Ok(EnvVarCall { doc, vars, default })
    }
}

#[proc_macro]
pub fn env_var(tokens: TokenStream) -> TokenStream {
    let mut var_exprs = Vec::new();
    let mut var_names = Vec::new();
    let outer = parse_macro_input!(tokens as EnvVarCall);
    for inner in outer.vars.into_iter() {
        let mut var_expr = quote::quote!("CR");
        let mut var_name = String::from("CR");
        for expr in inner.parts {
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
        std::fs::write(
            env_var_dir.join(var),
            outer
                .doc
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Undocumented"),
        )
        .expect("create file");
    }

    let or = outer.default;

    quote::quote!(
        crate::get_env_or(&[#(&[#var_exprs]),*], #or)
    )
    .into_token_stream()
    .into()
}
