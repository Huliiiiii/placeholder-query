use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DeriveInput, Fields, GenericParam, LitStr, meta::ParseNestedMeta};

use crate::row;

#[derive(Default)]
pub(crate) struct Args {
    name: Option<LitStr>,
    row: row::Args,
}

impl Args {
    pub(crate) fn parse(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("name") {
            if self.name.is_some() {
                return Err(meta.error("duplicate `name`"));
            }
            self.name = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("derive") {
            self.row.parse(meta)
        } else {
            Err(meta.error("expected `name` or `derive(...)`"))
        }
    }
}

pub(crate) fn expand(args: Args, input: DeriveInput) -> syn::Result<TokenStream> {
    let table_name = args.name.unwrap_or_else(|| {
        LitStr::new(
            &input
                .ident
                .to_string()
                .trim_start_matches("r#")
                .to_snake_case(),
            input.ident.span(),
        )
    });
    let table = expand_table(&input, &table_name)?;
    let record = row::expand(args.row, input)?;

    Ok(quote! { #record #table })
}

fn expand_table(input: &DeriveInput, table_name: &LitStr) -> syn::Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "table requires a struct with named fields",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "table requires named fields",
        ));
    };

    let crate_path = quote!(::placeholder_query_postgres);

    let ident = &input.ident;
    let generics = &input.generics;

    let (implementation, _, clause) = generics.split_for_impl();
    let arguments = generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote!(#ident)
            }
            GenericParam::Const(param) => {
                let ident = &param.ident;
                quote!(#ident)
            }
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote!(#lifetime)
            }
        })
        .collect::<Vec<_>>();

    let defaults = fields.named.iter().map(|field| {
        let field = field.ident.as_ref().expect("named field");
        let name = field.to_string().trim_start_matches("r#").to_owned();
        quote_spanned! {Span::mixed_site()=>
            #field: #crate_path::query::mode::Name::new(#name)
        }
    });

    Ok(quote_spanned! {Span::mixed_site()=>
        impl #implementation #ident<#(#arguments,)* #crate_path::query::mode::Value> #clause {
            pub fn table() -> #crate_path::query::table::TableSchema<
                #ident<#(#arguments,)* #crate_path::query::mode::Name>
            > {
                #crate_path::query::table::TableSchema::new(
                    #table_name,
                    #ident { #(#defaults,)* },
                )
            }
        }
    })
}
