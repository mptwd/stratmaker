use crate::{error::ErrorResponse, routes::Route, strategy::StrategyContent};
use gloo_net::http::Request;
use rmp_serde::to_vec_named;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub title: String,
    pub content: StrategyContent,
}

// New Strategy Page
#[function_component(NewStrategyPage)]
pub fn new_strategy_page() -> Html {
    let strategy_json = use_state(|| {
        r#"
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
        }"#
        .to_string()
    });

    let strategy_title = use_state(|| "Strategy1".to_string());
    let error = use_state(|| Option::<String>::None);

    let on_title_change = {
        let strategy_title = strategy_title.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            strategy_title.set(input.value());
        })
    };

    let on_json_change = {
        let strategy_json = strategy_json.clone();
        Callback::from(move |e: Event| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            strategy_json.set(textarea.value());
        })
    };

    let on_save = {
        let strategy_json = strategy_json.clone();
        let strategy_title = strategy_title.clone();
        let error = error.clone();
        Callback::from(move |_| {
            // Validate JSON
            let strategy_title = strategy_title.clone();
            let error = error.clone();
            let strategy_json = strategy_json.clone();
            match serde_json::from_str::<StrategyContent>(&*strategy_json) {
                Ok(content) => {
                    spawn_local(async move {
                        let strategy_title = strategy_title.clone();
                        let error = error.clone();

                        let content_msgpack =
                            to_vec_named(&content).expect("failed to encode message pack");
                        let payload_b64 = BASE64.encode(content_msgpack);

                        let response = Request::post("/api/strategy/create")
                            .json(&serde_json::json!({
                                    "title": *strategy_title,
                                    "content": payload_b64,
                            }))
                            .unwrap()
                            .send()
                            .await;

                        match response {
                            // We got a response, and it's OK.
                            Ok(r) if r.status() == 200 => {
                                web_sys::window()
                                    .unwrap()
                                    .location()
                                    .set_href("/app")
                                    .unwrap();
                            }
                            // We got a response, but it's an error.
                            Ok(r) => {
                                let err_msg = r.json::<ErrorResponse>().await;
                                match err_msg {
                                    Ok(e_msg) => error.set(Some(format!(
                                        "Failed to create strategy: {}",
                                        e_msg.error
                                    ))),
                                    Err(_) => error.set(Some(format!(
                                        "Failed to create strategy and no error message: {}",
                                        r.status_text()
                                    ))),
                                }
                            }
                            // We did not even get a response.
                            Err(e) => {
                                error.set(Some(format!(
                                    "Failed to create strategy completly: {}",
                                    e
                                )));
                            }
                        }
                    })
                    /*
                                        web_sys::window()
                                            .unwrap()
                                            .alert_with_message("Strategy saved! (Backend integration needed)")
                                            .unwrap();
                    */
                }
                Err(e) => {
                    error.set(Some(format!("Invalid JSON: {}", e)));
                }
            }
        })
    };

    html! {
        <div class="app-page">
            <nav class="app-navbar">
                <div class="container">
                    <h1 class="logo">{"StrategyMaker"}</h1>
                    <div class="nav-links">
                        <Link<Route> to={Route::App} classes="btn-secondary">{"← Back"}</Link<Route>>
                    </div>
                </div>
            </nav>
            <div class="app-content">
                <div class="container">
                    <h1>{"Create New Strategy"}</h1>

                    {if let Some(err) = (*error).as_ref() {
                        html! { <div class="error-message">{err}</div> }
                    } else {
                        html! {}
                    }}

                    <div class="strategy-editor">
                        <div class="form-group">
                            <label for="strategy-name">{"Strategy Name"}</label>
                            <input
                                type="text"
                                id="strategy-name"
                                value={(*strategy_title).clone()}
                                oninput={on_title_change}
                                class="strategy-name-input"
                            />
                        </div>

                        <div class="form-group">
                            <label for="strategy-json">{"Strategy Configuration (JSON)"}</label>
                            <textarea
                                id="strategy-json"
                                class="json-editor"
                                value={(*strategy_json).clone()}
                                onchange={on_json_change}
                            />
                        </div>

                        <div class="editor-actions">
                            <button class="btn-primary" onclick={on_save}>{"Save Strategy"}</button>
                            <Link<Route> to={Route::App} classes="btn-secondary">{"Cancel"}</Link<Route>>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
