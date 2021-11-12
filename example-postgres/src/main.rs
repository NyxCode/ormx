// #![feature(trace_macros)]
use chrono::{NaiveDateTime, Utc};
use ormx::{Insert, Table, Delete};
use sqlx::{PgConnection, PgPool};

// trace_macros!(true);

// To run this example-postgres, first run `/scripts/postgres.sh` to start postgres in a docker container and
// write the database URL to `.env`. Then, source `.env` (`. .env`) and run `cargo run`

mod query2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let db = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;
    let mut conn = db.acquire().await?;

    log::info!("insert a new row into the database");
    let mut new = InsertUser {
        user_id: 1,
        first_name: "Moritz".to_owned(),
        last_name: "Bischof".to_owned(),
        email: "moritz.bischof1@gmail.com".to_owned(),
        disabled: None,
        role: Role::User,
    }
    .insert(&mut conn)
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
    println!("{:?}", search_result);

    log::info!("delete the user from the database");
    new.delete(&db).await?;

    log::info!("inserting 3 dummy users with ids: 1, 2 & 3");
    insert_dummy_user(&mut conn, 2).await?;
    insert_dummy_user(&mut conn, 3).await?;
    insert_dummy_user(&mut conn, 4).await?;

    log::info!("getting many users by any user id (using 'get_any' getter)");
    let users = User::get_by_any_user_id(&mut conn, &[2, 4]).await?;
    dbg!(&users);
    assert_eq!(users.len(), 2);

    log::info!("empty user table");
    sqlx::query!("DELETE FROM users").execute(&db).await?;
    Ok(())
}

async fn insert_dummy_user(conn: &mut PgConnection, id: i32) -> Result<User, sqlx::Error> {
    InsertUser {
        user_id: id,
        first_name: "Dummy".to_owned(),
        last_name: "Dummy".to_owned(),
        email: format!("dummy{}@mail.com", id),
        disabled: None,
        role: Role::User,
    }
    .insert(conn)
    .await
}

#[derive(Debug, ormx::Table)]
#[ormx(table = "users", id = user_id, insertable, deletable)]
struct User {
    // map this field to the column "id"
    #[ormx(column = "id")]
    #[ormx(get_one = get_by_user_id)]
    #[ormx(get_by_any)]
    user_id: i32,
    first_name: String,
    last_name: String,
    // generate `User::by_email(&str) -> Result<Option<Self>>`
    #[ormx(get_optional(&str))]
    email: String,
    #[ormx(custom_type)]
    role: Role,
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

#[derive(Debug, ormx::Table)]
#[ormx(table = "test", id = id, insertable)]
struct Test {
    id: i32,
    #[ormx(by_ref)]
    rows: Vec<String>,
}
