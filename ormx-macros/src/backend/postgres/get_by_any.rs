use crate::{
    backend::{common, postgres::PgBindings},
    table::Table,
};

use super::PgBackend;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn impl_get_by_any_getter(table: &Table<PgBackend>) -> TokenStream {
    let column_list = table.select_column_list();
    let vis = &table.vis;
    let mut getters = TokenStream::new();

    for field in table.fields.iter() {
        if let Some(getter) = &field.get_by_any {
            let sql = format!(
                "SELECT {} FROM {} WHERE {} = ANY({})",
                column_list,
                table.table,
                field.column(),
                PgBindings::default().next().unwrap()
            );

            let func = getter.ident_or(&field, &format!("get_by_any_{}", field.field));
            let arg = getter.arg_ty.clone().unwrap_or_else(|| {
                let ty = &field.ty;
                syn::parse2(quote!(&[#ty])).unwrap()
            });

            getters.extend(common::get_many(vis, &func, &arg, &sql));
        }
    }

    let table_ident = &table.ident;
    quote! {
        impl #table_ident {
            #getters
        }
    }
}
