mod common;

use axum_test::TestServer;
use backend::{models::{AuthResponse, CreateStrategyRequest, GetStrategyRequest, LoginRequest, RegisterRequest, Strategy, StrategyResumed, UserResponse}, validators::StrategyContent};
use cookie::Cookie;
use serde_json::Value;
use uuid::Uuid;

use common::{TestContext, TestUser, assertions::*};

use crate::common::TestStrategy;

/*
 * test_user_registration_success
 * test_user_registration_duplicate_email
 * test_user_registration_email_validation
 * test_user_registration_duplicate_username
 * test_user_registration_username_validation
 * test_user_registration_password_validation
 * test_user_login_success
 * test_user_login_invalid_credentials
 * test_user_login_nonexistent_user
 * test_get_current_user_authenticated
 * test_get_current_user_unauthenticated
 * test_protected_route_authenticated
 * test_protected_route_unauthenticated
 * test_user_logout
 * test_session_expiration
 * test_invalid_session_id
 * test_concurrent_sessions
 * test_password_hashing_security
 * test_create_strategy_success
 * test_get_strategies_success
 * test_create_strategy_failure <- skipped for now
 * test_create_duplicate_title_strategy
 * test_full_authentication_flow
 */

#[tokio::test]
async fn test_user_registration_success() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&response);

    let json: AuthResponse = response.json();
    assert_eq!(json.user.email, test_user.email);
    assert_eq!(json.message, "User registered successfully");
    assert!(Uuid::parse_str(&json.user.id.to_string()).is_ok());

    // Verify user exists in database
    let db_user = ctx.db.get_user_by_email(&test_user.email).await.unwrap();
    assert!(db_user.is_some());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_registration_duplicate_email() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // First registration - should succeed
    let response1 = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "a_username".to_string(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;
    assert_success_response(&response1);

    // Second registration with same email - should fail
    let response2 = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "another_username".to_string(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_status_code(&response2, 409);

    let json: Value = response2.json();
    assert!(json["error"].as_str().unwrap().contains("already exists"));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_registration_email_validation() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Test empty email
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "".to_string(),
            password: "testpass123".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Email is required")
    );

    // Test email to long
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@waytolong.com".to_string(),
            password: "Testpass1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Email cannot be greater than 255 characters")
    );

    // Test email is wrong
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "notavalidemail".to_string(),
            password: "Testpass1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid email")
    );

    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "not avalidemail@nop.com".to_string(),
            password: "Testpass1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid email")
    );

    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "notavalidemail@nop".to_string(),
            password: "Testpass1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid email")
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_registration_duplicate_username() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // First registration - should succeed
    let response1 = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: "email1@email.com".to_string(),
            password: test_user.password.clone(),
        })
        .await;
    assert_success_response(&response1);

    // Second registration with same email - should fail
    let response2 = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: "email2@email.com".to_string(),
            password: test_user.password.clone(),
        })
        .await;

    assert_status_code(&response2, 409);

    let json: Value = response2.json();
    assert!(json["error"].as_str().unwrap().contains("already taken"));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_registration_username_validation() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Test empty username
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Username is required")
    );

    // Test username too short
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "ab".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Username cannot be smaller than 3 characters")
    );

    // Test username too long
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "this_is_a_very_long_username_over_25_chars".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Username cannot be greater than 25 characters")
    );


    // Test username double hyphen
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "a--b".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid username")
    );


    // Test username ends with _
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "username_".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid username")
    );

    // Test username doesn't have any letter
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "1234".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Must be a valid username")
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_user_registration_password_validation() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();


    // Test empty password
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password is required")
    );

    // Test too short password
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "ab".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password must be at least 12 characters long")
    );

    // Test too long password
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password cannot be longer than 128 characters")
    );

    // Test password respects only 1 rule
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "abcdefghijklmn".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password must contain at least 3 of these 4 : lowercase, uppercase, digit, special character")
    );

    // Test password respects only 2 rule
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "Abcdefghijklmn".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password must contain at least 3 of these 4 : lowercase, uppercase, digit, special character")
    );

    // Test password has more than 3 identical characters in a row
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "ThisIs1lmostAAAAAAgoodPassword".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password cannot contain 4 identical characters in a row")
    );

    // Test password cannot start with white space
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: " TestPassword1234".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password cannot start or end with whitespace")
    );

    // Test password cannot end with white space
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "TestPassword1234 ".to_string(),
        })
        .await;

    assert_status_code(&response, 400);

    let json: Value = response.json();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("Password cannot start or end with whitespace")
    );

    ctx.cleanup().await;
}

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

