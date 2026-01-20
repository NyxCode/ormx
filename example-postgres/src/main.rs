use chrono::{NaiveDateTime, Utc};
use futures_util::TryStreamExt;
use log::{info, LevelFilter};
use ormx::{Delete, Insert, Table};
use sqlx::PgPool;

mod query2;

#[derive(Debug, ormx::Table)]
#[ormx(table = "users", id = user_id, insertable, deletable, order_by = "email ASC")]
struct User {
    // `#[ormx(default)]` indicates that the database generates a value for us.
    // `#[ormx(get_one = ..)]` generates `User::get_by_user_id(db, id: i32) -> Result<User>` for us
    #[ormx(column = "id", default, get_one = get_by_user_id)] // map this field to the column "id"
    user_id: i32,

    // just some normal, 'NOT NULL' columns
    first_name: String,
    last_name: String,
    disabled: Option<String>,

    // generates `User::by_email(&str) -> Result<Option<Self>>`
    // unlike `#[ormx(get_one = .. )]`, `by_email` will return `None` instead of an error if no record is found.
    #[ormx(get_optional(&str))]
    email: String,

    // custom types need to be annotated with `#[ormx(custom_type)]`
    #[ormx(custom_type)]
    role: Role,

    // they can, of course, also be nullable
    #[ormx(column = "type", custom_type)]
    ty: Option<AccountType>,

    // the database can also provide a default value for them.
    // `#[ormx(set)]` generates `User::set_group(&mut self, g: UserGroup) -> Result` for us
    #[ormx(custom_type, default, set)]
    group: UserGroup,

    // besides enums, composite/record types are also supported
    #[ormx(custom_type, default)]
    favourite_color: Option<Color>,

    // generates `User::set_last_login(&mut self, Option<NaiveDateTime>) -> Result`
    #[ormx(default, set)]
    last_login: Option<NaiveDateTime>,
}

// Patches can be used to update multiple fields at once (in diesel, they're called "ChangeSets").
#[derive(ormx::Patch)]
#[ormx(table_name = "users", table = crate::User, id = "id")]
struct UpdateUser {
    first_name: String,
    last_name: String,
    disabled: Option<String>,
    #[ormx(custom_type)]
    role: Role,
}

// these are all enums, created using `CREATE TYPE .. AS ENUM (..);`

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
enum Role {
    User,
    Admin,
}

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "account_type")]
#[sqlx(rename_all = "lowercase")]
enum AccountType {
    Legacy,
    Normal,
}

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "user_group")]
#[sqlx(rename_all = "lowercase")]
enum UserGroup {
    Local,
    Global,
    Other,
}

// PostgreSQL also supports composite/record types

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "color")]
struct Color {
    red: i32,
    green: i32,
    blue: i32,
}

#[derive(Debug, ormx::Table)]
#[ormx(table = "test", id = id, insertable)]
struct Test {
    id: i32,
    #[ormx(by_ref)]
    rows: Vec<String>,
}

// Example using a schema-qualified table name
#[derive(Debug, ormx::Table)]
#[ormx(table = "app.products", id = id, insertable, deletable)]
struct Product {
    #[ormx(default)]
    id: i32,
    #[ormx(by_ref)]
    name: String,
    price: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let pool = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;
    let mut tx = pool.begin().await?;


    info!("insert a new row into the database..");
    let mut new = InsertUser {
        first_name: "Moritz".to_owned(),
        last_name: "Bischof".to_owned(),
        email: "moritz.bischof1@gmail.com".to_owned(),
        disabled: None,
        role: Role::User,
        ty: Some(AccountType::Normal),
    }
    .insert(&mut *tx)
    .await?;
    info!("after inserting a row, ormx loads the database-generated columns for us, including the ID ({})", new.user_id);


    info!("update a single field at a time, each in its own query..");
    new.set_last_login(&mut *tx, Some(Utc::now().naive_utc()))
        .await?;
    new.set_group(&mut *tx, UserGroup::Global).await?;


    info!("update all fields at once..");
    new.email = "asdf".to_owned();
    new.favourite_color = Some(Color {
        red: 255,
        green: 0,
        blue: 0,
    });
    new.update(&mut *tx).await?;


    info!("apply a patch to the user..");
    new.patch(
        &mut *tx,
        UpdateUser {
            first_name: "NewFirstName".to_owned(),
            last_name: "NewLastName".to_owned(),
            disabled: Some("Reason".to_owned()),
            role: Role::Admin,
        },
    )
    .await?;


    info!("reload the user, in case it has been modified..");
    new.email.clear();
    new.reload(&mut *tx).await?;


    info!("use the improved query macro for searching users..");
    let search_result = query2::query_users(&mut *tx, Some("NewFirstName"), None).await?;
    info!("found {} matching users", search_result.len());


    info!("load all users in the order specified by the 'order_by' attribute..");
    User::stream_all_paginated(&mut *tx, 0, 100)
        .try_for_each(|u| async move {
            info!("- user_id = {}", u.user_id);
            Ok(())
        })
        .await?;


    info!("delete the user from the database..");
    new.delete(&mut *tx).await?;


    info!("test schema-qualified table name (app.products)..");
    let product = InsertProduct {
        name: "Widget".to_owned(),
        price: 19.99,
    }
    .insert(&mut *tx)
    .await?;
    info!("inserted product with id {}", product.id);

    let fetched = Product::get(&mut *tx, product.id).await?;
    info!("fetched product: {:?}", fetched);

    product.delete(&mut *tx).await?;
    info!("deleted product");


    Ok(())
}
