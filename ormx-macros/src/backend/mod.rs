use std::borrow::Cow;

use proc_macro2::TokenStream;

use crate::{patch::Patch, table::Table};

mod common;
#[cfg(feature = "mariadb")]
mod mariadb;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mariadb")]
pub type Implementation = mariadb::MariaBackend;
#[cfg(feature = "mysql")]
pub type Implementation = mysql::MySqlBackend;
#[cfg(feature = "postgres")]
pub type Implementation = postgres::PgBackend;
#[cfg(feature = "sqlite")]
compile_error!("sqlite is currently not supported");

pub trait Backend: Sized + Clone {
    const QUOTE: char;
    const IS_MYSQL: bool = false;

    type Bindings: Iterator<Item = Cow<'static, str>> + Default;

    /// Returns the type used to return query results, e.g `sqlx::postgres::PgQueryResult`
    fn query_result() -> TokenStream;

    /// Generate an `impl <Table>` block, containing getter methods
    fn impl_getters(table: &Table<Self>) -> TokenStream {
        common::getters::<Self>(table)
    }

    /// Generate an `impl <Table>` block, containing setter methods
    fn impl_setters(table: &Table<Self>) -> TokenStream {
        common::setters::<Self>(table)
    }

    /// Generate an `impl Table for <Table>` block
    fn impl_table(table: &Table<Self>) -> TokenStream {
        common::impl_table::<Self>(table)
    }

    /// Generate an `impl Delete for <Table>` block
    fn impl_delete(table: &Table<Self>) -> TokenStream {
        common::impl_delete::<Self>(table)
    }

    /// Implement [Insert] for the helper struct for inserting
    fn impl_insert(table: &Table<Self>) -> TokenStream;

    /// Generate a helper struct for inserting
    fn insert_struct(table: &Table<Self>) -> TokenStream {
        common::insert_struct(table)
    }

    /// Implement [Patch]
    fn impl_patch(patch: &Patch) -> TokenStream {
        common::impl_patch::<Self>(patch)
    }
}
