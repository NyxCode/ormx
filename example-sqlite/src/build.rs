use anyhow::Context;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=example-sqlite/migrations/20211108025529_create.sql");
    dotenv::dotenv().ok();
    let url = env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

    let mut conn = SqliteConnectOptions::from_str(&url)?
        .create_if_missing(true)
        .connect()
        .await
        .context(format!("unable to open database connection: {}", url))?;

    sqlx::migrate!("./migrations/")
        .run(&mut conn)
        .await
        .context("unable to run database migration")?;

    Ok(())
}
