use crate::routes::Route;
use crate::strategy::{Strategy, StrategyMeta};
use yew::prelude::*;
use yew_router::prelude::*;

// App Page - Strategy List
#[function_component(AppPage)]
pub fn app_page() -> Html {
    let strategies = use_state(|| {
        vec![
            Strategy {
                id: "1".to_string(),
                name: "SMA Crossover Strategy".to_string(),
                meta: StrategyMeta {
                    strategy_type: "spot".to_string(),
                },
                actions: vec![],
                created_at: "2025-01-15".to_string(),
            },
            Strategy {
                id: "2".to_string(),
                name: "RSI Momentum Strategy".to_string(),
                meta: StrategyMeta {
                    strategy_type: "spot".to_string(),
                },
                actions: vec![],
                created_at: "2025-01-10".to_string(),
            },
        ]
    });

    let navigator = use_navigator().unwrap();

    let on_logout = {
        Callback::from(move |_| {
            web_sys::window().unwrap().location().set_href("/").unwrap();
        })
    };

    html! {
        <div class="app-page">
            <nav class="app-navbar">
                <div class="container">
                    <h1 class="logo">{"StrategyBuilder"}</h1>
                    <div class="nav-links">
                        <button class="btn-secondary" onclick={on_logout}>{"Logout"}</button>
                    </div>
                </div>
            </nav>
            <div class="app-content">
                <div class="container">
                    <div class="page-header">
                        <h1>{"Your Strategies"}</h1>
                        <Link<Route> to={Route::NewStrategy} classes="btn-primary">
                            {"+ New Strategy"}
                        </Link<Route>>
                    </div>

                    <div class="strategies-grid">
                        {for strategies.iter().map(|strategy| {
                            let strategy_id = strategy.id.clone();
                            html! {
                                <div class="strategy-card" key={strategy.id.clone()}>
                                    <div class="strategy-header">
                                        <h3>{&strategy.name}</h3>
                                        <span class="strategy-type">{&strategy.meta.strategy_type}</span>
                                    </div>
                                    <p class="strategy-date">{"Created: "}{&strategy.created_at}</p>
                                    <div class="strategy-actions">
                                        <Link<Route> to={Route::Strategy { id: strategy_id }} classes="btn-primary btn-small">
                                            {"View & Test"}
                                        </Link<Route>>
                                    </div>
                                </div>
                            }
                        })}
                    </div>

                    {if strategies.is_empty() {
                        html! {
                            <div class="empty-state">
                                <h2>{"No strategies yet"}</h2>
                                <p>{"Create your first strategy to get started!"}</p>
                                <Link<Route> to={Route::NewStrategy} classes="btn-primary">
                                    {"Create Strategy"}
                                </Link<Route>>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        </div>
    }
}
