use std::{convert::TryFrom, marker::PhantomData};

use proc_macro2::Span;
use syn::{ext::IdentExt, Data, DeriveInput, Error, Ident, Result};

use super::{Table, TableField};
use crate::{
    attrs::{parse_attrs, Insertable, TableAttr, TableFieldAttr},
    backend::Backend,
    utils::{missing_attr, set_once},
};

macro_rules! none {
    ($($i:ident),*) => { $( let mut $i = None; )* };
}

impl<B: Backend> TryFrom<&syn::Field> for TableField<B> {
    type Error = Error;

    fn try_from(value: &syn::Field) -> Result<Self> {
        let ident = value.ident.clone().unwrap();

        none!(
            column,
            custom_type,
            get_one,
            get_optional,
            get_many,
            set,
            default,
            by_ref
        );
        let mut insert_attrs = vec![];

        for attr in parse_attrs::<TableFieldAttr>(&value.attrs)? {
            match attr {
                TableFieldAttr::Column(c) => set_once(&mut column, c)?,
                TableFieldAttr::CustomType(..) => set_once(&mut custom_type, true)?,
                TableFieldAttr::GetOne(g) => set_once(&mut get_one, g)?,
                TableFieldAttr::GetOptional(g) => set_once(&mut get_optional, g)?,
                TableFieldAttr::GetMany(g) => set_once(&mut get_many, g)?,
                TableFieldAttr::Set(s) => {
                    let default =
                        || Ident::new(&format!("set_{}", ident.unraw()), Span::call_site());
                    set_once(&mut set, s.unwrap_or_else(default))?
                }
                TableFieldAttr::Default(..) => set_once(&mut default, true)?,
                TableFieldAttr::ByRef(..) => set_once(&mut by_ref, true)?,
                TableFieldAttr::InsertAttr(mut attr) => insert_attrs.append(&mut attr.0),
            }
        }
        Ok(TableField {
            column_name: column.unwrap_or_else(|| ident.unraw().to_string()),
            field: ident,
            ty: value.ty.clone(),
            custom_type: custom_type.unwrap_or(false),
            default: default.unwrap_or(false),
            get_one,
            get_optional,
            get_many,
            set,
            by_ref: by_ref.unwrap_or(false),
            insert_attrs,
            _phantom: PhantomData,
        })
    }
}

impl<B: Backend> TryFrom<&DeriveInput> for Table<B> {
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

        none!(table, id, insertable, deletable, order_by);
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
                TableAttr::Deletable(_) => set_once(&mut deletable, true)?,
                TableAttr::OrderBy(by) => set_once(&mut order_by, by)?,
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
            deletable: deletable.unwrap_or(false),
            order_by,
        })
    }
}
