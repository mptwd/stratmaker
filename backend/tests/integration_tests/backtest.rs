use axum_test::TestServer;
use backend::{
    models::{
        BacktestStatus, CreateBacktestRequest, CreateStrategyRequest, GetStrategyRequest,
        LoginRequest, RegisterRequest, Strategy,
    },
    validators::strategy_validator::StrategyContent,
};
use chrono::DateTime;
use cookie::Cookie;

use crate::helper::{
    TestContext, TestUser,
    assertions::{assert_success_response, extract_cookie_value},
};

#[tokio::test]
pub async fn test_submit_backtest() {
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

    // TODO: Ask for a backtest
    // TODO: Check that the backtest is defined as a job in the queue
    // TODO: Check the status ?
    let request_backtest_response = server
        .post("/api/backtest")
        .json(&CreateBacktestRequest {
            strategy_id: get_strat_json.id,
            dataset: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            date_start: DateTime::from_timestamp_secs(1546300800).unwrap(), // Tue Jan 01 2019 00:00:00 GMT+0000
            date_end: DateTime::from_timestamp_secs(1577836800).unwrap(), // Wed Jan 01 2020 00:00:00 GMT+0000
        })
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&request_backtest_response);
    let request_backtest_status: BacktestStatus = request_backtest_response.json();

    assert_eq!(request_backtest_status, BacktestStatus::Pending);

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}
