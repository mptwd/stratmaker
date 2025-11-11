use crate::{error::ErrorResponse, routes::Route};
use chrono::{DateTime, Utc};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::*;

/* ==== Strategy structs ==== */

#[derive(Clone, PartialEq, Deserialize)]
pub struct StrategyResumed {
    pub id: Uuid,
    pub title: String,
}

// Strategy data structures
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: StrategyContent,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StrategyType {
    Spot,
    Options,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    #[serde(rename = "type")]
    pub strategy_type: StrategyType,
    //#[serde(flatten)]
    //pub extra: std::collections::HashMap<String, rmpv::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    //Boolean(bool),
    Indicator(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Cond {
    And {
        conds: Vec<Cond>,
    },
    Or {
        conds: Vec<Cond>,
    },
    Not {
        cond: Box<Cond>,
    },
    #[serde(rename = "lt")]
    LessThan {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "gt")]
    GreaterThan {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "le")]
    LessThanOrEqual {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "ge")]
    GreaterThanOrEqual {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "eq")]
    Equal {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "neq")]
    NotEqual {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "bet")]
    Between {
        val: Box<Value>,
        min: Box<Value>,
        max: Box<Value>,
    },
    #[serde(rename = "xab")]
    CrossesAbove {
        l: Box<Value>,
        r: Box<Value>,
    },
    #[serde(rename = "xbe")]
    CrossesBelow {
        l: Box<Value>,
        r: Box<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub w: f64,
    pub cond: Cond,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyContent {
    pub meta: Meta,
    pub actions: Vec<Action>,
}

/* ==== Backtest structs ==== */

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BacktestResult {
    initial_balance: f64,
    final_balance: f64,
    total_return: f64,
    num_trades: i32,
    price_data: Vec<PricePoint>,
    balance_data: Vec<BalancePoint>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PricePoint {
    timestamp: String,
    price: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BalancePoint {
    timestamp: String,
    balance: f64,
}

// Strategy Detail Page with Backtest
#[derive(Properties, PartialEq)]
pub struct StrategyDetailProps {
    pub strategy_id: Uuid,
}

#[function_component(StrategyDetailPage)]
pub fn strategy_detail_page(props: &StrategyDetailProps) -> Html {
    let error = use_state(|| Option::<String>::None);
    let strategy: UseStateHandle<Option<Strategy>> = use_state(|| None);
    {
        let strategy = strategy.clone();
        let error = error.clone();
        let strategy_id = props.strategy_id;
        use_effect_with((), move |_| {
            let strategy = strategy.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // HACK: Using POST her even though it would be cleaner with a GET
                let response = Request::post("/api/strategy")
                    .json(&serde_json::json!({
                        "id": strategy_id,
                    }))
                    .unwrap()
                    .send()
                    .await;
                match response {
                    // We got a response, and it's OK.
                    Ok(r) if r.status() == 200 => {
                        let fetched_strategy = r.json::<Strategy>().await;
                        match fetched_strategy {
                            Ok(strat) => {
                                strategy.set(Some(strat));
                            }
                            Err(e) => error.set(Some(format!("Failed to fetch strategy: {}", e))),
                        }
                    }
                    // We got a response, but it's an error.
                    Ok(r) => {
                        let err_msg = r.json::<ErrorResponse>().await;
                        match err_msg {
                            Ok(e_msg) => {
                                error.set(Some(format!("Failed to load strat: {}", e_msg.error)))
                            }
                            Err(_) => error.set(Some(format!(
                                "Failed to load strat no message: {}",
                                r.status_text()
                            ))),
                        }
                    }
                    // We did not even get a response.
                    Err(e) => {
                        error.set(Some(format!("Failed to load anything: {}", e)));
                    }
                }
            });
        })
    }

    let backtest_result = use_state(|| Option::<BacktestResult>::None);
    let is_loading = use_state(|| false);

    let on_backtest = {
        let backtest_result = backtest_result.clone();
        let is_loading = is_loading.clone();
        let strategy = strategy.clone();

        Callback::from(move |_| {
            is_loading.set(true);

            // TODO: Replace with actual API call to your Axum backend
            // Example:
            // let strategy_json = serde_json::to_string(&*strategy).unwrap();
            // wasm_bindgen_futures::spawn_local(async move {
            //     let response = Request::post("/api/backtest")
            //         .json(&*strategy)
            //         .unwrap()
            //         .send()
            //         .await
            //         .unwrap();
            //     let result: BacktestResult = response.json().await.unwrap();
            //     backtest_result.set(Some(result));
            //     is_loading.set(false);
            // });

            // Mock data for demonstration
            use gloo_timers::callback::Timeout;
            let backtest_result = backtest_result.clone();
            let is_loading = is_loading.clone();

            Timeout::new(1000, move || {
                backtest_result.set(Some(BacktestResult {
                    initial_balance: 10000.0,
                    final_balance: 12500.0,
                    total_return: 25.0,
                    num_trades: 15,
                    price_data: vec![
                        PricePoint {
                            timestamp: "2024-01".to_string(),
                            price: 100.0,
                        },
                        PricePoint {
                            timestamp: "2024-02".to_string(),
                            price: 105.0,
                        },
                        PricePoint {
                            timestamp: "2024-03".to_string(),
                            price: 98.0,
                        },
                        PricePoint {
                            timestamp: "2024-04".to_string(),
                            price: 110.0,
                        },
                        PricePoint {
                            timestamp: "2024-05".to_string(),
                            price: 115.0,
                        },
                        PricePoint {
                            timestamp: "2024-06".to_string(),
                            price: 112.0,
                        },
                    ],
                    balance_data: vec![
                        BalancePoint {
                            timestamp: "2024-01".to_string(),
                            balance: 10000.0,
                        },
                        BalancePoint {
                            timestamp: "2024-02".to_string(),
                            balance: 10500.0,
                        },
                        BalancePoint {
                            timestamp: "2024-03".to_string(),
                            balance: 10200.0,
                        },
                        BalancePoint {
                            timestamp: "2024-04".to_string(),
                            balance: 11500.0,
                        },
                        BalancePoint {
                            timestamp: "2024-05".to_string(),
                            balance: 12000.0,
                        },
                        BalancePoint {
                            timestamp: "2024-06".to_string(),
                            balance: 12500.0,
                        },
                    ],
                }));
                is_loading.set(false);
            })
            .forget();
        })
    };

    let strategy_json =
        serde_json::to_string_pretty(&*strategy).unwrap_or_else(|_| "Error".to_string());

    html! {
    <div class="app-page">
        <nav class="app-navbar">
            <div class="container">
                <h1 class="logo">{"StrategyMaker"}</h1>
                <div class="nav-links">
                    <Link<Route> to={Route::App} classes="btn-secondary">{"← Back to Strategies"}</Link<Route>>
                </div>
            </div>
        </nav>
        <div class="app-content">
            <div class="container">
                {
                if let Some(strategy) = strategy.as_ref() {
                html! {
                <>
                <div class="page-header">
                    <h1>{&strategy.title}</h1>
                    <button
                        class="btn-primary"
                        onclick={on_backtest}
                        disabled={*is_loading}
                    >
                        {if *is_loading { "Running Backtest..." } else { "Run Backtest" }}
                    </button>
                </div>

                <div class="strategy-detail-grid">
                    <div class="strategy-json-view">
                        <h2>{"Strategy Configuration"}</h2>
                        <pre class="json-display">{strategy_json}</pre>
                    </div>

                    {if let Some(result) = (*backtest_result).as_ref() {
                    html! { <BacktestResults result={result.clone()} /> }
                    } else {
                    html! {
                    <div class="backtest-placeholder">
                        <h2>{"No Backtest Results Yet"}</h2>
                        <p>{"Click 'Run Backtest' to see how your strategy performs"}</p>
                    </div>
                    }
                    }}
                </div>
                </>
                }
                }
                else {
                html! {
                {
                if let Some(err) = (*error).as_ref() {
                html! { <div class="error-message">{ err }</div> }
                } else {
                html! { <div class="error-message">{ "No strategy loaded." }</div> }
                }
                }
                }
                }
                }
            </div>
        </div>
    </div>
    }
}

// Backtest Results Component
#[derive(Properties, PartialEq)]
pub struct BacktestResultsProps {
    result: BacktestResult,
}

#[function_component(BacktestResults)]
pub fn backtest_results(props: &BacktestResultsProps) -> Html {
    let result = &props.result;

    html! {
        <div class="backtest-results">
            <h2>{"Backtest Results"}</h2>

            <div class="metrics-grid">
                <div class="metric-card">
                    <span class="metric-label">{"Initial Balance"}</span>
                    <span class="metric-value">{format!("${:.2}", result.initial_balance)}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">{"Final Balance"}</span>
                    <span class="metric-value success">{format!("${:.2}", result.final_balance)}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">{"Total Return"}</span>
                    <span class="metric-value success">{format!("+{:.2}%", result.total_return)}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">{"Number of Trades"}</span>
                    <span class="metric-value">{result.num_trades}</span>
                </div>
            </div>

            <div class="charts-section">
                <div class="chart-container">
                    <h3>{"Price History"}</h3>
                    <div class="simple-chart">
                        {for result.price_data.iter().map(|point| {
                            let height = (point.price / 120.0 * 100.0).min(100.0);
                            html! {
                                <div class="chart-bar" style={format!("height: {}%", height)}>
                                    <span class="chart-label">{&point.timestamp}</span>
                                </div>
                            }
                        })}
                    </div>
                </div>

                <div class="chart-container">
                    <h3>{"Portfolio Balance"}</h3>
                    <div class="simple-chart">
                        {for result.balance_data.iter().map(|point| {
                            let height = (point.balance / 15000.0 * 100.0).min(100.0);
                            html! {
                                <div class="chart-bar balance-bar" style={format!("height: {}%", height)}>
                                    <span class="chart-label">{&point.timestamp}</span>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </div>

            <div class="data-table">
                <h3>{"Detailed Price Data"}</h3>
                <table>
                    <thead>
                        <tr>
                            <th>{"Date"}</th>
                            <th>{"Price"}</th>
                            <th>{"Balance"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {for result.price_data.iter().zip(result.balance_data.iter()).map(|(price, balance)| {
                            html! {
                                <tr>
                                    <td>{&price.timestamp}</td>
                                    <td>{format!("${:.2}", price.price)}</td>
                                    <td>{format!("${:.2}", balance.balance)}</td>
                                </tr>
                            }
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
