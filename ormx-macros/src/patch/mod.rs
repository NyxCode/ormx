use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::quote;
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
    pub custom_type: bool,
    pub by_ref: bool,
}

impl PatchField {
    pub fn fmt_as_argument(&self) -> TokenStream {
        let ident = &self.ident;
        // let ty = &self.ty;

        let mut out = quote!(#ident);
        if self.custom_type {
            // note: removed 'as #ty' to avoid creating a temporary
            // that is dropped before the stream is finished.
            out = quote!(#out);
        }
        if self.by_ref {
            out = quote!(&(#out));
        }
        out
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let parsed = Patch::try_from(&input)?;
    Ok(Implementation::impl_patch(&parsed))
}
