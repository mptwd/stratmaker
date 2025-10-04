use axum::{
    extract::{Request, State},
    response::Json,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use regex::Regex;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    AppState,
    auth::{hash_password, verify_password},
    errors::AppError,
    models::{
        AuthResponse,
        LoginRequest,
        RegisterRequest,
        UserResponse,
        ChangePasswordRequest
    },
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    /* ===> Username validation <=== */

    if payload.username.is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }

    if payload.username.len() < 3 {
        return Err(AppError::BadRequest(
            "Username cannot be smaller than 3 characters".to_string(),
        ));
    }

    if payload.username.len() > 25 {
        return Err(AppError::BadRequest(
            "Username cannot be greater than 25 characters".to_string(),
        ));
    }


    /*
     * At least 1 letter.
     * Starts with a letter or a digit.
     * 3-25 characters.
     * Only letters, digits, hyphen and underscores are allowed.
     * Hyphen and underscores have to be followed by a letter or a digit.
     */
    let chars: Vec<char> = payload.username.chars().collect();

    // Must start with letter or digit
    if !chars[0].is_ascii_alphanumeric() {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // At least 1 letter
    if !chars.iter().any(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Allowed characters only
    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
    {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Hyphen/underscore must be followed by a letter/digit
    for w in chars.windows(2) {
        if (w[0] == '-' || w[0] == '_') && !w[1].is_ascii_alphanumeric() {
            return Err(AppError::BadRequest(
                "Must be a valid username".to_string(),
            ));
        }
    }

    if chars.last() == Some(&'-') || chars.last() == Some(&'_') {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    /* ============================= */

    /* ===> Email validation <=== */

    if payload.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    if payload.email.len() > 255 {
        return Err(AppError::BadRequest(
            "Email cannot be greater than 255 characters".to_string(),
        ));
    }

    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(&payload.email) {
        return Err(AppError::BadRequest("Must be a valid email".to_string()));
    }

    /* ========================== */


    /* ===> Password validation <=== */

    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    if payload.password.len() < 12 {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters long".to_string(),
        ));
    }

    if payload.password.len() > 128 {
        return Err(AppError::BadRequest(
            "Password cannot be longer than 128 characters".to_string(),
        ));
    }

    let has_lower   = payload.password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper   = payload.password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit   = payload.password.chars().any(|c| c.is_ascii_digit());
    let has_special = payload.password.chars().any(|c| r"!@#$%^&*()_-+=[]{};:,.<>?/~\|".contains(c));

    let categories = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();

    if categories < 3 {
        return Err(AppError::BadRequest(
                "Password must contain at least 3 of these 4 : lowercase, uppercase, digit, special character".to_string()));
    }

    if payload.password.chars()
        .collect::<Vec<_>>()
        .windows(4)
        .any(|w| w.iter().all(|&c| c == w[0]))
    {
        return Err(AppError::BadRequest(
                "Password cannot contain 4 identical characters in a row".to_string()));
    }

    if payload.password.trim() != payload.password {
        return Err(AppError::BadRequest(
                "Password cannot start or end with whitespace".to_string()));
    }

    // TODO: blacklist common passwords

    /* ============================= */


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

pub async fn change_password(
    State(state): State<AppState>,
    req: Request,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    let user_id = req
        .extensions()
        .get::<Uuid>()
        .ok_or(AppError::Unauthorized)?;

    let user = state
        .db
        .get_user_by_id(*user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // To change password, user must be authenticated, and give his old password.
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    state.db.update_user_password(*user_id, &payload.new_password).await?;
        
    Ok(Json(json!({"message": "Password changed"})))
}
