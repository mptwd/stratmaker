use axum::{
    extract::State,
    response::Json,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde_json::{Value, json};

use crate::{
    AppState,
    auth::{hash_password, verify_password},
    errors::AppError,
    extractors::AuthenticatedUser,
    models::{
        AuthResponse,
        LoginRequest,
        RegisterRequest,
        UserResponse,
        ChangePasswordRequest
    },
    validators::{
        password_validator::validate_password,
        username_validator::validate_username,
        email_validator::validate_email,
    },
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    validate_username(&payload.username)?;
    validate_email(&payload.email)?;
    validate_password(&payload.password)?;

    // Check if username already taken
    if (state.db.get_user_by_username(&payload.username).await?).is_some() {
        return Err(AppError::UsernameTaken);
    }

    // Check if user already exists
    if (state.db.get_user_by_email(&payload.email).await?).is_some() {
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
        .get_user_by_email(&payload.email) // is this safe ? should i check if email is good first
                                           // to avoid sql injections ?
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify password
    // is this safe ? should i check if email is good first to avoid sql injections ?
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
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = state
        .db
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    Ok(Json(user.into()))
}

pub async fn change_password(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    let user = state
        .db
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    // To change password, user must be authenticated, and give his old password.
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    state.db.update_user_password(user_id, &payload.new_password).await?;
        
    Ok(Json(json!({"message": "Password changed"})))
}
