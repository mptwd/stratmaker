pub mod users;
pub mod strategies;
pub mod backtests;

use axum::{
    response::Json,
};
use serde_json::{Value, json};

use crate::{
    errors::AppError, extractors::AuthenticatedUser,
};

pub async fn protected_route(AuthenticatedUser(user_id): AuthenticatedUser) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "message": "This is a protected route",
        "user_id": user_id
    })))
}
