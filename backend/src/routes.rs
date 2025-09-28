use axum::{
    middleware, routing::{
        get, post
    }, Router};
use sqlx::PgPool;

use crate::{app::health_check, middlewares, AppState};
use crate::handlers::{
    users::{
        register_user,
//        get_user_by_id,
    },
    strategies::create_strategy,
};

pub fn routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", post(register_user))
 //       .route("/users/:id", get(get_user_by_id))
        .route("/strategies", post(create_strategy))
        .layer(middleware::from_fn_with_state(app_state, middlewares::auth_middleware))
}
