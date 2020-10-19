use crate::User;
use sqlx::MySqlPool;

pub(crate) async fn query_users(
    db: &MySqlPool,
    filter: Option<&str>,
    limit: Option<u32>,
) -> anyhow::Result<Vec<User>> {
    let result = ormx::conditional_query_as!(
        User,
        "SELECT id AS user_id, first_name, last_name, email, last_login FROM users"
        Some(f) = filter => {
            "WHERE first_name LIKE" ?(f)
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
