use axum_test::TestServer;
use backend::models::{AuthResponse, LoginRequest, RegisterRequest};
use cookie::Cookie;
use serde_json::Value;

use crate::helper::{
    TestContext, TestUser,
    assertions::{
        assert_cookie_present, assert_status_code, assert_success_response, extract_cookie_value,
    },
};

#[tokio::test]
async fn test_user_login_success() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // Register user first
    let _register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    // Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    assert_cookie_present(&login_response, "session_id");

    let json: AuthResponse = login_response.json();
    assert_eq!(json.user.email, test_user.email);
    assert_eq!(json.message, "Login successful");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_login_invalid_credentials() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // Register user first
    let _register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    // Login with wrong password
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: "wrongpassword".to_string(),
        })
        .await;

    assert_status_code(&login_response, 401);

    let json: Value = login_response.json();
    assert_eq!(json["error"].as_str().unwrap(), "Unauthorized");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_login_nonexistent_user() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Login with non-existent email
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "password".to_string(),
        })
        .await;

    assert_status_code(&login_response, 401);

    let json: Value = login_response.json();
    assert_eq!(json["error"].as_str().unwrap(), "Unauthorized");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_logout() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // Register and login
    let _register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    // Logout
    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    let json: Value = logout_response.json();
    assert_eq!(json["message"].as_str().unwrap(), "Logout successful");

    // Verify session is destroyed - try to access protected route
    let me_response = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&me_response, 401);

    ctx.cleanup().await;
}
