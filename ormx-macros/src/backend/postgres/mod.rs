use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{backend::Backend, table::Table};

mod insert;

#[derive(Clone)]
pub struct PgBackend;

impl Backend for PgBackend {
    const QUOTE: char = '"';
    type Bindings = PgBindings;

    fn query_result() -> TokenStream {
        quote!(sqlx::postgres::PgQueryResult)
    }

    fn impl_insert(table: &Table<Self>) -> TokenStream {
        insert::impl_insert(table)
    }
}

#[derive(Default)]
pub struct PgBindings(usize);

impl Iterator for PgBindings {
    type Item = Cow<'static, str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 += 1;
        Some(Cow::Owned(format!("${}", self.0)))
    }
}
