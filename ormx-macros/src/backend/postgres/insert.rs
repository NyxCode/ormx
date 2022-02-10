use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{
    backend::postgres::{PgBackend, PgBindings},
    table::{Table, TableField},
};

fn insert_sql(table: &Table<PgBackend>, insert_fields: &[&TableField<PgBackend>]) -> String {
    let columns = insert_fields.iter().map(|field| field.column()).join(", ");
    let fields = PgBindings::default().take(insert_fields.len()).join(", ");
    let returning_fields = table
        .default_fields()
        .map(TableField::fmt_for_select)
        .join(", ");

    if returning_fields.is_empty() {
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table.table, columns, fields
        )
    } else {
        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            table.table, columns, fields, returning_fields
        )
    }
}

pub fn impl_insert(table: &Table<PgBackend>) -> TokenStream {
    let insert_ident = match &table.insertable {
        Some(i) => &i.ident,
        None => return quote!(),
    };

    let insert_fields: Vec<&TableField<PgBackend>> = table.insertable_fields().collect();
    let default_fields: Vec<&TableField<PgBackend>> = table.default_fields().collect();

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

    let insert_field_exprs = insert_fields
        .iter()
        .map(|f| f.fmt_as_argument())
        .collect::<Vec<TokenStream>>();

    let fetch_funtion = if default_fields.is_empty() {
        Ident::new("execute", Span::call_site())
    } else {
        Ident::new("fetch_one", Span::call_site())
    };

    let box_future = crate::utils::box_future();
    quote! {
        impl ormx::Insert for #insert_ident {
            type Table = #table_ident;

            fn insert<'a, 'c: 'a>(
                self,
                db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            ) -> #box_future<'a, sqlx::Result<Self::Table>> {
                Box::pin(async move {
                    let _generated = sqlx::query!(#insert_sql, #( #insert_field_exprs, )*)
                        .#fetch_funtion(db)
                        .await?;

                    Ok(Self::Table {
                        #( #insert_field_idents: self.#insert_field_idents, )*
                        #( #default_field_idents: _generated.#default_field_idents, )*
                    })
                })
            }
        }
    }
}
