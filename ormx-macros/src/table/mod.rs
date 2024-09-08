use std::{convert::TryFrom, marker::PhantomData};

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, DeriveInput, Result, Type, Visibility};

use crate::{
    attrs::{Getter, Insertable},
    backend::{Backend, Implementation},
};

mod parse;

pub struct Table<B: Backend> {
    pub ident: Ident,
    pub vis: Visibility,
    table: String,
    pub id: TableField<B>,
    pub fields: Vec<TableField<B>>,
    pub insertable: Option<Insertable>,
    pub deletable: bool,
    pub order_by: Option<String>,
}

#[derive(Clone)]
pub struct TableField<B: Backend> {
    pub field: Ident,
    pub ty: Type,
    column_name: String,
    pub custom_type: bool,
    pub default: bool,
    pub get_one: Option<Getter>,
    pub get_optional: Option<Getter>,
    pub get_many: Option<Getter>,
    pub set: Option<Ident>,
    pub by_ref: bool,
    pub insert_attrs: Vec<Attribute>,
    pub _phantom: PhantomData<*const B>,
}

impl<B: Backend> Table<B> {
    pub fn fields_except_id(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        let id = self.id.field.clone();
        self.fields.iter().filter(move |field| field.field != id)
    }

    pub fn insertable_fields(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        self.fields.iter().filter(|field| !field.default)
    }

    pub fn default_fields(&self) -> impl Iterator<Item = &TableField<B>> + Clone {
        self.fields.iter().filter(|field| field.default)
    }

    pub fn select_column_list(&self) -> String {
        self.fields
            .iter()
            .map(|field| field.fmt_for_select())
            .join(", ")
    }

    pub fn name(&self) -> String {
        let q = B::QUOTE;
        format!("{q}{}{q}", self.table)
    }
}

impl<B: Backend> TableField<B> {
    pub fn fmt_for_select(&self) -> String {
        let q = B::QUOTE;

        if self.custom_type {
            format!(
                "{q}{}{q} AS {q}{}!: {}{q}",
                self.column_name,
                self.field.to_string(),
                self.ty.to_token_stream()
            )
        } else if self.field == self.column_name {
            self.column()
        } else {
            format!("{q}{}{q} AS {q}{}{q}", self.column_name, self.field)
        }
    }

    pub fn fmt_as_argument(&self) -> TokenStream {
        let ident = &self.field;
        let ty = &self.ty;

        let mut out = quote!(self.#ident);
        let mut ty = quote!(#ty);
        if self.by_ref {
            out = quote!(&#out);
            ty = quote!(&#ty);
        }
        if self.custom_type {
            out = quote!(#out as #ty);
        }

        out
    }

    pub fn column(&self) -> String {
        let q = B::QUOTE;
        format!("{q}{}{q}", self.column_name)
    }
}

impl Getter {
    pub fn or_fallback<B: Backend>(&self, field: &TableField<B>) -> (Ident, Type) {
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

    let impl_table = Implementation::impl_table(&parsed);
    let delete = Implementation::impl_delete(&parsed);
    let insert_struct = Implementation::insert_struct(&parsed);
    let impl_insert = Implementation::impl_insert(&parsed);
    let getters = Implementation::impl_getters(&parsed);
    let setters = Implementation::impl_setters(&parsed);

    Ok(quote! {
        #impl_table
        #delete
        #insert_struct
        #impl_insert
        #getters
        #setters
    })
}
