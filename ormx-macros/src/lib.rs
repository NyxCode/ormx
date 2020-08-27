#![cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]

mod attrs;
mod backend;
mod patch;
mod table;
mod utils;

#[proc_macro_derive(Table, attributes(ormx))]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match table::derive(input) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Patch, attributes(ormx))]
pub fn derive_patch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match patch::derive(input) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error(),
    }
    .into()
}
