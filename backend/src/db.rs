mod user_store;

use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!().run(&self.pool).await?;
        Ok(())
    }
}
