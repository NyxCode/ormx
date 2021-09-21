use std::convert::TryFrom;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident, Path, Result, Type};

use crate::backend::{Backend, Implementation};

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
    pub custom_type: bool
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let parsed = Patch::try_from(&input)?;
    Ok(Implementation::impl_patch(&parsed))
}
