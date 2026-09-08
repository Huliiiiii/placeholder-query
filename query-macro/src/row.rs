use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DeriveInput, Fields, GenericParam, Generics, Ident, Path, Token, Type, Visibility,
    meta::ParseNestedMeta, parenthesized, parse_quote,
};

#[derive(Default)]
pub(crate) struct Args {
    derives: Vec<Path>,
}

impl Args {
    pub(crate) fn parse(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if !meta.path.is_ident("derive") {
            return Err(meta.error("expected `derive(...)`"));
        }

        let content;
        parenthesized!(content in meta.input);
        self.derives
            .extend(content.parse_terminated(Path::parse_mod_style, Token![,])?);
        Ok(())
    }
}

pub(crate) fn expand(args: Args, mut input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(data) = &mut input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "row requires a struct with named fields",
        ));
    };

    let Fields::Named(fields) = &mut data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "row requires named fields",
        ));
    };

    let mode = (0..)
        .map(|index| {
            if index == 0 {
                format_ident!("M", span = Span::mixed_site())
            } else {
                format_ident!("M{index}", span = Span::mixed_site())
            }
        })
        .find(|candidate| {
            !input.generics.params.iter().any(|param| match param {
                GenericParam::Type(param) => param.ident == *candidate,
                GenericParam::Const(param) => param.ident == *candidate,
                GenericParam::Lifetime(_) => false,
            })
        })
        .unwrap();

    input.generics.params.push(parse_quote!(
        #mode: ::placeholder_query_postgres::query::mode::Mode = ::placeholder_query_postgres::query::mode::Value
    ));

    let mut row_fields = Vec::new();
    for field in &mut fields.named {
        let value = field.ty.clone();
        field.ty = parse_quote!(
            <#mode as ::placeholder_query_postgres::query::mode::Mode>::Field<#value>
        );
        row_fields.push(Field {
            ident: field.ident.clone().expect("named field"),
            visibility: field.vis.clone(),
            value,
        });
    }

    let derives = if args.derives.is_empty() {
        quote! {}
    } else {
        let traits = &args.derives;
        let bounds = if fields.named.is_empty() {
            quote! {}
        } else {
            let types = fields.named.iter().map(|field| &field.ty);
            quote! { ; #(#types),* }
        };
        quote! {
            #[::placeholder_query_postgres::__derive_where::derive_where(
                crate = ::placeholder_query_postgres::__derive_where
            )]
            #[derive_where(#(#traits),* #bounds)]
        }
    };

    let row = Input {
        ident: input.ident.clone(),
        generics: input.generics.clone(),
        fields: row_fields,
    };

    let columns = row.expand_columns();
    let projection = row.expand_projection();
    let accessors = row.expand_accessors();

    Ok(quote! { #derives #input #columns #projection #accessors })
}

struct Field {
    ident: Ident,
    visibility: Visibility,
    value: Type,
}

struct Input {
    ident: Ident,
    generics: Generics,
    fields: Vec<Field>,
}

fn value_arguments(generics: &Generics) -> Vec<TokenStream> {
    generics
        .params
        .iter()
        .take(generics.params.len().saturating_sub(1))
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
        .collect()
}

impl Input {
    fn expand_columns(&self) -> TokenStream {
        let pg = quote!(::placeholder_query_postgres::query);
        let ident = &self.ident;
        let mode = &self.generics.type_params().last().unwrap().ident;
        let generics = self.generics.clone();
        let target_mode = (0..)
            .map(|index| {
                if index == 0 {
                    format_ident!("N", span = Span::mixed_site())
                } else {
                    format_ident!("N{index}", span = Span::mixed_site())
                }
            })
            .find(|candidate| {
                !self.generics.params.iter().any(|param| match param {
                    GenericParam::Type(param) => param.ident == *candidate,
                    GenericParam::Const(param) => param.ident == *candidate,
                    GenericParam::Lifetime(_) => false,
                })
            })
            .unwrap();
        let (implementation, arguments, clause) = generics.split_for_impl();
        let value_arguments = value_arguments(&self.generics);
        let fields = self.fields.iter().map(|Field { ident, value, .. }| {
            quote_spanned! {Span::mixed_site()=>
                #ident: transformer.transform::<#value>(&self.#ident)
            }
        });
        let visits = self.fields.iter().map(|Field { ident, value, .. }| {
            quote_spanned! {Span::mixed_site()=>
                visitor.visit::<#value>(&self.#ident);
            }
        });
        quote_spanned! {Span::mixed_site()=>
            impl #implementation #pg::columns::Rebind for #ident #arguments #clause {
                type Rebound<#target_mode: #pg::mode::Mode> = #ident<#(#value_arguments,)* #target_mode>;
            }

            impl #implementation #pg::columns::Columns<#mode> for #ident #arguments #clause {
                fn visit_fields(&self, visitor: &mut impl #pg::columns::FieldVisitor<#mode>) {
                    #(#visits)*
                }

                fn map_fields<#target_mode: #pg::mode::Mode>(
                    &self,
                    transformer: &mut impl #pg::columns::FieldTransformer<#mode, #target_mode>,
                ) -> Self::Rebound<#target_mode> {
                    #ident { #(#fields,)* }
                }
            }
        }
    }

    fn expand_projection(&self) -> TokenStream {
        let pg = quote!(::placeholder_query_postgres::query);
        let ident = &self.ident;
        let expr = quote!(#pg::mode::Expr);
        let mut generics = self.generics.clone();
        generics.params.pop();
        let (implementation, _, clause) = generics.split_for_impl();
        let value_arguments = value_arguments(&self.generics);
        let value_type = quote!(#ident<#(#value_arguments,)* #pg::mode::Value>);
        let value_constructor = quote!(#ident::<#(#value_arguments,)* #pg::mode::Value>);
        let fields_type = self
            .fields
            .iter()
            .map(|Field { value, .. }| {
                quote!(<#pg::Expr<#value> as #pg::projection::Projection>::Fields)
            });
        let field_idents = self
            .fields
            .iter()
            .map(|Field { ident, .. }| ident.clone())
            .collect::<Vec<_>>();
        let fields = self.fields.iter().map(|ident| {
            let field = &ident.ident;
            quote_spanned! {Span::mixed_site()=>
                #field: #pg::projection::Projection::from_fields(&self.#field, #field)
            }
        });
        quote_spanned! {Span::mixed_site()=>
            impl #implementation #pg::projection::Projection for #ident<#(#value_arguments,)* #expr> #clause {
                type Fields = (#(#fields_type,)*);
                type Output = #value_type;
                fn from_fields(&self, fields: Self::Fields) -> Self::Output {
                    let (#(#field_idents,)*) = fields;
                    #value_constructor { #(#fields,)* }
                }
            }
        }
    }

    fn expand_accessors(&self) -> TokenStream {
        let pg = quote!(::placeholder_query_postgres::query);
        let ident = &self.ident;
        let mut generics = self.generics.clone();
        generics.params.pop();
        let (implementation, _, clause) = generics.split_for_impl();
        let arguments = generics.params.iter().map(|param| match param {
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
        });
        let accessors = self.fields.iter().map(
            |Field {
                 ident,
                 visibility,
                 value,
                 ..
             }| {
                quote_spanned! {Span::mixed_site()=>
                    #visibility fn #ident(&self) -> #pg::Expr<#value> {
                        self.#ident.clone()
                    }
                }
            },
        );
        quote_spanned! {Span::mixed_site()=>
            impl #implementation #ident<#(#arguments,)* #pg::mode::Expr> #clause {
                #(#accessors)*
            }
        }
    }
}
