use std::collections::HashSet;

use tokio::net::TcpListener;

use backend::{
    AppState, Database, SessionStore, create_app, dataset_client::DatasetManagerClient, s3_manager,
    validators::strategy_validator::StrategyValidator,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    // Database connection.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/backend".to_string());
    let db = Database::new(&database_url).await?;
    db.migrate().await?;

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let session_store = SessionStore::new(&redis_url).await?;

    let dataset_manager = DatasetManagerClient::new("http://localhost:8081");

    let mut valid_indicators = HashSet::new();
    // HACK: Hard coding them, but should probably use dataset_manager
    valid_indicators.insert("sma_10".to_string());
    valid_indicators.insert("sma_50".to_string());
    valid_indicators.insert("rsi".to_string());
    valid_indicators.insert("macd".to_string());
    valid_indicators.insert("volume".to_string());
    let strat_validator = StrategyValidator::new(valid_indicators);

    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or_else(|_| "datasets".to_string());
    let account_id = std::env::var("R2_ACCOUNT_ID")?;
    let access_key_id = std::env::var("R2_ACCESS_KEY_ID")?;
    let access_key_secret = std::env::var("R2_ACCESS_KEY_SECRET")?;

    let s3 =
        s3_manager::S3Manager::new(bucket_name, account_id, access_key_id, access_key_secret).await;

    let app_state = AppState {
        db,
        session_store,
        dataset_manager,
        strat_validator,
        s3,
    };
    let app = create_app(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
