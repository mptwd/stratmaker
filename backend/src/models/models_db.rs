use serde::Serialize;

#[derive(sqlx::FromRow)]
pub struct UserDB {
    pub id: Option<i64>,
    pub username: String,
    pub password_hash: String, // TODO: Type is probably not correct
    pub email: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct StrategyDB {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: serde_json::Value,
}
