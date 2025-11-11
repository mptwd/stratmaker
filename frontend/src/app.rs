use crate::error::ErrorResponse;
use crate::routes::Route;
use crate::strategy::StrategyResumed;
use gloo_net::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

// App Page - Strategy List
#[function_component(AppPage)]
pub fn app_page() -> Html {
    let error = use_state(|| Option::<String>::None);
    let strategies: UseStateHandle<Vec<StrategyResumed>> = use_state(|| vec![]);
    {
        let strategies = strategies.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            let strategies = strategies.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let response = Request::get("/api/strategy/all").send().await;
                match response {
                    // We got a response, and it's OK.
                    Ok(r) if r.status() == 200 => {
                        let fetched_strategies = r.json::<Vec<StrategyResumed>>().await;
                        match fetched_strategies {
                            Ok(strats) => {
                                strategies.set(strats);
                            }
                            Err(e) => error.set(Some(format!("Failed to login: {}", e))),
                        }
                    }
                    // We got a response, but it's an error.
                    Ok(r) => {
                        let err_msg = r.json::<ErrorResponse>().await;
                        match err_msg {
                            Ok(e_msg) => {
                                error.set(Some(format!("Failed to login: {}", e_msg.error)))
                            }
                            Err(_) => {
                                error.set(Some(format!("Failed to login: {}", r.status_text())))
                            }
                        }
                    }
                    // We did not even get a response.
                    Err(e) => {
                        error.set(Some(format!("Failed to login: {}", e)));
                    }
                }
            });
        })
    }

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
                    <h1 class="logo">{"StrategyMaker"}</h1>
                    <div class="nav-links">
                        <button class="btn-secondary" onclick={on_logout}>{"Logout"}</button>
                    </div>
                </div>
            </nav>
            <div class="app-content">
                <div class="container">
                    {if let Some(err) = (*error).as_ref() {
                        html! { <div class="error-message">{err}</div> }
                    } else {
                        html! {}
                    }}
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
                                <div class="strategy-card" key={strategy.id.to_string()}>
                                    <div class="strategy-header">
                                        <h3>{&strategy.title}</h3>
                                    </div>
                                    //<p class="strategy-date">{"Created: "}{&strategy.created_at}</p>
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
