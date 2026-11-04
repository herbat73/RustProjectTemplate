use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::hooks::{USER_INFO, use_user_info};

#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let _user_info = use_user_info();

    use_effect(move || {
        if USER_INFO.read().uuid.is_empty() {
            navigator().push(AppRoute::Login {});
        }
    });

    rsx! { {children} }
}
