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
//! See the docs of [derive(Table)](ormx::derive_table) and [derive(Patch)](ormx::derive_patch).

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use sqlx::{Database, Executor, Result};

pub use ormx_macros::*;

#[doc(hidden)]
pub mod exports {
    pub use futures;
}

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
        row: impl Insert<Table=Self>,
    ) -> BoxFuture<Result<Self>> {
        row.insert(db)
    }

    /// Queries the row of the given id.
    fn get<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> BoxFuture<'a, Result<Self>>;

    /// Stream all rows from this table.
    fn stream_all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxStream<'a, Result<Self>>;

    /// Load all rows from this table.
    fn all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxFuture<'a, Result<Vec<Self>>> {
        use futures::TryStreamExt;

        Box::pin(async move { Self::stream_all(db).try_collect().await })
    }

    /// Applies a patch to this row.
    fn patch<'a, 'c: 'a, P>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
        patch: P,
    ) -> BoxFuture<'a, Result<()>>
    where
        P: Patch<Table = Self>,
    {
        Box::pin(async move {
            let patch: P = patch;
            patch.patch_row(db, self.id()).await?;
            patch.apply_to(self);
            Ok(())
        })
    }

    /// Updates all fields of this row, regardless if they have been changed or not.
    fn update<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxFuture<'a, Result<()>>;

    // Refresh this row, querying all columns from the database.
    fn reload<'a, 'c: 'a>(
        &'a mut self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            *self = Self::get(db, self.id()).await?;
            Ok(())
        })
    }

    /// Deletes this row from the database.
    fn delete<'a, 'c: 'a>(
        self,
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxFuture<'a, Result<()>>;
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
    ) -> BoxFuture<'a, Result<()>>;
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
