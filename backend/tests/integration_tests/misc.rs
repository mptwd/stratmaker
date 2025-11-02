use axum_test::TestServer;
use backend::models::{LoginRequest, RegisterRequest};
use cookie::Cookie;

use crate::helper::{
    TestContext, TestUser,
    assertions::{assert_status_code, assert_success_response, extract_cookie_value},
};

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
