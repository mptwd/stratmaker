use chrono::Utc;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    errors::AppError, models::{Strategy, StrategyResumed}, validators::strategy_validator::StrategyContent, Database
};

impl Database {
    pub async fn create_strategy(
        &self,
        user_id: Uuid,
        title: &str,
        content: &StrategyContent,
    ) -> Result<Strategy, AppError> {
        let now = Utc::now();

        let strategy = sqlx::query_as::<_, Strategy>(
            r#"
            INSERT INTO strategies (user_id, title, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(title)
        .bind(Json(content))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(strategy)
    }

    pub async fn delete_strategy(
        &self,
        strat_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM strategies WHERE id = $1 AND user_id = $2")
            .bind(strat_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn modify_strategy(
        &self,
        strat_id: Uuid,
        user_id: Uuid,
        title: &str,
        content: &StrategyContent,
    ) -> Result<u64, AppError> {
        let result = sqlx::query("UPDATE strategies SET title = $1, content = $2, updated_at = NOW() WHERE id = $3 AND user_id = $4")
            .bind(title)
            .bind(Json(content))
            .bind(strat_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_strategy_by_id(&self, strat_id: Uuid, user_id: Uuid) -> Result<Option<Strategy>, AppError> {
        let strat = sqlx::query_as::<_, Strategy>("SELECT * FROM strategies WHERE id = $1 AND user_id = $2")
            .bind(strat_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(strat)
    }

    pub async fn get_strategy_by_title(&self, title: &String, user_id: Uuid) -> Result<Option<Strategy>, AppError> {
        let strat = sqlx::query_as::<_, Strategy>("SELECT * FROM strategies WHERE title = $1 AND user_id = $2")
            .bind(title)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(strat)
    }

    pub async fn get_user_strategies(&self, user_id: Uuid) -> Result<Vec<StrategyResumed>, AppError> {
        let strategies = sqlx::query_as::<_, StrategyResumed>(
            r#"
            SELECT id, title 
            FROM strategies 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(strategies)
    }

}
