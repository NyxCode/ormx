use crate::table::Table;
use proc_macro2::TokenStream;
use quote::quote;

pub fn setters(table: &Table) -> TokenStream {
    let vis = &table.vis;
    let mut setters = TokenStream::new();

    for field in table.fields.iter() {
        let field_ident = &field.field;
        let field_ty = &field.ty;

        if let Some(fn_name) = &field.set {
            let sql = format!(
                "UPDATE {} SET {} = ? WHERE {} = ?",
                table.table, field.column, table.id.column
            );
            setters.extend(quote! {
                #vis async fn #fn_name(
                    &mut self,
                    db: impl sqlx::Executor<'_, Database = ormx::Db>,
                    value: #field_ty
                ) -> sqlx::Result<()> {
                    sqlx::query!(#sql, value, <Self as ormx::Table>::id(self))
                        .execute(db)
                        .await?;
                    self.#field_ident = value;
                    Ok(())
                }
            })
        }
    }

    let table_ident = &table.ident;
    quote! {
        impl #table_ident {
            #setters
        }
    }
}
