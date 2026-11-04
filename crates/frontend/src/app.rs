use dioxus::prelude::*;

use crate::components::{
    auth::{admin_route::AdminRoute, protected_route::ProtectedRoute},
    footer::Footer,
    header::Header,
};
use crate::hooks::use_user_info;
use crate::views::{
    admin_view::AdminView, chat::Chat, home::Home, login::Login, not_found::NotFound,
    register::Register, request_reset::RequestReset, reset::Reset, user_view::UserView,
    todo::Todo, weather::Weather,
};

/// App routes
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum AppRoute {
    #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/admin")]
        AdminPanel {},
        #[route("/settings")]
        UserPanel {},
        #[route("/chat")]
        ChatPage {},
        #[route("/weather")]
        WeatherPage {},
        #[route("/todo")]
        TodoPage {},
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/reset")]
        Reset {},
        #[route("/reset/request")]
        RequestReset {},
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}

#[component]
fn AdminPanel() -> Element {
    rsx! { AdminRoute { AdminView {} } }
}

#[component]
fn UserPanel() -> Element {
    rsx! { ProtectedRoute { UserView {} } }
}

#[component]
fn ChatPage() -> Element {
    rsx! { ProtectedRoute { Chat {} } }
}

#[component]
fn WeatherPage() -> Element {
    rsx! { ProtectedRoute { Weather {} } }
}

#[component]
fn TodoPage() -> Element {
    rsx! { ProtectedRoute { Todo {} } }
}

#[component]
fn Shell() -> Element {
    let _user_info = use_user_info();

    rsx! {
        div { class: "flex flex-col space-between w-screen h-screen bg-slate-50 dark:bg-slate-700 overflow-hidden",
            Header {}
            main { class: "grid grid-rows-24 grid-cols-12 auto-rows-auto h-full w-full py-12",
                Outlet::<AppRoute> {}
            }
            Footer {}
        }
    }
}

#[component]
pub fn App() -> Element {
    rsx! { Router::<AppRoute> {} }
}
