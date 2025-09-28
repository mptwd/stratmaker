use axum::{
    response::IntoResponse,
    http::StatusCode};

pub async fn health_check(
) -> impl IntoResponse {
    (StatusCode::OK, "Status is available")
}
