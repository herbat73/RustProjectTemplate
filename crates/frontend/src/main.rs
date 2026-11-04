mod app;
mod components;
mod graphics;
mod hooks;
mod services;
mod views;

use crate::app::App;

fn main() {
    services::create_http_clients();
    dioxus::launch(App);
}