#[tokio::test]
async fn test_protected_route_authenticated() {
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

    // Access protected route
    let protected_response = server
        .get("/api/protected")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&protected_response);

    let json: Value = protected_response.json();
    assert_eq!(
        json["message"].as_str().unwrap(),
        "This is a protected route"
    );
    assert!(json["user_id"].as_str().is_some());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_protected_route_unauthenticated() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Try to access protected route without authentication
    let protected_response = server.get("/api/protected").await;

    assert_status_code(&protected_response, 401);

    let json: Value = protected_response.json();
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

#[tokio::test]
async fn test_password_hashing_security() {
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

    // Verify password is hashed in database
    let db_user = ctx
        .db
        .get_user_by_email(&test_user.email)
        .await
        .unwrap()
        .unwrap();

    // Password should not be stored in plaintext
    assert_ne!(db_user.password_hash, test_user.password);

    // Password hash should start with Argon2 identifier
    assert!(db_user.password_hash.starts_with("$argon2"));

    // Should be able to verify password
    let is_valid =
        backend::auth::verify_password(&test_user.password, &db_user.password_hash).unwrap();
    assert!(is_valid);

    // Wrong password should not verify
    let is_invalid =
        backend::auth::verify_password("wrongpassword", &db_user.password_hash).unwrap();
    assert!(!is_invalid);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_create_strategy_success() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let json = r#"
        {
          "condition": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_10",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let strat: StrategyContent = serde_json::from_str(&json).unwrap();

    // 1. Register
    let register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
    .await;

    assert_success_response(&register_response);

    // 2. Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    let create_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat_response);
    let create_strat_json: Strategy = create_strat_response.json();

    let get_strat_response = server
        .get("/api/strategy")
        .json(&GetStrategyRequest {
            id: create_strat_json.id,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&get_strat_response);
    let get_strat_json: Strategy = get_strat_response.json();

    assert_eq!(get_strat_json.id, create_strat_json.id);
    assert_eq!(get_strat_json.title, create_strat_json.title);


    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_get_strategies_success() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();
    let test_strat1 = TestStrategy::new();
    let test_strat2 = TestStrategy::new();
    let test_strat3 = TestStrategy::new();
    let test_strat4 = TestStrategy::new();

    // 1. Register
    let register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
    .await;

    assert_success_response(&register_response);

    // 2. Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    let create_strat1_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: test_strat1.title.clone(),
            content: test_strat1.content.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat1_response);

    let create_strat2_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: test_strat2.title.clone(),
            content: test_strat2.content.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat2_response);
     
    let create_strat3_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: test_strat3.title.clone(),
            content: test_strat3.content.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat3_response);

    let create_strat4_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: test_strat4.title.clone(),
            content: test_strat4.content.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat4_response);

    let create_strat1_json: Strategy = create_strat1_response.json();
    let create_strat2_json: Strategy = create_strat2_response.json();
    let create_strat3_json: Strategy = create_strat3_response.json();
    let create_strat4_json: Strategy = create_strat4_response.json();

    let get_all_strats_response = server
        .get("/api/strategy/all")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&get_all_strats_response);
    let all_strats: Vec<StrategyResumed> = get_all_strats_response.json();

    assert!(all_strats.iter().any(|s| s.id == create_strat1_json.id));
    assert!(all_strats.iter().any(|s| s.id == create_strat2_json.id));
    assert!(all_strats.iter().any(|s| s.id == create_strat3_json.id));
    assert!(all_strats.iter().any(|s| s.id == create_strat4_json.id));

    assert!(all_strats.iter().any(|s| s.title == create_strat1_json.title && s.title == test_strat1.title));
    assert!(all_strats.iter().any(|s| s.title == create_strat2_json.title && s.title == test_strat2.title));
    assert!(all_strats.iter().any(|s| s.title == create_strat3_json.title && s.title == test_strat3.title));
    assert!(all_strats.iter().any(|s| s.title == create_strat4_json.title && s.title == test_strat4.title));

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}

