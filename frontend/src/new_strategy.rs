use crate::routes::Route;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

// New Strategy Page
#[function_component(NewStrategyPage)]
pub fn new_strategy_page() -> Html {
    let strategy_json = use_state(|| {
        r#"{
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

    let strategy_name = use_state(|| "My Strategy".to_string());
    let error = use_state(|| Option::<String>::None);

    let on_name_change = {
        let strategy_name = strategy_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            strategy_name.set(input.value());
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
        let strategy_name = strategy_name.clone();
        let error = error.clone();
        Callback::from(move |_| {
            // Validate JSON
            match serde_json::from_str::<serde_json::Value>(&*strategy_json) {
                Ok(_) => {
                    error.set(None);
                    // TODO: Save to backend
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Strategy saved! (Backend integration needed)")
                        .unwrap();
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
                    <h1 class="logo">{"StrategyBuilder"}</h1>
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
                                value={(*strategy_name).clone()}
                                oninput={on_name_change}
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
