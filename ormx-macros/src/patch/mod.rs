use crate::backend::{Backend, Implementation};
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use syn::{DeriveInput, Ident, Path, Result, Type};

mod parse;

pub struct Patch {
    pub ident: Ident,
    pub table_name: String,
    pub table: Path,
    pub id: String,
    pub fields: Vec<PatchField>,
}

pub struct PatchField {
    pub ident: Ident,
    pub column: String,
    pub ty: Type,
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let parsed = Patch::try_from(&input)?;
    Ok(Implementation::impl_patch(&parsed))
}
