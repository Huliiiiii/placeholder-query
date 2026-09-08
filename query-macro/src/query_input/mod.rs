mod view;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::input::{self, Field};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let input = input::parse(&input)?;
    let ident = input.ident;

    let params_path = quote!(::placeholder_query_postgres::query::params);
    let query_path = quote!(::placeholder_query_postgres::query);
    let record_path = quote!(#params_path::record);

    if input.fields.is_empty() {
        return Ok(quote! {
            impl #record_path::InputRecord for #ident {
                type Fields = ();
                type Names = ();

                const NAMES: &'static [&'static str] = &[];

                fn names() {}
                fn fields(&self) {}
            }
        });
    }

    let fields_ident = format_ident!("{ident}Fields", span = Span::mixed_site());
    let params_ident = format_ident!("{ident}Params", span = Span::mixed_site());
    let columns_ident = format_ident!("{ident}Columns", span = Span::mixed_site());

    let view = view::expand(&input, &fields_ident, &params_ident, &columns_ident);

    let types = input.fields.iter().map(|field| field.ty);
    let names = input.fields.iter().map(|field| {
        use syn::ext::IdentExt;
        field.ident.unraw().to_string()
    });

    let named_fields = input
        .fields
        .iter()
        .enumerate()
        .map(|(index, Field { ident, .. })| {
            let index = syn::LitInt::new(&index.to_string(), ident.span());
            quote! {
                #ident: #query_path::mode::Name::new(Self::NAMES[#index])
            }
        })
        .collect::<Vec<_>>();

    let values = input
        .fields
        .iter()
        .map(|Field { ident, .. }| quote! { &self.#ident });

    Ok(quote! {
        #view

        impl #record_path::InputRecord for #ident {
            type Fields = (#(#types,)*);
            type Names = #fields_ident<#query_path::mode::Name>;

            const NAMES: &'static [&'static str] = &[#(#names,)*];

            fn names() -> Self::Names {
                #fields_ident { #(#named_fields,)* }
            }

            fn fields(&self) -> <Self::Fields as #record_path::RecordFields>::Values<'_> {
                (#(#values,)*)
            }
        }
    })
}
