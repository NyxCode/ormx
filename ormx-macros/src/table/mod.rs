use crate::attrs::{Getter, Insertable};
use crate::backend::{Backend, Implementation};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use std::convert::TryFrom;
use syn::{DeriveInput, Result, Type, Visibility};

mod parse;

pub struct Table {
    pub ident: Ident,
    pub vis: Visibility,
    pub table: String,
    pub id: TableField,
    pub fields: Vec<TableField>,
    pub insertable: Option<Insertable>,
}

#[derive(Clone)]
pub struct TableField {
    pub field: Ident,
    pub ty: Type,
    pub column: String,
    pub custom_type: bool,
    pub default: bool,
    pub get_one: Option<Getter>,
    pub get_optional: Option<Getter>,
    pub get_many: Option<Getter>,
    pub set: Option<Ident>,
}

impl Table {
    pub fn fields_except_id(&self) -> impl Iterator<Item = &TableField> + Clone {
        let id = self.id.field.clone();
        self.fields.iter().filter(move |field| field.field != id)
    }

    pub fn insertable_fields(&self) -> impl Iterator<Item = &TableField> + Clone {
        self.fields_except_id().filter(|field| !field.default)
    }

    pub fn default_fields(&self) -> impl Iterator<Item = &TableField> + Clone {
        self.fields.iter().filter(|field| field.default)
    }

    pub fn column_list(&self) -> String {
        self.fields
            .iter()
            .map(|field| field.fmt_for_select())
            .join(", ")
    }
}

impl TableField {
    pub fn fmt_for_select(&self) -> String {
        if self.custom_type {
            format!("{} AS `{}: _`", self.column, self.field)
        } else if self.column == self.field.to_string() {
            self.column.clone()
        } else {
            format!("{} AS {}", self.column, self.field)
        }
    }
}

impl Getter {
    pub fn or_fallback(&self, field: &TableField) -> (Ident, Type) {
        let ident = self
            .func
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("by_{}", field.field), Span::call_site()));
        let arg = self.arg_ty.clone().unwrap_or_else(|| {
            let ty = &field.ty;
            syn::parse2(quote!(&#ty)).unwrap()
        });
        (ident, arg)
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let parsed = Table::try_from(&input)?;

    let impl_table = Implementation::implement_table(&parsed);
    let insert_struct = Implementation::insert_struct(&parsed);
    let impl_insert = Implementation::implement_insert(&parsed);
    let getters = Implementation::getters(&parsed);
    let setters = Implementation::setters(&parsed);

    Ok(quote! {
        #impl_table
        #insert_struct
        #impl_insert
        #getters
        #setters
    })
}
