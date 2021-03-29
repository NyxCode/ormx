use std::convert::TryFrom;

use proc_macro2::Span;
use syn::{Data, DeriveInput, Error, Ident, Result};

use crate::attrs::{parse_attrs, Insertable, TableAttr, TableFieldAttr};
use crate::utils::{missing_attr, set_once};

use super::{Table, TableField};
use crate::backend::Backend;
use std::marker::PhantomData;

macro_rules! none {
    ($($i:ident),*) => { $( let mut $i = None; )* };
}

impl<B: Backend> TryFrom<&syn::Field> for TableField<B> {
    type Error = Error;

    fn try_from(value: &syn::Field) -> Result<Self> {
        let ident = value.ident.clone().unwrap();

        let reserved_ident = B::RESERVED_IDENTS.contains(&&*ident.to_string().to_uppercase());
        if reserved_ident {
            proc_macro_error::emit_warning!(
                ident.span(),
                "This is a reserved keyword, you might want to consider choosing a different name."
            );
        }

        none!(
            column,
            custom_type,
            get_one,
            get_optional,
            get_many,
            set,
            default
        );

        for attr in parse_attrs::<TableFieldAttr>(&value.attrs)? {
            match attr {
                TableFieldAttr::Column(c) => set_once(&mut column, c)?,
                TableFieldAttr::CustomType(..) => set_once(&mut custom_type, true)?,
                TableFieldAttr::GetOne(g) => set_once(&mut get_one, g)?,
                TableFieldAttr::GetOptional(g) => set_once(&mut get_optional, g)?,
                TableFieldAttr::GetMany(g) => set_once(&mut get_many, g)?,
                TableFieldAttr::Set(s) => {
                    let default = || Ident::new(&format!("set_{}", ident), Span::call_site());
                    set_once(&mut set, s.unwrap_or_else(default))?
                }
                TableFieldAttr::Default(..) => set_once(&mut default, true)?,
            }
        }
        Ok(TableField {
            column_name: column.unwrap_or_else(|| ident.to_string()),
            field: ident,
            ty: value.ty.clone(),
            custom_type: custom_type.unwrap_or(false),
            reserved_ident,
            default: default.unwrap_or(false),
            get_one,
            get_optional,
            get_many,
            set,
            _phantom: PhantomData,
        })
    }
}

impl<B: Backend> TryFrom<&syn::DeriveInput> for Table<B> {
    type Error = Error;

    fn try_from(value: &DeriveInput) -> Result<Self> {
        let data = match &value.data {
            Data::Struct(s) => s,
            _ => panic!("not a struct with named fields"),
        };

        let fields = data
            .fields
            .iter()
            .map(TableField::try_from)
            .collect::<Result<Vec<_>>>()?;

        none!(table, id, insertable);
        for attr in parse_attrs::<TableAttr>(&value.attrs)? {
            match attr {
                TableAttr::Table(x) => set_once(&mut table, x)?,
                TableAttr::Id(x) => set_once(&mut id, x)?,
                TableAttr::Insertable(x) => {
                    let default = || Insertable {
                        attrs: vec![],
                        ident: Ident::new(&format!("Insert{}", value.ident), Span::call_site()),
                    };
                    set_once(&mut insertable, x.unwrap_or_else(default))?;
                }
            }
        }

        let id = id.ok_or_else(|| missing_attr("id"))?;
        let id = fields
            .iter()
            .find(|field| field.field == id)
            .ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    "id does not refer to a field of the struct",
                )
            })?
            .clone();

        if insertable.is_none() && fields.iter().any(|field| field.default) {
            return Err(Error::new(
                Span::call_site(),
                "#[ormx(default)] has no effect without #[ormx(insertable = ..)]",
            ));
        }

        Ok(Table {
            ident: value.ident.clone(),
            vis: value.vis.clone(),
            table: table.ok_or_else(|| missing_attr("table"))?,
            id,
            insertable,
            fields,
        })
    }
}
