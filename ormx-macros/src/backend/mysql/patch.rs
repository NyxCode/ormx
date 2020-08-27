use crate::patch::Patch;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn impl_patch(patch: &Patch) -> TokenStream {
    let patch_ident = &patch.ident;
    let table_path = &patch.table;
    let field_idents = &patch
        .fields
        .iter()
        .map(|field| &field.ident)
        .collect::<Vec<&Ident>>();
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        &patch.table_name,
        patch
            .fields
            .iter()
            .map(|field| format!("{} = ?", field.column))
            .join(", "),
        patch.id
    );

    let box_future = crate::utils::box_future();
    quote! {
        impl ormx::Patch for #patch_ident {
            type Table = #table_path;

            fn apply_to(self, entity: &mut Self::Table) {
                #( entity.#field_idents = self.#field_idents; )*
            }

            fn patch_row<'a, 'c: 'a>(
                &'a self,
                db: impl sqlx::Executor<'c, Database = ormx::Db> + 'a,
                id: <Self::Table as ormx::Table>::Id,
            ) -> #box_future<'a, sqlx::Result<()>> {
                Box::pin(async move {
                    sqlx::query!(#sql, #( self.#field_idents, )* id)
                        .execute(db)
                        .await?;
                    Ok(())
                })
            }
        }
    }
}
