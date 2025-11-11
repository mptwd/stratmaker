use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Type, types::Json};
use uuid::Uuid;

use crate::validators::strategy_validator::StrategyContent;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    // Do we need username here ?
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub message: String,
}

/* Strategy models */

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Strategy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: Json<StrategyContent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StrategyResumed {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub title: String,
    //pub content: StrategyContent,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStrategyRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "backtest_status", rename_all = "lowercase")]
pub enum BacktestStatus {
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Backtest {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub status: BacktestStatus,
    pub dataset: String,
    pub timeframe: String,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    //    pub result_summary: Json(ResultSummary), Do i need that here ?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBacktestRequest {
    pub strategy_id: Uuid,
    pub dataset: String,
    pub timeframe: String,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
}
