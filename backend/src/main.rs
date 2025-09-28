use axum::{routing::{get, post}, Router};
use tokio::net::TcpListener;

use crate::{auth::SessionStore, db::Database, handlers::*};

mod db;
mod app;
mod models;
//mod routes;
mod handlers;
mod middleware;
//mod store;
mod auth;
mod errors;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    session_store: SessionStore
}


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

    /*
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::COOKIE,
        ]);
    */

    // Build routes
    let app = Router::new()
        //.merge(routes::routes(&app_state))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_current_user))
        .route("/api/protected", get(protected_route))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth_middleware,
        ))
        //.layer(cors)
        //.layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

