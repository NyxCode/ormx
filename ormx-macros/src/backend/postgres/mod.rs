mod insert;

use crate::backend::Backend;
use crate::table::Table;
use proc_macro2::TokenStream;
use std::borrow::Cow;

pub struct PgBackend;

impl Backend for PgBackend {
    const QUOTE: char = '"';
    type Bindings = PgBindings;

    fn impl_insert(table: &Table) -> TokenStream {
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
