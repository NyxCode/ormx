#![cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
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

use futures::{future::BoxFuture, stream::BoxStream};
pub use ormx_macros::*;
use sqlx::{Database, Executor, Result};

#[doc(hidden)]
pub mod exports {
    pub use futures;

    pub use crate::query2::map::*;
}

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
mod query2;

#[cfg(feature = "mysql")]
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
    fn insert(
        db: &mut <Db as Database>::Connection,
        row: impl Insert<Table = Self>,
    ) -> BoxFuture<Result<Self>> {
        row.insert(db)
    }

    /// Queries the row of the given id.
    fn get<'a, 'c: 'a, E>(db: E, id: Self::Id) -> BoxFuture<'a, Result<Self>>
    where
        E: Executor<'c, Database = Db> + 'a;

    /// Stream all rows from this table.
    fn stream_all<'a>(db: &'a sqlx::Pool<Db>) -> BoxStream<'a, Result<Self>>;

    fn stream_all_paginated<'a>(
        db: &'a sqlx::Pool<Db>,
        offset: i64,
        limit: i64,
    ) -> BoxStream<'a, Result<Self>>;

    /// Load all rows from this table.
    fn all<'a>(db: &'a sqlx::Pool<Db>) -> BoxFuture<'a, Result<Vec<Self>>> {
        use futures::TryStreamExt;

        Box::pin(Self::stream_all(db).try_collect())
    }

    fn all_paginated<'a>(
        db: &'a sqlx::Pool<Db>,
        offset: i64,
        limit: i64,
    ) -> BoxFuture<'a, Result<Vec<Self>>> {
        use futures::TryStreamExt;

        Box::pin(Self::stream_all_paginated(db, offset, limit).try_collect())
    }

    /// Applies a patch to this row.
    fn patch<'a, 'c: 'a, P, E>(&'a mut self, db: E, patch: P) -> BoxFuture<'a, Result<()>>
    where
        P: Patch<Table = Self>,
        E: sqlx::Executor<'c, Database = Db> + 'a,
    {
        Box::pin(async move {
            let patch: P = patch;
            patch.patch_row(db, self.id()).await?;
            patch.apply_to(self);
            Ok(())
        })
    }

    /// Updates all fields of this row, regardless if they have been changed or not.
    fn update<'a, 'c: 'a, E>(&'a self, db: E) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a;

    // Refresh this row, querying all columns from the database.
    fn reload<'a, 'c: 'a, E>(&'a mut self, db: E) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a,
    {
        Box::pin(async move {
            *self = Self::get(db, self.id()).await?;
            Ok(())
        })
    }
}

pub trait Delete
where
    Self: Table + Sized + Send + Sync + 'static,
{
    /// Delete a row from the database
    fn delete_row<'a, 'c: 'a, E>(db: E, id: Self::Id) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a;

    /// Deletes this row from the database
    fn delete<'a, 'c: 'a, E>(self, db: E) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a,
    {
        Self::delete_row(db, self.id())
    }

    /// Deletes this row from the database
    fn delete_ref<'a, 'c: 'a, E>(&self, db: E) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a,
    {
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
    fn patch_row<'a, 'c: 'a, E>(
        &'a self,
        db: E,
        id: <Self::Table as Table>::Id,
    ) -> BoxFuture<'a, Result<()>>
    where
        E: sqlx::Executor<'c, Database = Db> + 'a;
}

/// A type which can be inserted as a row into the database.
pub trait Insert
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Insert a row into the database, returning the inserted row.
    fn insert(self, db: &mut <Db as Database>::Connection) -> BoxFuture<Result<Self::Table>>;
}

#[ouroboros::self_referencing]
pub struct SelfRefStream<Args: 'static, Item>
{
    args: Args,
    #[borrows(args)]
    #[covariant] // Box is covariant.
    inner: BoxStream<'this, Result<Item>>,
}

impl<Args: 'static, Item> SelfRefStream<Args, Item>
{
    #[inline]
    pub fn build(
        args: Args,
        inner_builder: impl for<'this> FnOnce(&'this Args) -> BoxStream<'this, Result<Item>>,
    ) -> Self {
        SelfRefStreamBuilder {
            args,
            inner_builder,
        }
        .build()
    }
}

impl<Args: 'static, Item> futures::Stream for SelfRefStream<Args, Item>
{
    type Item = Result<Item>;

    #[inline]
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut futures::task::Context<'_>,
    ) -> futures::task::Poll<Option<Self::Item>> {
        self.with_inner_mut(|s| s.as_mut().poll_next(cx))
    }
}
