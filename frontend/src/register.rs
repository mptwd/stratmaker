use crate::routes::Route;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

// Register Page Component
#[function_component(RegisterPage)]
pub fn register_page() -> Html {
    let name = use_state(|| String::new());
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());
    let error = use_state(|| Option::<String>::None);

    let on_name_change = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

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

    let on_confirm_password_change = {
        let confirm_password = confirm_password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            confirm_password.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let email = email.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        let error = error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if name.is_empty() || email.is_empty() || password.is_empty() {
                error.set(Some("Please fill in all fields".to_string()));
            } else if *password != *confirm_password {
                error.set(Some("Passwords do not match".to_string()));
            } else if password.len() < 8 {
                error.set(Some("Password must be at least 8 characters".to_string()));
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
                    <h2>{"Create Your Account"}</h2>
                    <p>{"Start building your financial strategies today"}</p>
                </div>

                <form class="auth-form" onsubmit={on_submit}>
                    {if let Some(err) = (*error).as_ref() {
                        html! { <div class="error-message">{err}</div> }
                    } else {
                        html! {}
                    }}

                    <div class="form-group">
                        <label for="name">{"Full Name"}</label>
                        <input
                            type="text"
                            id="name"
                            placeholder="John Doe"
                            value={(*name).clone()}
                            oninput={on_name_change}
                        />
                    </div>

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

                    <div class="form-group">
                        <label for="confirm-password">{"Confirm Password"}</label>
                        <input
                            type="password"
                            id="confirm-password"
                            placeholder="••••••••"
                            value={(*confirm_password).clone()}
                            oninput={on_confirm_password_change}
                        />
                    </div>

                    <button type="submit" class="btn-primary btn-full">{"Create Account"}</button>
                </form>

                <div class="auth-footer">
                    <p>{"Already have an account? "}
                        <Link<Route> to={Route::Login}>{"Sign in"}</Link<Route>>
                    </p>
                </div>
            </div>
        </div>
    }
}
