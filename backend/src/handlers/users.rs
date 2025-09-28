use std::sync::Arc;

use base64::Engine;
use cookie::{time, Cookie};
use redis::AsyncCommands;
use serde_json::json;
use axum::{
    extract::{Path, State}, http::StatusCode, response::IntoResponse, Extension, Json
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rand::RngCore;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    models::{models_api::User, models_db::UserDB},
    store::user_store::create_user
};


impl AppState {
    pub async fn redis_conn(&self) -> redis::aio::Connection {
        self.redis.get_async_connection().await.expect("redis conn")
    }
}



/* ======> Register User <====== */

pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_register_request(&payload) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": e })));
    }

    let password_hash = format!("hash({})", payload.password); // TODO: Actually hash the password

    let user = UserDB{
        id: None, // This is clearly not clean code
        email: payload.email,
        username: payload.username,
        password_hash: password_hash,
    };

    let store_result = create_user(pool, user).await;

    match store_result {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))),
        Err(e) => {
            eprintln!("DB error create_user: {:?}", e); // TODO Better logging
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "internal server error" })))
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterUserRequest {
    username: String,
    email: String,
    password: String,
}

fn validate_register_request(req: &RegisterUserRequest) -> Result<(), String> {
    if req.email == "" {
        return Err("email is required".to_string());
    }

    if req.email.len() > 255 {
        return Err("email cannot be greater than 255 characters".to_string());
    }

    // TODO: regex the email
    
    // TODO: check the username (maybe for bad words or something)
    if req.username == "" {
        return Err("username is required".to_string());
    }
    if req.email.len() > 255 {
        return Err("username cannot be greater than 255 characters".to_string());
    }
    
    if req.password == "" {
        return Err("password is required".to_string());
    }

    Ok(())
}


/* ============================= */


/* ======> LOGIN <====== */

pub async fn login(
        Extension(state): Extension<Arc<AppState>>,
        Json(payload): Json<LoginRequest>,
    ) -> impl IntoResponse {
    let pool = &state.pg_pool;
    
    if let Ok(Some(user)) = check_credentials(pool.clone(), &payload.email, &payload.password).await { // Clone here bothers me
        // Credentials valid -> create session in Rediedy-tmux
        let session_id = generate_session_id();
        let ttl_seconds = 60 * 60 * 24 * 7; // 7 days

        let mut con = state.redis_conn().await;
        let key = format!("session:{}", session_id);
        let _: () = con.set_ex(key, user.id, ttl_seconds).await.unwrap();

        // Send session ID as HttpOnly, Secure cookie
        let cookie = Cookie::build("session_id", session_id)
            .http_only(true)
            .secure(true) // requires HTTPS
            .path("/")
            .max_age(time::Duration::seconds(ttl_seconds as i64))
            .finish();

        (
            StatusCode::OK,
            [(axum::http::header::SET_COOKIE, cookie.to_string())],
            Json(json!({
                "message": format!("Welcom, {}", user.username),
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid credentials" }))
        )
            .into_response()
    }
}

async fn check_credentials(
    pool: PgPool,
    email: &str,
    password: &str,
) -> Result<Option<User>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT id, email, username, password_hash FROM users WHERE email = $1",
        email,
        )
        .fetch_optional(&pool)
        .await?;

    let Some(user) = result else {
        // No such user
        return Ok(None);
    };

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| sqlx::Error::RowNotFound)?; // Treat as invalid credentials

    let verified = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    if verified {
        Ok(Some(User {
            id: user.id,
            email: user.email,
            username: user.username,
        }))
    } else {
        Ok(None)
    }
}


fn generate_session_id() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

/* ===================== */

/* ======> LOGOUT <====== */

async fn logout(
    Extension(state): Extension<Arc<AppState>>,
    TypedHeader(cookies): TypedHeader<HeaderCookie>,
) -> impl IntoResponse {
    if let Some(session_id) = cookies.get("session_id") {
        let mut con = state.redis_conn().await;
        let key = format!("session:{}", session_id);
        let _: () = con.del(key).await.unwrap_or(());
    }

    // Clear cookie
    let cookie = Cookie::build("session_id", "")
        .http_only(true)
        .secure(true)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .finish();

    (
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        "Logged out",
    )
}

/* ====================== */


/*
pub async fn get_user_by_id(
    Path(id): Path<i64>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT username, email FROM users WHERE id = $1",
        id,
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => (StatusCode::OK, Json(json!({
            "username": user.username,
            "email": user.email,
        }))),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "user not found" })))
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "internal server error" })))
        }
    }
}
*/
