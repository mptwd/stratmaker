use axum_test::TestServer;
use backend::models::{LoginRequest, RegisterRequest, UserResponse};
use cookie::Cookie;
use serde_json::Value;
use uuid::Uuid;

use crate::helper::{
    TestContext, TestUser,
    assertions::{assert_status_code, assert_success_response, extract_cookie_value},
};

#[tokio::test]
async fn test_get_current_user_authenticated() {
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

    // Get current user
    let me_response = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&me_response);

    let json: UserResponse = me_response.json();
    assert_eq!(json.email, test_user.email);
    assert!(Uuid::parse_str(&json.id.to_string()).is_ok());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_get_current_user_unauthenticated() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Try to get current user without authentication
    let me_response = server.get("/api/me").await;

    assert_status_code(&me_response, 401);

    let json: Value = me_response.json();
    assert_eq!(json["error"].as_str().unwrap(), "Unauthorized");

    ctx.cleanup().await;
}