/*
#[tokio::test]
async fn test_create_strategy_failure() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let empty_json = "";

    let badly_named_condition_json = r#"
        {
          "wrong": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_10",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let badly_named_operator_json = r#"
        {
          "condition": {
            "op": "bad_operator",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_10",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let badly_named_indicator_json = r#"
        {
          "condition": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "wrong_indicator",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let badly_named_action_json = r#"
        {
          "condition": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_30",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "WRONG_ACTION",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let empty_strat: StrategyContent = serde_json::from_str(&empty_json).unwrap();
    let badly_named_condition_strat: StrategyContent = serde_json::from_str(&badly_named_condition_json).unwrap();
    let badly_named_operator_strat: StrategyContent = serde_json::from_str(&badly_named_operator_json).unwrap();
    let badly_named_indicator_strat: StrategyContent = serde_json::from_str(&badly_named_indicator_json).unwrap();
    let badly_named_action_strat: StrategyContent = serde_json::from_str(&badly_named_action_json).unwrap();

    // 1. Register
    let register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
    .await;

    assert_success_response(&register_response);

    // 2. Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    let empty_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: empty_strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&empty_strat_response, 401);

    let badly_named_condition_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: badly_named_condition_strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&badly_named_condition_strat_response, 401);

    let badly_named_operator_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: badly_named_operator_strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&badly_named_operator_strat_response, 401);

    let badly_named_indicator_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: badly_named_indicator_strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&badly_named_indicator_strat_response, 401);

    let badly_named_action_strat_response = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: badly_named_action_strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&badly_named_action_strat_response, 401);

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}
*/

#[tokio::test]
async fn test_create_duplicate_title_strategy() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let json = r#"
        {
          "condition": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_10",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#.to_string();

    let strat: StrategyContent = serde_json::from_str(&json).unwrap();

    // 1. Register
    let register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
    .await;

    assert_success_response(&register_response);

    // 2. Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    let create_strat_response1 = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: strat.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat_response1);

    let create_strat_response2 = server
        .post("/api/strategy/create")
        .json(&CreateStrategyRequest {
            title: "myStrat".to_string(),
            content: strat,
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&create_strat_response2, 409);

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}


#[tokio::test]
async fn test_full_authentication_flow() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    // 1. Register
    let register_response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
    .await;

    assert_success_response(&register_response);

    // 2. Login
    let login_response = server
        .post("/api/login")
        .json(&LoginRequest {
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;

    assert_success_response(&login_response);
    let session_cookie = extract_cookie_value(&login_response, "session_id").unwrap();

    // 3. Access protected resources multiple times
    for _ in 0..3 {
        let me_response = server
            .get("/api/me")
            .add_cookie(Cookie::new("session_id", &session_cookie))
            .await;
        assert_success_response(&me_response);

        let protected_response = server
            .get("/api/protected")
            .add_cookie(Cookie::new("session_id", &session_cookie))
            .await;
        assert_success_response(&protected_response);
    }

    // 4. Logout
    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    // 5. Verify access is denied after logout
    let final_me_response = server
        .get("/api/me")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_status_code(&final_me_response, 401);

    ctx.cleanup().await;
}
