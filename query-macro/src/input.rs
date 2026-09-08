use syn::{Data, DeriveInput, Fields, Ident, Type, Visibility};

pub(crate) struct Input<'a> {
    pub(crate) ident: &'a Ident,
    pub(crate) visibility: &'a Visibility,
    pub(crate) fields: Vec<Field<'a>>,
}

pub(crate) struct Field<'a> {
    pub(crate) ident: &'a Ident,
    pub(crate) ty: &'a Type,
}

pub(crate) fn parse(input: &DeriveInput) -> syn::Result<Input<'_>> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "QueryInput does not support generic structs",
        ));
    }

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "QueryInput requires a unit struct or a struct with named fields",
        ));
    };

    let ident = &input.ident;
    let fields = match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| Field {
                ident: field.ident.as_ref().expect("named field"),
                ty: &field.ty,
            })
            .collect(),
        Fields::Unit => Vec::new(),
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "QueryInput requires a unit struct or named fields",
            ));
        }
    };

    Ok(Input {
        ident,
        visibility: &input.vis,
        fields,
    })
}
