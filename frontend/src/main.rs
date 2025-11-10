mod app;
mod home;
mod login;
mod new_strategy;
mod register;
mod routes;
mod strategy;

use routes::Main;

fn main() {
    yew::Renderer::<Main>::new().render();
}
