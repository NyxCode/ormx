use crate::backend::postgres::{PgBindings, PgBackend};
use crate::table::{Table, TableField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

fn insert_sql(table: &Table, insert_fields: &[&TableField]) -> String {
    format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        table.table,
        insert_fields.iter().map(|field| &field.column).join(", "),
        PgBindings::default().take(insert_fields.len()).join(", "),
        table.id.fmt_for_select::<PgBackend>()
    )
}

fn query_default_sql(table: &Table, default_fields: &[&TableField]) -> String {
    format!(
        "SELECT {} FROM {} WHERE {} = {}",
        default_fields
            .iter()
            .map(|field| field.fmt_for_select::<PgBackend>())
            .join(", "),
        table.table,
        table.id.column,
        PgBindings::default().next().unwrap()
    )
}

pub fn impl_insert(table: &Table) -> TokenStream {
    let insert_ident = match &table.insertable {
        Some(i) => &i.ident,
        None => return quote!(),
    };

    let insert_fields: Vec<&TableField> = table.insertable_fields().collect();
    let default_fields: Vec<&TableField> = table.default_fields().collect();

    let id_ident = &table.id.field;
    let table_ident = &table.ident;
    let insert_field_idents = insert_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();
    let default_field_idents = default_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();

    let insert_sql = insert_sql(table, &insert_fields);

    let query_default_sql = query_default_sql(table, &default_fields);
    let query_default = if default_fields.is_empty() {
        quote!()
    } else {
        quote! {
            let _generated = sqlx::query!(#query_default_sql, _id)
                .fetch_one(db)
                .await?;
        }
    };

    let box_future = crate::utils::box_future();
    quote! {
        impl ormx::Insert for #insert_ident {
            type Table = #table_ident;

            fn insert(
                self,
                db: &mut sqlx::PgConnection,
            ) -> #box_future<sqlx::Result<Self::Table>> {
                Box::pin(async move {
                    let _id = sqlx::query!(#insert_sql, #( self.#insert_field_idents, )*)
                        .fetch_one(db as &mut sqlx::PgConnection)
                        .await?
                        .#id_ident;

                    #query_default

                    Ok(Self::Table {
                        #id_ident: _id as _,
                        #( #insert_field_idents: self.#insert_field_idents, )*
                        #( #default_field_idents: _generated.#default_field_idents, )*
                    })
                })
            }
        }
    }
}
