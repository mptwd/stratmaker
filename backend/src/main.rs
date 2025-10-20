use std::collections::HashSet;

use tokio::net::TcpListener;

use backend::{
    Database,
    SessionStore,
    dataset_client::DatasetManagerClient,
    validators::strategy_validator::StrategyValidator,
    AppState,
    create_app,
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
    valid_indicators.insert("sma_10".to_string());
    valid_indicators.insert("sma_50".to_string());
    valid_indicators.insert("rsi".to_string());
    valid_indicators.insert("macd".to_string());
    valid_indicators.insert("volume".to_string());
    let strat_validator = StrategyValidator::new(valid_indicators);

    let app_state = AppState { db, session_store, dataset_manager, strat_validator };
    let app = create_app(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
