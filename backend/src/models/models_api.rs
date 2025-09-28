use serde::{Deserialize, Serialize};

#[derive(Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Strategy {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: serde_json::Value,
}
