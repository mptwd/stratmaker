use uuid::Uuid;
use axum::{
    extract::State,
    response::Json,
};

use serde_json::{Value, json};

use crate::{
    errors::AppError, extractors::AuthenticatedUser, models::{
        CreateStrategyRequest, GetStrategyRequest, Strategy, StrategyResumed
    }, validators::validate_strategy, AppState
};

pub async fn create_strategy(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<CreateStrategyRequest>,
) -> Result<Json<Strategy>, AppError> {
    // Checking if user already has a strategy named <title>
    if state.db.get_strategy_by_title(&payload.title, user_id).await?.is_some() {
        return Err(AppError::StratExists);
    }

    validate_strategy(&payload.content)?;

    let strategy = state.db.create_strategy(user_id, &payload.title, &payload.content).await?;
        
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
    let strat = state.db.get_strategy_by_id(payload.id, user_id).await?.ok_or(AppError::NotFound)?;
    // Is this usefull ?
    if strat.user_id != user_id {
        return Err(AppError::NotFound);
    }

    validate_strategy(&payload.content)?;

    let res = state.db.modify_strategy(payload.id, user_id, &payload.title, &payload.content).await?;

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
    let strat = state.db.get_strategy_by_id(payload.id, user_id).await?.ok_or(AppError::NotFound)?;

    Ok(Json(strat))
}

pub async fn get_strategies(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<Vec<StrategyResumed>>, AppError> {
    let strats = state.db.get_user_strategies(user_id).await?;
    Ok(Json(strats))
}

