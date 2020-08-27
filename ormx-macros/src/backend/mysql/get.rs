use crate::table::Table;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

pub fn getters(table: &Table) -> TokenStream {
    let column_list = table.column_list();
    let vis = &table.vis;
    let mut getters = TokenStream::new();

    for field in table.fields.iter() {
        let sql = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            column_list, table.table, field.column
        );

        if let Some(getter) = &field.get_one {
            let (func, arg) = getter.or_fallback(&field);
            getters.extend(get_one(vis, &func, &arg, &sql));
        }

        if let Some(getter) = &field.get_optional {
            let (func, arg) = getter.or_fallback(&field);
            getters.extend(get_optional(vis, &func, &arg, &sql));
        }

        if let Some(getter) = &field.get_many {
            let (func, arg) = getter.or_fallback(&field);
            getters.extend(get_many(vis, &func, &arg, &sql));
        }
    }

    let table_ident = &table.ident;
    quote! {
        impl #table_ident {
            #getters
        }
    }
}

fn get_one(vis: &Visibility, ident: &Ident, by_ty: &Type, sql: &str) -> TokenStream {
    quote! {
        #vis async fn #ident(
            db: impl sqlx::Executor<'_, Database = ormx::Db>,
            by: #by_ty,
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #sql, by)
                .fetch_one(db)
                .await
        }
    }
}

fn get_optional(vis: &Visibility, ident: &Ident, by_ty: &Type, sql: &str) -> TokenStream {
    quote! {
        #vis async fn #ident(
            db: impl sqlx::Executor<'_, Database = ormx::Db>,
            by: #by_ty,
        ) -> sqlx::Result<Option<Self>> {
            sqlx::query_as!(Self, #sql, by)
                .fetch_optional(db)
                .await
        }
    }
}

fn get_many(vis: &Visibility, ident: &Ident, by_ty: &Type, sql: &str) -> TokenStream {
    quote! {
        #vis async fn #ident(
            db: impl sqlx::Executor<'_, Database = ormx::Db>,
            by: #by_ty,
        ) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as!(Self, #sql, by)
                .fetch_all(db)
                .await
        }
    }
}
