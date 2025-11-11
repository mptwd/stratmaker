use crate::{Database, models::BacktestStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{errors::AppError, validators::strategy_validator::StrategyContent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    ProcessBacktest,
}

impl JobType {
    fn as_str(&self) -> &str {
        match self {
            JobType::ProcessBacktest => "process_backtest",
        }
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: i64,
    pub job_type: String,
    pub payload: Vec<u8>,
    pub retry_count: i32,
    pub timeout_seconds: i32,
}

impl Database {
    pub async fn enqueue<T: Serialize>(
        &self,
        job_type: JobType,
        payload: &T,
        priority: i32,
        max_retries: i32,
        delay_seconds: i32,
        timeout_seconds: i32,
    ) -> Result<i64, AppError> {
        let payload_bytes = rmp_serde::to_vec(payload)?;

        let job_id: (i64,) = sqlx::query_as(
            r#"
        SELECT enqueue_job($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(job_type.as_str())
        .bind(&payload_bytes)
        .bind(priority)
        .bind(max_retries)
        .bind(delay_seconds)
        .bind(timeout_seconds)
        .fetch_one(&self.pool)
        .await?;

        Ok(job_id.0)
    }

    pub async fn enqueue_backtest(
        &self,
        strategy: &StrategyContent,
        priority: i32,
    ) -> Result<i64, AppError> {
        let mut payload = HashMap::new();

        let strategy_json = serde_json::to_value(strategy)?;
        payload.insert("strategy", strategy_json.to_string());

        // WARN: Hard coded some values, but it's probably not the right aproach
        self.enqueue(JobType::ProcessBacktest, &payload, priority, 3, 0, 600)
            .await
    }

    pub async fn get_job_status(&self, job_id: i64) -> Result<BacktestStatus, AppError> {
        let status = sqlx::query(
            r#"
            SELECT status FROM jobs WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await;

        match status {
            Ok(s) => {}
            Err(sqlx::Error::RowNotFound) => {
                // Job was done to long ago...
            }
            Err(e) => {}
        }

        Ok(BacktestStatus::Pending)
    }
}
