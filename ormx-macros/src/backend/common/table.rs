use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    backend::Backend,
    table::{Table, TableField},
};

pub fn impl_table<B: Backend>(table: &Table<B>) -> TokenStream {
    let table_ident = &table.ident;
    let id_ident = &table.id.field;
    let id_ty = &table.id.ty;
    let column_list = table.select_column_list();

    let get = get::<B>(table, &column_list);
    let stream_all = stream_all(table, &column_list);
    let stream_all_paginated = stream_all_paginated::<B>(table, &column_list);
    let update = update::<B>(table);

    quote! {
        impl ormx::Table for #table_ident {
            type Id = #id_ty;

            fn id(&self) -> Self::Id { self.#id_ident }

            #get
            #stream_all
            #stream_all_paginated
            #update
        }
    }
}

pub fn impl_delete<B: Backend>(table: &Table<B>) -> TokenStream {
    if !table.deletable {
        return quote!();
    }

    let table_ident = &table.ident;
    let delete = delete::<B>(table);

    quote! {
        impl ormx::Delete for #table_ident {
            #delete
        }
    }
}

fn get<B: Backend>(table: &Table<B>, column_list: &str) -> TokenStream {
    let get_sql = format!(
        "SELECT {} FROM {} WHERE {} = {}",
        column_list,
        table.name(),
        table.id.column(),
        B::Bindings::default().next().unwrap()
    );

    quote! {
        async fn get<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            id: Self::Id,
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #get_sql, id)
                .fetch_one(db)
                .await
        }
    }
}

fn update<B: Backend>(table: &Table<B>) -> TokenStream {
    let mut bindings = B::Bindings::default();
    let mut assignments = vec![];
    for field in table.fields_except_id() {
        let fragment = format!("{} = {}", field.column(), bindings.next().unwrap());
        assignments.push(fragment);
    }
    let assignments = assignments.join(", ");

    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = {}",
        table.name(),
        assignments,
        table.id.column(),
        bindings.next().unwrap()
    );
    let id_argument = &table.id.field;
    let other_arguments = table.fields_except_id().map(TableField::fmt_as_argument);

    quote! {
        async fn update<'a, 'c: 'a>(
            &'a self,
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> sqlx::Result<()> {
            sqlx::query!(#update_sql, #( #other_arguments, )* self.#id_argument)
                .execute(db)
                .await?;
            Ok(())
        }
    }
}

fn stream_all<B: Backend>(table: &Table<B>, column_list: &str) -> TokenStream {
    let return_type = crate::utils::stream!(sqlx::Result<Self>);
    let order_by = match &table.order_by {
        None => &format!("{} DESC", table.id.column()),
        Some(by) => by,
    };
    let all_sql = format!(
        "SELECT {} FROM {} ORDER BY {order_by}",
        column_list,
        table.name()
    );

    quote! {
        fn stream_all<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #return_type + 'a {
            sqlx::query_as!(Self, #all_sql)
                .fetch(db)
        }
    }
}

fn stream_all_paginated<B: Backend>(table: &Table<B>, column_list: &str) -> TokenStream {
    let return_type = crate::utils::stream!(sqlx::Result<Self>);
    let mut bindings = B::Bindings::default();
    let order_by = match &table.order_by {
        None => &format!("{} DESC", table.id.column()),
        Some(by) => by,
    };
    let all_sql = format!(
        "SELECT {} FROM {} ORDER BY {order_by} LIMIT {} OFFSET {} ",
        column_list,
        table.name(),
        bindings.next().unwrap(),
        bindings.next().unwrap()
    );

    quote! {
        fn stream_all_paginated<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            offset: i64,
            limit: i64,
        ) -> #return_type + 'a {
            sqlx::query_as!(Self, #all_sql, limit, offset)
                .fetch(db)
        }
    }
}

fn delete<B: Backend>(table: &Table<B>) -> TokenStream {
    let id_ty = &table.id.ty;
    let delete_sql = format!(
        "DELETE FROM {} WHERE {} = {}",
        table.name(),
        table.id.column(),
        B::Bindings::default().next().unwrap()
    );
    let query_result = B::query_result();

    quote! {
        async fn delete_row<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            id: #id_ty
        ) -> sqlx::Result<()> {
            use #query_result;

            let result = sqlx::query!(#delete_sql, id)
                .execute(db)
                .await?;
            if result.rows_affected() == 0 {
                Err(sqlx::Error::RowNotFound)
            } else {
                Ok(())
            }
        }
    }
}
