use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::AppPage;
use crate::home::LandingPage;
use crate::login::LoginPage;
use crate::new_strategy::NewStrategyPage;
use crate::register::RegisterPage;
use crate::strategy::StrategyDetailPage;

// Define routes
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/app")]
    App,
    #[at("/app/strategy/:id")]
    Strategy { id: String },
    #[at("/app/new")]
    NewStrategy,
}

// Main component with router
#[function_component(Main)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <LandingPage /> },
        Route::Login => html! { <LoginPage /> },
        Route::Register => html! { <RegisterPage /> },
        Route::App => html! { <AppPage /> },
        Route::Strategy { id } => html! { <StrategyDetailPage strategy_id={id} /> },
        Route::NewStrategy => html! { <NewStrategyPage /> },
    }
}
