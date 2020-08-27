mod get;
mod insert;
mod patch;
mod set;
mod table;

use crate::backend::Backend;
use crate::patch::Patch;
use crate::table::Table;
use proc_macro2::TokenStream;

pub struct MySqlBackend;

impl Backend for MySqlBackend {
    fn getters(table: &Table) -> TokenStream {
        get::getters(table)
    }

    fn setters(table: &Table) -> TokenStream {
        set::setters(table)
    }

    fn implement_table(table: &Table) -> TokenStream {
        table::impl_table(table)
    }

    fn implement_insert(table: &Table) -> TokenStream {
        insert::impl_insert(table)
    }

    fn insert_struct(table: &Table) -> TokenStream {
        insert::insert_struct(table)
    }

    fn implement_patch(table: &Patch) -> TokenStream {
        patch::impl_patch(table)
    }
}
