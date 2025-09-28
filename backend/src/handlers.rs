//pub mod users;
//pub mod strategies;

use axum::{
    extract::{Request, State},
    response::Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    auth::{hash_password, verify_password},
    errors::AppError,
    models::{AuthResponse, LoginRequest, RegisterRequest, UserResponse},
    AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Basic validation
    if payload.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    if payload.email.len() > 255 {
        return Err(AppError::BadRequest("Email cannot be greater than 255 characters".to_string()));
    }

    // TODO: Check email through regex.
    
    if payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters long".to_string(),
        ));
    }
    // TODO: better password checking.

    // Check if user already exists
    if let Some(_) = state.db.get_user_by_email(&payload.email).await? {
        return Err(AppError::UserExists);
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user
    let user = state
        .db
        .create_user(&payload.username, &payload.email, &password_hash)
        .await?;

    let response = AuthResponse {
        user: user.into(),
        message: "User registered successfully".to_string(),
    };

    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    // Find user by email
    let user = state
        .db
        .get_user_by_email(&payload.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify password
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    // Create session with Redis
    let session_id = state.session_store.create_session(user.id).await?;

    // Create secure cookie
    let session_cookie = Cookie::build(("session_id", session_id))
        .http_only(true)
        .secure(true) // Use only in HTTPS in production
        .same_site(cookie::SameSite::Lax) // TODO: What is Lax ?
        .path("/")
        .max_age(cookie::time::Duration::days(1));

    let jar = jar.add(session_cookie);

    let response = AuthResponse {
        user: user.into(),
        message: "Login successful".to_string(),
    };

    Ok((jar, Json(response)))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>), AppError> {
    if let Some(session_cookie) = jar.get("session_id") {
        let session_id = session_cookie.value();
        
        // Delete session from Redis
        state.session_store.delete_session(session_id).await?;
    }

    // Remove cookie
    let session_cookie = Cookie::build(("session_id", ""))
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::seconds(0));

    let jar = jar.add(session_cookie);

    Ok((jar, Json(json!({ "message": "Logout successful" }))))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    req: Request,
) -> Result<Json<UserResponse>, AppError> {
    // Get user_id from request extensions (set by middleware)
    let user_id = req
        .extensions()
        .get::<Uuid>()
        .ok_or(AppError::Unauthorized)?;

    let user = state
        .db
        .get_user_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}

pub async fn protected_route(req: Request) -> Result<Json<Value>, AppError> {
    let user_id = req
        .extensions()
        .get::<Uuid>()
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(json!({
        "message": "This is a protected route",
        "user_id": user_id
    })))
}
