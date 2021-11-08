use sqlx::SqlitePool;

use crate::User;

pub(crate) async fn query_users(
    db: &SqlitePool,
    filter: Option<&str>,
    limit: Option<i64>,
) -> anyhow::Result<Vec<User>> {
    let result = ormx::conditional_query_as!(
        User,
        r#"SELECT id AS user_id, first_name, last_name, email, disabled, role AS "role: _", last_login"#
        "FROM users"
        Some(f) = filter => {
            "WHERE first_name LIKE" ?(f)
            "OR last_name LIKE" ?(f)
        }
        "ORDER BY first_name DESC"
        Some(l) = limit => {
            "LIMIT" ?(l)
        }
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}
