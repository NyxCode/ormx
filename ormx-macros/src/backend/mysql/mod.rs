use std::borrow::Cow;

use proc_macro2::TokenStream;

use crate::backend::Backend;
use crate::table::Table;

mod insert;

pub struct MySqlBackend;

impl Backend for MySqlBackend {
    const QUOTE: char = '`';
    type Bindings = MySqlBindings;

    fn impl_insert(table: &Table) -> TokenStream {
        insert::impl_insert(table)
    }
}

#[derive(Default)]
pub struct MySqlBindings;

impl Iterator for MySqlBindings {
    type Item = Cow<'static, str>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Cow::Borrowed("?"))
    }
}
