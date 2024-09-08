#![cfg(any(
    feature = "mysql",
    feature = "postgres",
    feature = "sqlite",
    feature = "mariadb"
))]
//! Lightweight derive macros for bringing orm-like features to sqlx.
//!
//! # Example: Table
//! ```rust,ignore
//! #[derive(ormx::Table)]
//! #[ormx(table = "users", id = user_id, insertable)]
//! struct User {
//!     #[ormx(column = "id")]
//!     user_id: u32,
//!     first_name: String,
//!     last_name: String,
//!     #[ormx(get_optional(&str))]
//!     email: String,
//!     #[ormx(default, set)]
//!     last_login: Option<NaiveDateTime>,
//! }
//! ```
//!
//! # Example: Patch
//! ```rust,ignore
//! #[derive(ormx::Patch)]
//! #[ormx(table_name = "users", table = User, id = "id")]
//! struct UpdateName {
//!     first_name: String,
//!     last_name: String,
//! }
//! ```
//!
//! # Documentation
//! See the docs of [derive(Table)](derive.Table.html) and [Patch](trait.Patch.html).

use std::future::Future;

use futures::{Stream, TryStreamExt};
pub use ormx_macros::*;
use sqlx::{Executor, Result};

#[doc(hidden)]
pub mod exports {
    pub use futures::Stream;

    pub use crate::query2::map::*;
}

#[cfg(any(feature = "mysql", feature = "postgres", feature = "mariadb"))]
mod query2;

#[cfg(any(feature = "mysql", feature = "mariadb"))]
pub type Db = sqlx::MySql;
#[cfg(feature = "postgres")]
pub type Db = sqlx::Postgres;
#[cfg(feature = "sqlite")]
pub type Db = sqlx::Sqlite;

/// A database table in which each row is identified by a unique ID.
pub trait Table
where
    Self: Sized + Send + Sync + 'static,
{
    /// Type of the ID column of this table.
    type Id: 'static + Copy + Send;

    /// Returns the id of this row.
    fn id(&self) -> Self::Id;

    /// Insert a row into the database.
    fn insert<'a, 'c: 'a>(
        #[cfg(not(feature = "mysql"))] db: impl Executor<'c, Database = Db> + 'a,
        #[cfg(feature = "mysql")] db: &'c mut sqlx::MySqlConnection,
        row: impl Insert<Table = Self>,
    ) -> impl Future<Output = Result<Self>> + Send + 'a {
        row.insert(db)
    }

    /// Queries the row of the given id.
    fn get<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> impl Future<Output = Result<Self>> + Send + 'a;

    /// Stream all rows from this table.
    /// By default, results are ordered in descending order according to their ID column.
    /// This can be configured using `#[ormx(order_by = "some_column ASC")]`.
    fn stream_all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Stream<Item = Result<Self>> + Send + 'a;

    /// Streams at most `limit` rows from this table, skipping the first `offset` rows.
    /// By default, results are ordered in descending order according to their ID column.
    /// This can be configured using `#[ormx(order_by = "some_column ASC")]`.
    fn stream_all_paginated<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        offset: i64,
        limit: i64,
    ) -> impl Stream<Item = Result<Self>> + Send + 'a;

    /// Load all rows from this table.
    /// By default, results are ordered in descending order according to their ID column.
    /// This can be configured using `#[ormx(order_by = "some_column ASC")]`.
    fn all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send + 'a {
        Self::stream_all(db).try_collect()
    }

    /// Load at most `limit` rows from this table, skipping the first `offset`.
    /// By default, results are ordered in descending order according to their ID column.
    /// This can be configured using `#[ormx(order_by = "some_column ASC")]`.
    fn all_paginated<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        offset: i64,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send + 'a {
        Self::stream_all_paginated(db, offset, limit).try_collect()
    }

    /// Applies a patch to this row.
    fn patch<'a, 'c: 'a, P>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
        patch: P,
    ) -> impl Future<Output = Result<()>> + Send + 'a
    where
        P: Patch<Table = Self>,
    {
        async move {
            let patch: P = patch;
            patch.patch_row(db, self.id()).send().await?;
            patch.apply_to(self);
            Ok(())
        }
    }

    /// Updates all fields of this row, regardless if they have been changed or not.
    fn update<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Future<Output = Result<()>> + Send + 'a;

    /// Refresh this row, querying all columns from the database.
    fn reload<'a, 'c: 'a>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Future<Output = Result<()>> + Send + 'a {
        async move {
            *self = Self::get(db, self.id()).send().await?;
            Ok(())
        }
    }
}

pub trait Delete
where
    Self: Table + Sized + Send + Sync + 'static,
{
    /// Delete a row from the database
    fn delete_row<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> impl Future<Output = Result<()>> + Send + 'a;

    /// Deletes this row from the database
    fn delete<'a, 'c: 'a>(
        self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Future<Output = Result<()>> + Send + 'a {
        Self::delete_row(db, self.id())
    }

    /// Deletes this row from the database
    fn delete_ref<'a, 'c: 'a>(
        &self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> impl Future<Output = Result<()>> + Send + 'a {
        Self::delete_row(db, self.id())
    }
}

/// A type which can be used to "patch" a row, updating multiple fields at once.
pub trait Patch
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Applies the data of this patch to the given entity.
    /// This does not persist the change in the database.
    fn apply_to(self, entity: &mut Self::Table);

    /// Applies this patch to a row in the database.
    fn patch_row<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
        id: <Self::Table as Table>::Id,
    ) -> impl Future<Output = Result<()>> + Send + 'a;
}

/// A type which can be inserted as a row into the database.
pub trait Insert
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Insert a row into the database, returning the inserted row.
    fn insert<'a, 'c: 'a>(
        self,
        #[cfg(not(feature = "mysql"))] db: impl Executor<'c, Database = Db> + 'a,
        #[cfg(feature = "mysql")] db: &'c mut sqlx::MySqlConnection,
    ) -> impl Future<Output = Result<Self::Table>> + Send + 'a;
}

// Ridiculous workaround for [#100013](https://github.com/rust-lang/rust/issues/100013#issuecomment-2210995259).
trait SendFuture: Future {
    fn send(self) -> impl Future<Output = Self::Output> + Send
    where
        Self: Sized + Send,
    {
        self
    }
}

impl<T: Future> SendFuture for T {}
