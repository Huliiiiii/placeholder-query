use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::input::{Field, Input};

pub(super) fn expand(
    input: &Input<'_>,
    fields_ident: &Ident,
    params_ident: &Ident,
    columns_ident: &Ident,
) -> TokenStream {
    let Input {
        ident,
        visibility,
        fields,
    } = input;

    let pg = quote!(::placeholder_query_postgres::query);
    let mode = Ident::new("__InputMode", Span::mixed_site());
    let target = Ident::new("__TargetMode", Span::mixed_site());

    let declarations = fields.iter().map(|Field { ident, ty }| {
        quote! { pub #ident: <#mode as #pg::mode::Mode>::Field<#ty> }
    });

    let visitor = Ident::new("visitor", Span::mixed_site());
    let visits = fields.iter().map(|Field { ident, ty }| {
        quote! { #visitor.visit::<#ty>(&self.#ident); }
    });

    let transformer = Ident::new("transformer", Span::mixed_site());
    let transforms = fields.iter().map(|Field { ident, ty }| {
        quote! { #ident: #transformer.transform::<#ty>(&self.#ident) }
    });

    let cloned = fields
        .iter()
        .map(|Field { ident, .. }| quote! { #ident: self.#ident.clone() });

    let field_types = fields.iter().map(|Field { ty, .. }| {
        quote! { <#pg::Expr<#ty> as #pg::Projection>::Fields }
    });

    let decoded = Ident::new("fields", Span::mixed_site());
    let decoded_fields = fields
        .iter()
        .enumerate()
        .map(|(index, Field { ident, .. })| {
            let index = syn::Index::from(index);
            quote! { #ident: #pg::Projection::from_fields(&self.#ident, #decoded.#index) }
        });

    let accessors = fields.iter().map(|Field { ident, ty }| {
        quote! {
            pub fn #ident(&self) -> #pg::Expr<#ty> {
                self.#ident.clone()
            }
        }
    });

    quote! {
        #[doc(hidden)]
        #visibility struct #fields_ident<#mode: #pg::mode::Mode> {
            #(#declarations,)*
        }

        #visibility type #params_ident =
            #fields_ident<#pg::params::mode::Scalar>;

        #visibility type #columns_ident = #fields_ident<#pg::mode::Expr>;

        impl ::core::clone::Clone for #fields_ident<#pg::mode::Expr> {
            fn clone(&self) -> Self {
                Self { #(#cloned,)* }
            }
        }

        impl<#mode: #pg::mode::Mode> #pg::columns::Rebind for #fields_ident<#mode> {
            type Rebound<#target: #pg::mode::Mode> = #fields_ident<#target>;
        }

        impl<#mode: #pg::mode::Mode> #pg::columns::Columns<#mode> for #fields_ident<#mode> {
            fn visit_fields(&self, #visitor: &mut impl #pg::columns::FieldVisitor<#mode>) {
                #(#visits)*
            }

            fn map_fields<#target: #pg::mode::Mode>(
                &self,
                #transformer: &mut impl #pg::columns::FieldTransformer<#mode, #target>,
            ) -> Self::Rebound<#target> {
                #fields_ident { #(#transforms,)* }
            }
        }

        impl #pg::Projection for #fields_ident<#pg::mode::Expr> {
            type Fields = (#(#field_types,)*);
            type Output = #ident;

            fn from_fields(&self, #decoded: Self::Fields) -> Self::Output {
                #ident { #(#decoded_fields,)* }
            }
        }

        impl #fields_ident<#pg::mode::Expr> {
            #(#accessors)*
        }
    }
}
