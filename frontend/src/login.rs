use crate::routes::Route;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

// Login Page Component
#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| Option::<String>::None);

    let on_email_change = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            email.set(input.value());
        })
    };

    let on_password_change = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let error = error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if email.is_empty() || password.is_empty() {
                error.set(Some("Please fill in all fields".to_string()));
            } else {
                // TODO: Call your Axum backend API
                error.set(None);
                web_sys::window()
                    .unwrap()
                    .location()
                    .set_href("/app")
                    .unwrap();
            }
        })
    };

    html! {
        <div class="auth-page">
            <div class="auth-container">
                <div class="auth-header">
                    <Link<Route> to={Route::Home} classes="logo-link">
                        <h1>{"StrategyBuilder"}</h1>
                    </Link<Route>>
                    <h2>{"Welcome Back"}</h2>
                    <p>{"Sign in to continue building your strategies"}</p>
                </div>

                <form class="auth-form" onsubmit={on_submit}>
                    {if let Some(err) = (*error).as_ref() {
                        html! { <div class="error-message">{err}</div> }
                    } else {
                        html! {}
                    }}

                    <div class="form-group">
                        <label for="email">{"Email"}</label>
                        <input
                            type="email"
                            id="email"
                            placeholder="you@example.com"
                            value={(*email).clone()}
                            oninput={on_email_change}
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">{"Password"}</label>
                        <input
                            type="password"
                            id="password"
                            placeholder="••••••••"
                            value={(*password).clone()}
                            oninput={on_password_change}
                        />
                    </div>

                    <div class="form-footer">
                        <a href="#" class="forgot-link">{"Forgot password?"}</a>
                    </div>

                    <button type="submit" class="btn-primary btn-full">{"Sign In"}</button>
                </form>

                <div class="auth-footer">
                    <p>{"Don't have an account? "}
                        <Link<Route> to={Route::Register}>{"Sign up"}</Link<Route>>
                    </p>
                </div>
            </div>
        </div>
    }
}
