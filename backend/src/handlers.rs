pub mod users;
//pub mod strategies;

use axum::{
    extract::Request,
    response::Json,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    errors::AppError,
};

pub async fn protected_route(req: Request) -> Result<Json<Value>, AppError> {
    let user_id = req
        .extensions()
        .get::<Uuid>()
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(json!({
        "message": "This is a protected route",
        "user_id": user_id
    })))
}
