use std::borrow::Cow;

use proc_macro2::TokenStream;

use crate::backend::Backend;
use crate::table::Table;

use self::get_by_any::impl_get_by_any_getter;

use super::common;

mod get_by_any;
mod insert;

#[derive(Clone)]
pub struct PgBackend;

impl Backend for PgBackend {
    const QUOTE: char = '"';
    #[rustfmt::skip]
    const RESERVED_IDENTS: &'static [&'static str] = &[
        "ALL", "ANALYSE", "ANALYZE", "AND", "ANY", "ARRAY", "AS", "ASC", "ASYMMETRIC",
        "AUTHORIZATION", "BETWEEN", "BINARY", "BOTH", "CASE", "CAST", "CHECK", "COLLATE", "COLUMN",
        "CONSTRAINT", "CREATE", "CROSS", "CURRENT_DATE", "CURRENT_ROLE", "CURRENT_TIME",
        "CURRENT_TIMESTAMP", "CURRENT_USER", "DEFAULT", "DEFERRABLE", "DESC", "DISTINCT", "DO",
        "ELSE", "END", "EXCEPT", "FALSE", "FETCH", "FOR", "FOREIGN", "FROM", "FREEZE", "FULL",
        "GRANT", "GROUP", "HAVING", "ILIKE", "IN", "INITIALLY", "INNER", "INTERSECT", "INTO", "IS",
        "ISNULL", "JOIN", "LATERAL", "LEADING", "LEFT", "LIMIT", "LIKE", "LOCALTIME",
        "LOCALTIMESTAMP", "NATURAL", "NOT", "NOTNULL", "NULL", "OFFSET", "ON", "ONLY", "OR",
        "ORDER", "OUTER", "OVERLAPS", "PLACING", "PRIMARY", "REFERENCES", "RETURNING", "RIGHT",
        "SELECT", "SESSION_USER", "SIMILAR", "SOME", "SYMMETRIC", "TABLE", "TABLESAMPLE", "THEN",
        "TO", "TRAILING", "TRUE", "UNION", "UNIQUE", "USER", "USING", "VARIADIC", "VERBOSE", "WHEN",
        "WHERE", "WINDOW", "WITH"
    ];
    type Bindings = PgBindings;

    fn impl_insert(table: &Table<Self>) -> TokenStream {
        insert::impl_insert(table)
    }

    fn impl_getters(table: &Table<Self>) -> TokenStream {
        let mut getters = common::getters::<Self>(table);
        getters.extend(impl_get_by_any_getter(table));
        getters
    }
}

#[derive(Default)]
pub struct PgBindings(usize);

impl Iterator for PgBindings {
    type Item = Cow<'static, str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 += 1;
        Some(Cow::Owned(format!("${}", self.0)))
    }
}
