use axum::Router;
use backend::{
    AppState,
    auth::SessionStore,
    dataset_client::DatasetManagerClient,
    db::Database,
    validators::strategy_validator::{StrategyContent, StrategyValidator},
};
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

pub struct TestContext {
    pub app: Router,
    pub db: Database,
    pub session_store: SessionStore,
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
    pub dataset_manager: DatasetManagerClient,
    pub strat_validator: StrategyValidator,
}

impl TestContext {
    pub async fn new() -> Self {
        // Initialize tracing for tests (only once)
        let _ = tracing_subscriber::fmt::try_init();

        // Create test database
        let db_pool = create_test_database().await;
        let db = Database::from_pool(db_pool.clone());

        // Create test Redis connection
        let redis_url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6380".to_string());
        let redis_client = RedisClient::open(redis_url).expect("Failed to connect to Redis");
        let session_store = SessionStore::from_client(redis_client.clone());

        // Clean Redis before each test
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get Redis connection");
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .expect("Failed to flush Redis");

        let dataset_manager = DatasetManagerClient::new("http://localhost:8081");

        let mut valid_indicators = HashSet::new();
        valid_indicators.insert("sma_10".to_string());
        valid_indicators.insert("sma_50".to_string());
        valid_indicators.insert("rsi".to_string());
        valid_indicators.insert("macd".to_string());
        valid_indicators.insert("volume".to_string());
        let strat_validator = StrategyValidator::new(valid_indicators);

        let app_state = AppState {
            db: db.clone(),
            session_store: session_store.clone(),
            dataset_manager: dataset_manager.clone(),
            strat_validator: strat_validator.clone(),
        };
        let app = backend::create_app(app_state);

        Self {
            app,
            db,
            session_store,
            db_pool,
            redis_client,
            dataset_manager,
            strat_validator,
        }
    }

    pub async fn cleanup(&self) {
        // Clean up database
        sqlx::query("TRUNCATE TABLE users CASCADE")
            .execute(&self.db_pool)
            .await
            .expect("Failed to clean database");

        // Clean up Redis
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get Redis connection");
        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .expect("Failed to flush Redis");
    }
}

async fn create_test_database() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost:5432/test_backend".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!().run(&pool).await.expect("migration failed");

    pool
}

// Test user data
pub struct TestUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl TestUser {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            username: format!("test"),
            email: format!("test{}@example.com", uuid),
            password: "Testpass123-a".to_string(),
        }
    }
}

pub struct TestStrategy {
    pub title: String,
    pub content: StrategyContent,
}

impl TestStrategy {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
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

        Self {
            title: format!("test{}", uuid),
            content: serde_json::from_str(&json).unwrap(),
        }
    }
}

// Common test assertions
pub mod assertions {
    use axum_test::TestResponse;
    use serde_json::Value;

    pub fn assert_success_response(response: &TestResponse) {
        assert!(
            response.status_code().is_success(),
            "Expected success status, got: {}",
            response.status_code()
        );
    }

    pub fn assert_status_code(response: &TestResponse, expected: u16) {
        assert_eq!(
            response.status_code().as_u16(),
            expected,
            "Expected status {}, got: {}",
            expected,
            response.status_code()
        );
    }

    pub fn assert_json_field(json: &Value, field: &str, expected: &str) {
        assert_eq!(
            json.get(field).and_then(|v| v.as_str()),
            Some(expected),
            "Expected field '{}' to be '{}', got: {:?}",
            field,
            expected,
            json.get(field)
        );
    }

    pub fn assert_json_contains_field(json: &Value, field: &str) {
        assert!(
            json.get(field).is_some(),
            "Expected JSON to contain field '{}', got: {}",
            field,
            json
        );
    }

    pub fn assert_cookie_present(response: &TestResponse, cookie_name: &str) {
        let cookies: Vec<&str> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();

        let cookie_found = cookies
            .iter()
            .any(|cookie| cookie.starts_with(&format!("{}=", cookie_name)));

        assert!(
            cookie_found,
            "Expected cookie '{}' to be present. Found cookies: {:?}",
            cookie_name, cookies
        );
    }

    pub fn extract_cookie_value(response: &TestResponse, cookie_name: &str) -> Option<String> {
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|cookie| {
                if cookie.starts_with(&format!("{}=", cookie_name)) {
                    cookie
                        .split(';')
                        .next()
                        .and_then(|part| part.split('=').nth(1))
                        .map(|value| value.to_string())
                } else {
                    None
                }
            })
    }
}
