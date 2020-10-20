use crate::backend::Backend;
use crate::table::Table;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_table<B: Backend>(table: &Table) -> TokenStream {
    let table_ident = &table.ident;
    let id_ident = &table.id.field;
    let id_ty = &table.id.ty;
    let column_list = table.column_list();

    let get = get::<B>(table, &column_list);
    let stream_all = stream_all(table, &column_list);
    let update = update::<B>(table);
    let delete = delete::<B>(table);

    quote! {
        impl ormx::Table for #table_ident {
            type Id = #id_ty;

            fn id(&self) -> Self::Id { self.#id_ident }

            #get
            #stream_all
            #update
            #delete
        }
    }
}

fn get<B: Backend>(table: &Table, column_list: &str) -> TokenStream {
    let box_future = crate::utils::box_future();
    let get_sql = format!(
        "SELECT {} FROM {} WHERE {} = {}",
        column_list,
        table.table,
        table.id.column,
        B::Bindings::default().next().unwrap()
    );

    quote! {
        fn get<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
            id: Self::Id,
        ) -> #box_future<'a, sqlx::Result<Self>> {
            Box::pin(async move {
                sqlx::query_as!(Self, #get_sql, id)
                    .fetch_one(db)
                    .await
            })
        }
    }
}

fn update<B: Backend>(table: &Table) -> TokenStream {
    let box_future = crate::utils::box_future();
    let id_ident = &table.id.field;
    let idents_except_id = table.fields_except_id().map(|field| &field.field);

    let mut bindings = B::Bindings::default();
    let mut assignments = vec![];
    for field in table.fields_except_id() {
        let fragment = format!("{} = {}", field.column, bindings.next().unwrap());
        assignments.push(fragment);
    }
    let assignments = assignments.join(", ");

    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = {}",
        table.table,
        assignments,
        table.id.column,
        bindings.next().unwrap()
    );

    quote! {
        fn update<'a, 'c: 'a>(
            &'a self,
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_future<'a, sqlx::Result<()>> {
            Box::pin(async move {
                sqlx::query!(#update_sql, #( self.#idents_except_id, )* self.#id_ident)
                    .execute(db)
                    .await?;
                Ok(())
            })
        }
    }
}

fn stream_all(table: &Table, column_list: &str) -> TokenStream {
    let box_stream = crate::utils::box_stream();
    let all_sql = format!("SELECT {} FROM {}", column_list, table.table);

    quote! {
        fn stream_all<'a, 'c: 'a>(
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_stream<'a, sqlx::Result<Self>> {
            sqlx::query_as!(Self, #all_sql)
                .fetch(db)
        }
    }
}

fn delete<B: Backend>(table: &Table) -> TokenStream {
    let id_ident = &table.id.field;
    let box_future = crate::utils::box_future();
    let delete_sql = format!(
        "DELETE FROM {} WHERE {} = {}",
        table.table,
        table.id.column,
        B::Bindings::default().next().unwrap()
    );

    quote! {
        fn delete<'a, 'c: 'a>(
            self,
            db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
        ) -> #box_future<'a, sqlx::Result<()>> {
            Box::pin(async move {
                sqlx::query!(#delete_sql, self.#id_ident)
                    .execute(db)
                    .await?;
                Ok(())
            })
        }
    }
}
