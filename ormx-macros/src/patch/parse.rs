use std::convert::TryFrom;

use syn::{Data, DeriveInput, Error, Field, Result};

use super::Patch;
use crate::{
    attrs::{parse_attrs, PatchAttr, PatchFieldAttr},
    patch::PatchField,
    utils::{missing_attr, set_once},
};

impl TryFrom<&DeriveInput> for Patch {
    type Error = Error;

    fn try_from(value: &DeriveInput) -> Result<Self> {
        let data = match &value.data {
            Data::Struct(s) => s,
            _ => panic!("not a struct with named fields"),
        };

        let fields = data
            .fields
            .iter()
            .map(PatchField::try_from)
            .collect::<Result<Vec<_>>>()?;

        let mut table = None;
        let mut table_name = None;
        let mut id = None;
        for attr in parse_attrs::<PatchAttr>(&value.attrs)? {
            match attr {
                PatchAttr::Table(x) => set_once(&mut table, x)?,
                PatchAttr::TableName(x) => set_once(&mut table_name, x)?,
                PatchAttr::Id(x) => set_once(&mut id, x)?,
            }
        }

        Ok(Patch {
            ident: value.ident.clone(),
            table_name: table_name.ok_or_else(|| missing_attr("table_name"))?,
            table: table.ok_or_else(|| missing_attr("table"))?,
            id: id.ok_or_else(|| missing_attr("id"))?,
            fields,
        })
    }
}

impl TryFrom<&Field> for PatchField {
    type Error = Error;

    fn try_from(value: &Field) -> Result<Self> {
        let ident = value.ident.clone().unwrap();

        let mut column = None;
        let mut custom_type = None;
        let mut by_ref = None;
        for attr in parse_attrs::<PatchFieldAttr>(&value.attrs)? {
            match attr {
                PatchFieldAttr::Column(x) => set_once(&mut column, x)?,
                PatchFieldAttr::CustomType(_) => set_once(&mut custom_type, true)?,
                PatchFieldAttr::ByRef(_) => set_once(&mut by_ref, true)?,
            }
        }

        Ok(PatchField {
            ident: value.ident.clone().unwrap(),
            column: column.unwrap_or_else(|| ident.to_string()),
            ty: value.ty.clone(),
            custom_type: custom_type.unwrap_or(false),
            by_ref: by_ref.unwrap_or(false),
        })
    }
}
