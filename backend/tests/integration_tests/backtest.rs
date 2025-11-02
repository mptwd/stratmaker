use crate::{
    Cookie, CreateStrategyRequest, GetStrategyRequest, LoginRequest, RegisterRequest, Strategy,
    StrategyContent, TestContext, TestServer, TestUser, assert_success_response,
    extract_cookie_value,
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

    let logout_response = server
        .post("/api/logout")
        .add_cookie(Cookie::new("session_id", &session_cookie))
        .await;

    assert_success_response(&logout_response);

    ctx.cleanup().await;
}
