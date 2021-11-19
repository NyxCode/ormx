use chrono::Utc;
use ormx::exports::futures::StreamExt;
use ormx::{Db, Delete, Insert, Table};
use sqlx::SqlitePool;

// mod query2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let db = SqlitePool::connect(&dotenv::var("DATABASE_URL")?).await?;
    let mut conn = db.acquire().await?;

    log::info!("insert a new row into the database");
    let mut new = InsertUser {
        id: 1,
        first_name: "Moritz".to_owned(),
        last_name: "Bischof".to_owned(),
        email: "moritz.bischof1@gmail.com".to_owned(),
        disabled: None,
        role: Role::User,
    }
    .insert(&mut conn)
    .await?;

    new.reload(&mut conn).await?;
    log::info!("reloaded user: {:?}", new);
    new.reload(&db).await?;

    log::info!("update a single field");
    new.set_last_login(&db, Utc::now().naive_utc().timestamp())
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

    // log::info!("use the improved query macro for searching users");
    // let search_result = query2::query_users(&db, Some("NewFirstName"), None).await?;
    // println!("{:?}", search_result);
    let mut stream = User::stream_all(&db);
    while let Some(x) = stream.next().await {
        log::info!("{:?}", x);
    }

    new.reload(&db).await?;

    log::info!("update a single field");
    new.set_last_login(&db, Utc::now().naive_utc().timestamp())
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

    log::info!("delete the user from the database");
    new.delete(&db).await?;

    Ok(())
}

#[derive(Debug, ormx::Table)]
#[ormx(table = "users", id = id, insertable, deletable)]
struct User {
    #[ormx(column = "id")]
    #[ormx(get_one = get_by_id)]
    id: i64,
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
    last_login: i64,
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
