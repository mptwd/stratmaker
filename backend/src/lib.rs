pub mod app;
pub mod db;
pub mod models;
//mod routes;
pub mod handlers;
pub mod middleware;
//mod store;
pub mod auth;
pub mod dataset_client;
pub mod errors;
pub mod extractors;
pub mod s3_manager;
pub mod validators;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::protected_route;
use crate::handlers::strategies::*;
use crate::handlers::{backtests::request_backtest, users::*}; // TODO: delete

// Making those public because they are needed for integration testing.
pub use crate::app::AppState;
pub use crate::auth::SessionStore;
pub use crate::db::Database;

pub fn create_app(app_state: AppState) -> Router {
    // TODO: Check what CORS do
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(["http://localhost:3000".parse::<HeaderValue>().unwrap()])
        .allow_credentials(true)
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
        ]);

    Router::new()
        //.merge(routes::routes(&app_state))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_current_user))
        .route("/api/protected", get(protected_route)) // TODO: delete this, it was for testing
        .route("/api/password", post(change_password))
        .route("/api/strategy/create", post(create_strategy))
        .route("/api/strategy/delete", post(delete_strategy))
        .route("/api/strategy/modify", post(modify_strategy))
        // HACK: Changed it to a post because frontend didn't like get with body (here the strat id)
        .route("/api/strategy", post(get_strategy))
        .route("/api/strategy/all", get(get_strategies))
        .route("/api/backtest", post(request_backtest))
        //.route("/api/backtest/:id", get(backtest_status))
        //.route("/api/backtest/:id/results", get(backtest_results))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth_middleware,
        ))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .layer(cors)
        //.layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
