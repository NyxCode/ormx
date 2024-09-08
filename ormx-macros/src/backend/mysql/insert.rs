use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use super::MySqlBackend;
use crate::{
    backend::mysql::MySqlBindings,
    table::{Table, TableField},
};

pub fn impl_insert(table: &Table<MySqlBackend>) -> TokenStream {
    let insert_ident = match &table.insertable {
        Some(i) => &i.ident,
        None => return quote!(),
    };

    let table_ident = &table.ident;

    let insert = insert(table);
    let query_id = query_id(table);
    let query_default = query_default(table);
    let construct_row = construct_row(table);

    quote! {
        impl ormx::Insert for #insert_ident {
            type Table = #table_ident;

            async fn insert<'a, 'c: 'a>(
                self,
                db: &'c mut sqlx::MySqlConnection,
            ) -> sqlx::Result<Self::Table> {
                use sqlx::Connection;

                let mut tx = db.begin().await?;
                #insert
                #query_id
                #query_default
                tx.commit().await?;
                Ok(#construct_row)
            }
        }
    }
}

/// build an instance of the table struct from
/// - `_id` (see `query_id` below)
/// - `_generated` (see `query_default` below)
/// - all fields already present in the insert struct
fn construct_row(table: &Table<MySqlBackend>) -> TokenStream {
    let id_ident = &table.id.field;
    let insert_field_idents = table
        .insertable_fields()
        .map(|f| &f.field)
        .filter(|f| *f != id_ident);
    let default_field_idents = table
        .default_fields()
        .map(|f| &f.field)
        .filter(|f| *f != id_ident);

    quote! {
        Self::Table {
            #id_ident: _id as _,
            #( #insert_field_idents: self.#insert_field_idents, )*
            #( #default_field_idents: _generated.#default_field_idents, )*
        }
    }
}

/// queries default fields from the database, except the ID.
fn query_default(table: &Table<MySqlBackend>) -> TokenStream {
    let mut default_fields = table
        .default_fields()
        .filter(|f| f.field != table.id.field)
        .peekable();

    if default_fields.peek().is_none() {
        return quote!();
    }

    let query_default_sql = format!(
        "SELECT {} FROM {} WHERE {} = ?",
        default_fields.map(TableField::fmt_for_select).join(", "),
        table.name(),
        table.id.column()
    );

    quote! {
        let _generated = sqlx::query!(#query_default_sql, _id)
            .fetch_one(&mut *tx)
            .await?;
    }
}

/// inserts the struct into the database
fn insert(table: &Table<MySqlBackend>) -> TokenStream {
    let insert_fields: Vec<_> = table.insertable_fields().collect();
    let insert_field_idents = insert_fields.iter().map(|field| &field.field);

    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table.name(),
        insert_fields.iter().map(|field| field.column()).join(", "),
        MySqlBindings.take(insert_fields.len()).join(", ")
    );

    quote! {
        sqlx::query!(#insert_sql, #( self.#insert_field_idents, )*)
            .execute(&mut *tx)
            .await?;
    }
}

/// obtains the id of the inserted row.
///
/// case 1:
///     The ID is database generated, so we query it with LAST_INSERT_ID
/// case 2:
///     The ID is already known, so we can just use it.
fn query_id(table: &Table<MySqlBackend>) -> TokenStream {
    match table.id.default {
        true => quote! {
            let _id = sqlx::query!("SELECT LAST_INSERT_ID() AS id")
                .fetch_one(&mut *tx)
                .await?
                .id;
        },
        false => {
            let id_ident = &table.id.field;
            quote!(let _id = self.#id_ident;)
        }
    }
}
