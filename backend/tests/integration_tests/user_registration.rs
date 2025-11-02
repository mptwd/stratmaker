use axum_test::TestServer;
use backend::models::{AuthResponse, RegisterRequest};
use serde_json::Value;
use uuid::Uuid;

use crate::helper::{
    TestContext, TestUser,
    assertions::{assert_status_code, assert_success_response},
};

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
