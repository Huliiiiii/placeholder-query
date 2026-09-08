mod input;
mod query_input;
mod row;
mod table;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Defines a query row.
///
/// Requires a named fields struct.
///
/// Use `#[row(derive(Clone, Debug))]` to derive traits with bounds on field types.
#[proc_macro_attribute]
pub fn row(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut options = row::Args::default();
    let parser = syn::meta::parser(|meta| options.parse(meta));
    parse_macro_input!(args with parser);
    match row::expand(options, parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Defines a query row of a database table.
///
/// Provides the same behavior as [`row`] and adds table-specific methods:
/// - `table()`: returns the table schema.
///
/// Use `#[table(name = "...")]` to override the table name.
/// Use `#[table(derive(Clone, Debug))]` to derive traits with bounds on field types.
#[proc_macro_attribute]
pub fn table(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut options = table::Args::default();
    let parser = syn::meta::parser(|meta| options.parse(meta));
    parse_macro_input!(args with parser);
    match table::expand(options, parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Derives typed query parameters for a unit or named-field struct.
///
/// Generic structs are not supported.
#[proc_macro_derive(QueryInput)]
pub fn derive_query_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match query_input::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
