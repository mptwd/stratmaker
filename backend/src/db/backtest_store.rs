use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{Backtest, BacktestStatus},
    Database
};


impl Database {
    pub async fn create_backtest(
        &self,
        strategy_id: Uuid,
        dataset: &str,
        timeframe: &str,
        date_start: DateTime<Utc>, 
        date_end: DateTime<Utc>,
    ) -> Result<Backtest, AppError> {
        let now = Utc::now();

        let backtest = sqlx::query_as::<_, Backtest>(
            r#"
            INSERT INTO backtests (strategy_id, dataset, timeframe, date_start, date_end, created_at, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            RETURNING id, strategy_id, status, dataset, timeframe, date_start, date_end, created_at
            "#,
        )
        .bind(strategy_id)
        .bind(dataset)
        .bind(timeframe)
        .bind(date_start)
        .bind(date_end)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(backtest)
    }

    pub async fn get_backtest_by_id(
        &self,
        backtest_id: Uuid,
        user_id: Uuid,
    ) -> Result<Backtest, AppError> {
        let backtest = sqlx::query_as::<_, Backtest>(
            r#"
            SELECT * FROM backtests
            JOIN strategies ON backtests.strategy_id = strategies.id
            WHERE id = $1 AND strategies.user_id = $2
            "#,
        )
        .bind(backtest_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(backtest)
    }

    pub async fn get_backtest_status(
        &self,
        backtest_id: Uuid,
        user_id: Uuid,
    ) -> Result<BacktestStatus, AppError> {
        let status = sqlx::query_scalar::<_, BacktestStatus>(
            r#"
            SELECT status FROM backtests
            JOIN strategies ON backtests.strategy_id = strategies.id
            WHERE id = $1 AND strategies.user_id = $2
            "#,
        )
        .bind(backtest_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(status)
    }
}
