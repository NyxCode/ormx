use std::convert::TryFrom;

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result, Type, Visibility};

use crate::attrs::{Getter, Insertable};
use crate::backend::{Backend, Implementation};
use std::borrow::Cow;
use std::marker::PhantomData;

mod parse;

pub struct Table<B: Backend> {
    pub ident: Ident,
    pub vis: Visibility,
    pub table: String,
    pub id: TableField<B>,
    pub fields: Vec<TableField<B>>,
    pub insertable: Option<Insertable>,
}

#[derive(Clone)]
pub struct TableField<B: Backend> {
    pub field: Ident,
    pub ty: Type,
    pub column_name: String,
    pub custom_type: bool,
    pub reserved_ident: bool,
    pub default: bool,
    pub get_one: Option<Getter>,
    pub get_optional: Option<Getter>,
    pub get_many: Option<Getter>,
    pub set: Option<Ident>,
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
}

impl<B: Backend> TableField<B> {
    pub fn fmt_for_select(&self) -> String {
        if self.custom_type {
            format!(
                "{} AS {}{}: _{}",
                self.column(),
                B::QUOTE,
                self.field,
                B::QUOTE
            )
        } else if self.field == self.column_name {
            self.column().into()
        } else {
            format!("{} AS {}", self.column(), self.field)
        }
    }

    pub fn column(&self) -> Cow<str> {
        if self.reserved_ident {
            format!("{}{}{}", B::QUOTE, self.column_name, B::QUOTE).into()
        } else {
            Cow::Borrowed(&self.column_name)
        }
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
    let insert_struct = Implementation::insert_struct(&parsed);
    let impl_insert = Implementation::impl_insert(&parsed);
    let getters = Implementation::impl_getters(&parsed);
    let setters = Implementation::impl_setters(&parsed);

    Ok(quote! {
        #impl_table
        #insert_struct
        #impl_insert
        #getters
        #setters
    })
}
