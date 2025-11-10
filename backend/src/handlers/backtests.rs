use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    extractors::AuthenticatedUser,
    models::{Backtest, BacktestStatus, CreateBacktestRequest},
    validators::strategy_validator::StrategyValidator,
};

pub async fn request_backtest(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<CreateBacktestRequest>,
) -> Result<Json<BacktestStatus>, AppError> {
    let strat = state
        .db
        .get_strategy_by_id(payload.strategy_id, user_id)
        .await?
        .ok_or(AppError::StratNotFound)?;

    // TODO: maybe check if the user already has pending/running strategies and check how many he
    // is allowed to have.

    // Check if dataset with given timeframe exists
    let dataset_meta = state
        .dataset_manager
        .get_dataset(format!("{}-{}", payload.dataset, payload.timeframe))
        .await?;

    if payload.date_start < dataset_meta.start || payload.date_end > dataset_meta.end {
        return Err(AppError::BadRequest(format!(
            "start and end date must be between {} and {}",
            dataset_meta.start, dataset_meta.end
        )));
    }

    let indicators = StrategyValidator::get_indicators(&strat.content);
    if !indicators.iter().all(|i| dataset_meta.ta.contains(i)) {
        return Err(AppError::BadRequest(
            "Using indicators not present in dataset".to_string(),
        ));
    }

    // TODO: Check if users can still run backtest based on subscription

    let backtest = state
        .db
        .create_backtest(
            payload.strategy_id,
            &payload.dataset,
            &payload.timeframe,
            payload.date_start,
            payload.date_end,
        )
        .await?;

    // TODO: Log the dataset and start/end time for metrics

    // Add backtest to job queue or whatever
    let job_id = state.db.enqueue_backtest(&strat.content.0, 1).await?; // TODO: Make the priority system

    // Returning only the backtest's initial status
    Ok(Json(backtest.status))
}

// NOTE: This handler is possibly not needed anymore
pub async fn backtest_status(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(backtest_id): Path<Uuid>,
) -> Result<Json<BacktestStatus>, AppError> {
    let status = state.db.get_backtest_status(backtest_id, user_id).await?;
    Ok(Json(status))
}

pub async fn backtest_results(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(backtest_id): Path<Uuid>,
) -> Result<Json<Backtest>, AppError> {
    let status = state.db.get_backtest_status(backtest_id, user_id).await?;
    match status {
        BacktestStatus::Done => {
            let backtest = state.db.get_backtest_by_id(backtest_id, user_id).await?;
            // TODO: Send the results rather than the backtest.
            Ok(Json(backtest))
        }
        BacktestStatus::Failed | BacktestStatus::Cancelled => {
            let backtest = state.db.get_backtest_by_id(backtest_id, user_id).await?;
            // HACK: Just sending the backtest here cause the failed status is in it
            Ok(Json(backtest))
        }
        BacktestStatus::Pending | BacktestStatus::Running => {
            // TODO: Get the job's status
            // And basically do the same as above again
            let backtest = state.db.get_backtest_by_id(backtest_id, user_id).await?;
            Ok(Json(backtest))
        }
    }
}
