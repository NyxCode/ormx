use crate::patch::Patch;
use crate::table::Table;
use proc_macro2::TokenStream;

mod mysql;

#[cfg(feature = "mysql")]
pub type Implementation = mysql::MySqlBackend;
#[cfg(feature = "postgres")]
compile_error!("postgres is currently not supported");
#[cfg(feature = "sqlite")]
compile_error!("sqlite is currently not supported");

pub trait Backend {
    fn getters(table: &Table) -> TokenStream;
    fn setters(table: &Table) -> TokenStream;
    fn implement_table(table: &Table) -> TokenStream;
    fn implement_insert(table: &Table) -> TokenStream;
    fn insert_struct(table: &Table) -> TokenStream;
    fn implement_patch(table: &Patch) -> TokenStream;
}
