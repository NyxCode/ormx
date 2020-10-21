use crate::patch::Patch;
use crate::table::Table;
use proc_macro2::TokenStream;
use std::borrow::Cow;

mod common;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mysql")]
pub type Implementation = mysql::MySqlBackend;
#[cfg(feature = "postgres")]
pub type Implementation = postgres::PgBackend;
#[cfg(feature = "sqlite")]
compile_error!("sqlite is currently not supported");

pub trait Backend: Sized {
    const QUOTE: char;
    type Bindings: Iterator<Item = Cow<'static, str>> + Default;

    /// Generate an `impl <Table>` block, containing getter methods
    fn impl_getters(table: &Table) -> TokenStream {
        common::getters::<Self>(table)
    }

    /// Generate an `impl <Table>` block, containing setter methods
    fn impl_setters(table: &Table) -> TokenStream {
        common::setters::<Self>(table)
    }

    /// Generate an `impl Table for <Table>` block
    fn impl_table(table: &Table) -> TokenStream {
        common::impl_table::<Self>(table)
    }

    /// Implement [Insert] for the helper struct for inserting
    fn impl_insert(table: &Table) -> TokenStream;

    /// Generate a helper struct for inserting
    fn insert_struct(table: &Table) -> TokenStream {
        common::insert_struct(table)
    }

    /// Implement [Patch]
    fn impl_patch(patch: &Patch) -> TokenStream {
        common::impl_patch::<Self>(patch)
    }
}
