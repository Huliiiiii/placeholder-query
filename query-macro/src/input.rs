use syn::{Data, DeriveInput, Fields, Generics, Ident, Type, Visibility};

pub(crate) struct Input<'a> {
    pub(crate) ident: &'a Ident,
    pub(crate) visibility: &'a Visibility,
    pub(crate) generics: &'a Generics,
    pub(crate) fields: Vec<Field<'a>>,
}

pub(crate) struct Field<'a> {
    pub(crate) ident: &'a Ident,
    pub(crate) ty: &'a Type,
    pub(crate) borrowed: bool,
}

pub(crate) fn parse(input: &DeriveInput) -> syn::Result<Input<'_>> {
    if input.generics.type_params().next().is_some()
        || input.generics.const_params().next().is_some()
    {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "QueryInput supports lifetime parameters, but not type or const parameters",
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
            .map(|field| {
                let (ty, borrowed) = match &field.ty {
                    Type::Reference(reference) if reference.mutability.is_none() => {
                        (&*reference.elem, true)
                    }
                    _ => (&field.ty, false),
                };
                Field {
                    ident: field.ident.as_ref().expect("named field"),
                    ty,
                    borrowed,
                }
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
        generics: &input.generics,
        fields,
    })
}
