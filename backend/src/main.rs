use tokio::net::TcpListener;

use backend::SessionStore;
use backend::Database;

use backend::{AppState, create_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    // Database connection.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/myapp".to_string());
    let db = Database::new(&database_url).await?;
    db.migrate().await?;


    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let session_store = SessionStore::new(&redis_url).await?;

    let app_state = AppState { db, session_store };
    let app = create_app(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

