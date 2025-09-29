pub mod db;
pub mod app;
pub mod models;
//mod routes;
pub mod handlers;
pub mod middleware;
//mod store;
pub mod auth;
pub mod errors;

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::*;
pub use crate::db::Database;
pub use crate::auth::SessionStore;
pub use crate::app::AppState;

pub fn create_app(app_state: AppState) -> Router {
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

    Router::new()
        //.merge(routes::routes(&app_state))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_current_user))
        .route("/api/protected", get(protected_route))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth_middleware,
        ))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        //.layer(cors)
        //.layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
