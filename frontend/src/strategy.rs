use crate::routes::Route;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

// Strategy data structures
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub meta: StrategyMeta,
    pub actions: Vec<Action>,
    pub created_at: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyMeta {
    #[serde(rename = "type")]
    pub strategy_type: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub w: f64,
    pub cond: Condition,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    #[serde(flatten)]
    pub op: ConditionOp,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOp {
    Gt { l: String, r: String },
    Lt { l: String, r: String },
    Eq { l: String, r: String },
}

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
    pub strategy_id: String,
}

#[function_component(StrategyDetailPage)]
pub fn strategy_detail_page(props: &StrategyDetailProps) -> Html {
    let strategy = use_state(|| Strategy {
        id: props.strategy_id.clone(),
        name: "SMA Crossover Strategy".to_string(),
        meta: StrategyMeta {
            strategy_type: "spot".to_string(),
        },
        actions: vec![Action {
            action_type: "buy".to_string(),
            w: 0.8,
            cond: Condition {
                op: ConditionOp::Gt {
                    l: "sma_10".to_string(),
                    r: "sma_50".to_string(),
                },
            },
        }],
        created_at: "2025-01-15".to_string(),
    });

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
                    <h1 class="logo">{"StrategyBuilder"}</h1>
                    <div class="nav-links">
                        <Link<Route> to={Route::App} classes="btn-secondary">{"← Back to Strategies"}</Link<Route>>
                    </div>
                </div>
            </nav>
            <div class="app-content">
                <div class="container">
                    <div class="page-header">
                        <h1>{&strategy.name}</h1>
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
