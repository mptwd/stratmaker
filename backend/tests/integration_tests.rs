mod common;

use axum_test::TestServer;
use cookie::Cookie;
use serde_json::Value;
use uuid::Uuid;
use backend::models::{AuthResponse, RegisterRequest, LoginRequest, UserResponse};

use common::{assertions::*, TestContext, TestUser};

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
            username: test_user.username.clone(),
            email: test_user.email.clone(),
            password: test_user.password.clone(),
        })
        .await;
    assert_success_response(&response1);

    // Second registration with same email - should fail
    let response2 = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: test_user.username.clone(),
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
async fn test_user_registration_validation() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();

    // Test short password
    let response = server
        .post("/api/register")
        .json(&RegisterRequest {
            username: "testusername".to_string(),
            email: "test@example.com".to_string(),
            password: "123".to_string(),
        })
        .await;

    assert_status_code(&response, 400);
    
    let json: Value = response.json();
    assert!(json["error"].as_str().unwrap().contains("at least 6 characters"));

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
    assert!(json["error"].as_str().unwrap().contains("Email is required"));
    
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
    assert_eq!(json["message"].as_str().unwrap(), "This is a protected route");
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
    let mut conn = ctx.redis_client.get_multiplexed_async_connection().await.unwrap();
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
    let db_user = ctx.db.get_user_by_email(&test_user.email).await.unwrap().unwrap();
    
    // Password should not be stored in plaintext
    assert_ne!(db_user.password_hash, test_user.password);
    
    // Password hash should start with Argon2 identifier
    assert!(db_user.password_hash.starts_with("$argon2"));
    
    // Should be able to verify password
    let is_valid = backend::auth::verify_password(&test_user.password, &db_user.password_hash).unwrap();
    assert!(is_valid);
    
    // Wrong password should not verify
    let is_invalid = backend::auth::verify_password("wrongpassword", &db_user.password_hash).unwrap();
    assert!(!is_invalid);
    
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
