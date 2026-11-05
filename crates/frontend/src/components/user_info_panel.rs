use dioxus::prelude::*;

use crate::components::card::{Card, CardContent, CardHeader, CardTitle};
use crate::hooks::use_user_info;

#[component]
pub fn UserInfoPanel() -> Element {
    let user_info = use_user_info();

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Account" }
            }
            CardContent {
                p { "UUID: {user_info.uuid}" }
                p { "Username: {user_info.username}" }
                p { "Email: {user_info.email}" }
                p { "Is Admin: {user_info.is_admin}" }
            }
        }
    }
}
