use axum::{extract::State, response::Json};
use rmp_serde::from_slice;
use uuid::Uuid;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use serde_json::{Value, json};

use crate::{
    AppState,
    errors::AppError,
    extractors::AuthenticatedUser,
    models::{CreateStrategyRequest, GetStrategyRequest, Strategy, StrategyResumed},
    validators::strategy_validator::{StrategyContent, validate_strategy_title},
};

pub async fn create_strategy(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<CreateStrategyRequest>,
) -> Result<Json<Strategy>, AppError> {
    // Checking if user already has a strategy named <title>
    if state
        .db
        .get_strategy_by_title(&payload.title, user_id)
        .await?
        .is_some()
    {
        return Err(AppError::StratExists);
    }

    let decoded = match BASE64.decode(&payload.content) {
        Ok(d) => d,
        Err(_) => return Err(AppError::BadRequest("Invalid base64 content".to_string())),
    };

    // Deserialize MessagePack
    let content: StrategyContent = match from_slice(&decoded) {
        Ok(c) => c,
        Err(_) => return Err(AppError::BadRequest("Invalid content".to_string())),
    };

    validate_strategy_title(&payload.title)?;
    state.strat_validator.validate_strategy(&content)?;

    let strategy = state
        .db
        .create_strategy(user_id, &payload.title, &content)
        .await?;

    Ok(Json(strategy))
}

pub async fn delete_strategy(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(strat_id): Json<Uuid>,
) -> Result<Json<Value>, AppError> {
    let res = state.db.delete_strategy(user_id, strat_id).await?;

    Ok(Json(json!({
        "message": "sucessfuly deleted strategy",
        "num_del": res,
    })))
}

pub async fn modify_strategy(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<Strategy>,
) -> Result<Json<Value>, AppError> {
    // Checking if user has the strategy to modify
    let strat = state
        .db
        .get_strategy_by_id(payload.id, user_id)
        .await?
        .ok_or(AppError::StratNotFound)?;

    validate_strategy_title(&payload.title)?;
    state.strat_validator.validate_strategy(&strat.content)?;

    let res = state
        .db
        .modify_strategy(payload.id, user_id, &payload.title, &payload.content)
        .await?;

    Ok(Json(json!({
        "message": "sucessfuly modified strategy",
        "num_del": res,
    })))
}

pub async fn get_strategy(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<GetStrategyRequest>,
) -> Result<Json<Strategy>, AppError> {
    let strat = state
        .db
        .get_strategy_by_id(payload.id, user_id)
        .await?
        .ok_or(AppError::StratNotFound)?;
    Ok(Json(strat))
}

pub async fn get_strategies(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<Vec<StrategyResumed>>, AppError> {
    let strats = state.db.get_user_strategies(user_id).await?;
    Ok(Json(strats))
}
