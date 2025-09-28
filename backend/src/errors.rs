use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migrate error: {0}")]
    Migrate(#[from] MigrateError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Password hashing error")]
    PasswordHash,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("User not found")]
    NotFound,

    #[error("User already exists")]
    UserExists,

    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);

                // Handle unique constraint violations
                if let Some(db_err) = e.as_database_error() {
                    if db_err.constraint() == Some("users_email_key") {
                        return (
                            StatusCode::CONFLICT,
                            Json(json!({
                                "error": "User with this email already exists"
                            }))
                        ).into_response();
                    }
                }

                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Migrate(ref e) => {
                tracing::error!("Migrate error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Redis(ref e) => {
                tracing::error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::PasswordHash => {
                tracing::error!("Password hashing error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Json(ref e) => {
                tracing::error!("JSON error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid JSON".to_string())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, "User not found".to_string())
            }
            AppError::UserExists => {
                (StatusCode::CONFLICT, "User already exists".to_string())
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Internal => {
                tracing::error!("Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
