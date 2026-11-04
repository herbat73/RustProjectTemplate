use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::hooks::{USER_INFO, use_user_info};

#[component]
pub fn AdminRoute(children: Element) -> Element {
    let _user_info = use_user_info();

    use_effect(move || {
        let user = USER_INFO.read();
        if user.uuid.is_empty() || !user.is_admin {
            navigator().push(AppRoute::Login {});
        }
    });

    rsx! { {children} }
}
