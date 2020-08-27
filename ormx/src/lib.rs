#![cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]

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

pub trait Table
where
    Self: Sized + Send + Sync + 'static,
{
    type Id: 'static + Copy + Send;

    /// Returns the id of this row.
    fn id(&self) -> Self::Id;

    /// Queries the row of the given id.
    fn get<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
        id: Self::Id,
    ) -> BoxFuture<'a, Result<Self>>;

    fn stream_all<'a, 'c: 'a>(
        db: impl Executor<'c, Database = Db> + 'a,
    ) -> BoxStream<'a, Result<Self>>;

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

pub trait Patch
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    /// Applies the data of this patch to the given entity.
    fn apply_to(self, entity: &mut Self::Table);

    /// Executes this patch against the row with the given id.
    fn patch_row<'a, 'c: 'a>(
        &'a self,
        db: impl Executor<'c, Database = Db> + 'a,
        id: <Self::Table as Table>::Id,
    ) -> BoxFuture<'a, Result<()>>;
}

pub trait Insert
where
    Self: Sized + Send + Sync + 'static,
{
    type Table: Table;

    fn insert(self, db: &mut <Db as Database>::Connection) -> BoxFuture<Result<Self::Table>>;
}
