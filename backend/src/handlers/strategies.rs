use serde_json::json;
use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CreateStrategy {
    user_id: i64,
    title: String,
    content: serde_json::Value,
}

pub async fn create_strategy(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateStrategy>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO strategies (user_id, title, content) VALUES ($1, $2, $3) RETURNING id",
        payload.user_id,
        payload.title,
        payload.content
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(record) => (
            StatusCode::CREATED,
            Json(json!({ "id": record.id }))
        ),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" }))
            )
        }
    }
}

