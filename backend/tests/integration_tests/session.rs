use axum_test::TestServer;
use backend::models::{LoginRequest, RegisterRequest};
use cookie::Cookie;
use serde_json::Value;

use crate::helper::{
    TestContext, TestUser,
    assertions::{assert_status_code, assert_success_response, extract_cookie_value},
};

#[tokio::test]
async fn test_session_expiration() {
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

    // Manually expire the session in Redis
    let mut conn = ctx
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    let _: () = redis::cmd("EXPIRE")
        .arg(&session_cookie)
        .arg(0)
        .query_async(&mut conn)
        .await
        .unwrap();

    // Wait a bit to ensure expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to access protected route with expired session
    let me_response = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&me_response, 401);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_invalid_session_id() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Try to access protected route with invalid session ID
    let me_response = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", "invalid-session-id"))
        .await;

    assert_status_code(&me_response, 401);

    let json: Value = me_response.json();
    assert_eq!(json["error"].as_str().unwrap(), "Unauthorized");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_sessions() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // Register user
    let _register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    // Login multiple times to create multiple sessions
    let login1 = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    let login2 = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    let session1 = extract_cookie_value(&login1, "session_id").unwrap();
    let session2 = extract_cookie_value(&login2, "session_id").unwrap();

    // Both sessions should be valid
    let me1 = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session1))
        .await;

    let me2 = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session2))
        .await;

    assert_success_response(&me1);
    assert_success_response(&me2);

    // Logout one session
    let _logout1 = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session1))
        .await;

    // First session should be invalid, second should still work
    let me1_after = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session1))
        .await;

    let me2_after = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session2))
        .await;

    assert_status_code(&me1_after, 401);
    assert_success_response(&me2_after);

    ctx.cleanup().await;
}
