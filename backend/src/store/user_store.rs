use sqlx::PgPool;

use crate::models::{models_api::User, models_db::UserDB};

pub async fn create_user(pool: PgPool, user: UserDB) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash, email) VALUES ($1, $2, $3) RETURNING id",
        user.username,
        user.password_hash,
        user.email
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(id) => return Ok(id.id),
        Err(e) => return Err(e),
    }
}

pub async fn get_user_by_id(
    id: i64,
    pool: PgPool,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT username, email FROM users WHERE id = $1",
        id,
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(User{
            id: id,
            username: user.username,
            email: user.email,
            }),
        Err(e) => Err(e)
    }
}
