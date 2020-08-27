use crate::table::{Table, TableField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn impl_insert(table: &Table) -> TokenStream {
    let insert_ident = match &table.insertable {
        Some(i) => i,
        None => return quote!(),
    };

    let insert_fields: Vec<&TableField> = table.insertable_fields().collect();
    let default_fields: Vec<&TableField> = table.default_fields().collect();

    let id_ident = &table.id.field;
    let table_ident = &table.ident;
    let box_future = quote!(ormx::exports::futures::future::BoxFuture);
    let insert_field_idents = insert_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();
    let default_field_idents = default_fields
        .iter()
        .map(|field| &field.field)
        .collect::<Vec<&Ident>>();
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table.table,
        insert_fields.iter().map(|field| &field.column).join(", "),
        std::iter::repeat("?").take(insert_fields.len()).join(", ")
    );
    let query_default_sql = format!(
        "SELECT {} FROM {} WHERE {} = ?",
        default_fields
            .iter()
            .map(|field| field.fmt_for_select())
            .join(", "),
        table.table,
        table.id.column
    );
    let query_default = if default_fields.is_empty() {
        quote!()
    } else {
        quote! {
            let _generated = sqlx::query!(#query_default_sql, _id)
                .fetch_one(db)
                .await?;
        }
    };

    quote! {
        impl ormx::Insert for #insert_ident {
            type Table = #table_ident;

            fn insert(
                self,
                db: &mut sqlx::MySqlConnection,
            ) -> #box_future<sqlx::Result<Self::Table>> {
                Box::pin(async move {
                    sqlx::query!(#insert_sql, #( self.#insert_field_idents, )*)
                        .execute(db as &mut sqlx::MySqlConnection)
                        .await?;
                    let _id = sqlx::query!("SELECT LAST_INSERT_ID() as id")
                        .fetch_one(db as &mut sqlx::MySqlConnection)
                        .await?
                        .id;

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

pub fn insert_struct(table: &Table) -> TokenStream {
    let ident = match &table.insertable {
        Some(i) => i,
        None => return quote!(),
    };
    let vis = &table.vis;
    let insert_fields = table.insertable_fields().map(|field| {
        let ident = &field.field;
        let ty = &field.ty;
        quote!(#vis #ident: #ty)
    });

    quote! {
        #vis struct #ident {
            #( #insert_fields, )*
        }
    }
}
