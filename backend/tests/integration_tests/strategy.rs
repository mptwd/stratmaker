use axum_test::TestServer;
use backend::{
    models::{
        CreateStrategyRequest, GetStrategyRequest, LoginRequest, RegisterRequest, Strategy,
        StrategyResumed,
    },
    validators::strategy_validator::StrategyContent,
};
use cookie::Cookie;

use crate::helper::{
    TestContext, TestStrategy, TestUser,
    assertions::{
        assert_json_contains_field, assert_json_field, assert_status_code, assert_success_response,
        extract_cookie_value,
    },
};

#[tokio::test]
async fn test_create_strategy_success() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let json = r#"
        {
          "meta": {
            "type": "spot"
          },
          "actions": [
            {
              "type": "buy",
              "w": 0.8,
              "cond": {
                "gt": {
                  "l": "sma_10",
                  "r": "sma_50"
                }
              }
            }
          ]
        }"#;

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

    assert!(
        all_strats
            .iter()
            .any(|s| s.title == create_strat1_json.title && s.title == test_strat1.title)
    );
    assert!(
        all_strats
            .iter()
            .any(|s| s.title == create_strat2_json.title && s.title == test_strat2.title)
    );
    assert!(
        all_strats
            .iter()
            .any(|s| s.title == create_strat3_json.title && s.title == test_strat3.title)
    );
    assert!(
        all_strats
            .iter()
            .any(|s| s.title == create_strat4_json.title && s.title == test_strat4.title)
    );

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_create_duplicate_title_strategy() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();

    let json = r#"
        {
            "meta": {
                "type": "spot"
            },
            "actions": [
            {
                "type": "sell",
                "w": 1.0,
                "cond": {
                    "and": {
                        "conds": [
                        {
                            "lt": {
                                "l": "rsi",
                                "r": 30
                            }
                        },
                        {
                            "gt": {
                                "l": "macd",
                                "r": 0
                            }
                        }
                        ]
                    }
                }
            }
            ]
        }"#;

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

/*
#[tokio::test]
async fn test_delete_strategy() {
    let ctx = TestContext::new().await;
    let server = TestServer::new(ctx.app.clone()).unwrap();
    let test_user = TestUser::new();
    let test_strat = TestStrategy::new();

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
            title: test_strat.title.clone(),
            content: test_strat.content.clone(),
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&create_strat_response);

    let created_strat: Strategy = create_strat_response.json();

    let delete_strat_response = server
        .post("/api/strategy/delete")
        .json(&created_strat.id)
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&delete_strat_response);

    let delete_json = &delete_strat_response.json();

    assert_json_contains_field(delete_json, "num_del");
    assert_json_field(delete_json, "num_del", "1");

    ctx.cleanup().await;
}
*/
