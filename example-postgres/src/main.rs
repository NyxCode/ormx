use chrono::{NaiveDateTime, Utc};
use log::LevelFilter;
use ormx::{Delete, Insert, Table};
use sqlx::PgPool;

// To run this example, first run `/scripts/postgres.sh` to start postgres in a docker container and
// write the database URL to `.env`. Then, source `.env` (`. .env`) and run `cargo run`

mod query2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let db = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    log::info!("insert a few new rows into the database");
    let mut new = InsertUser {
        user_id: 1,
        first_name: "Moritz".to_owned(),
        last_name: "Bischof".to_owned(),
        email: "moritz.bischof1@gmail.com".to_owned(),
        disabled: None,
        role: Role::User,
        ty: Some(AccountType::Normal),
    }
    .insert(&db)
    .await?;
    InsertUser {
        user_id: 2,
        first_name: "Dylan".to_owned(),
        last_name: "Thomas".to_owned(),
        email: "dylan.thomas@gmail.com".to_owned(),
        disabled: Some("email not verified".to_owned()),
        role: Role::Admin,
        ty: None,
    }
    .insert(&db)
    .await?;

    log::info!("update a single field");
    new.set_last_login(&db, Some(Utc::now().naive_utc()))
        .await?;

    log::info!("update all fields at once");
    new.email = "asdf".to_owned();
    new.update(&db).await?;

    log::info!("apply a patch to the user");
    new.patch(
        &db,
        UpdateUser {
            first_name: "NewFirstName".to_owned(),
            last_name: "NewLastName".to_owned(),
            disabled: Some("Reason".to_owned()),
            role: Role::Admin,
        },
    )
    .await?;

    log::info!("reload the user, in case it has been modified");
    new.reload(&db).await?;

    log::info!("use the improved query macro for searching users");
    let search_result = query2::query_users(&db, Some("NewFirstName"), None).await?;
    log::info!("search result: {:?}", search_result);

    log::info!("load all users in the order specified by the 'order_by' attribute");
    let all = User::all_paginated(&db, 0, 100).await?;
    log::info!("all users: {all:?}");

    log::info!("delete the user from the database");
    new.delete(&db).await?;

    Ok(())
}

#[derive(Debug, ormx::Table)]
#[ormx(table = "users", id = user_id, insertable, deletable, order_by = "email ASC")]
struct User {
    // map this field to the column "id"
    #[ormx(column = "id")]
    #[ormx(get_one = get_by_user_id)]
    user_id: i32,
    first_name: String,
    last_name: String,
    // generate `User::by_email(&str) -> Result<Option<Self>>`
    #[ormx(get_optional(&str))]
    email: String,
    #[ormx(custom_type)]
    role: Role,
    #[ormx(column = "type", custom_type)]
    ty: Option<AccountType>,
    #[ormx(custom_type, default, set)]
    group: UserGroup,
    disabled: Option<String>,
    // don't include this field into `InsertUser` since it has a default value
    // generate `User::set_last_login(Option<NaiveDateTime>) -> Result<()>`
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

#[derive(Debug, ormx::Table)]
#[ormx(table = "test", id = id, insertable)]
struct Test {
    id: i32,
    #[ormx(by_ref)]
    rows: Vec<String>,
}
