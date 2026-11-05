use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::components::buttons::nav_button::NavButton;
use crate::hooks::use_user_info;

#[component]
pub fn Header() -> Element {
    let user_info = use_user_info();
    let logged_in = !user_info.uuid.is_empty();

    rsx! {
        header { class: "flex flex-row h-12 z-50 fixed top-0 left-0 right-0 border-[var(--primary-color-6)] border-b
            bg-[var(--primary-color-3)] shadow-md w-full
            justify-between justify-items-center items-center",
            div { class: "flex flex-row h-full",
                NavButton { label: "Home", destination: AppRoute::Home {} }
                if logged_in {
                    NavButton { label: "Chat", destination: AppRoute::ChatPage {} }
                }
            }
            div { class: "flex flex-row h-full pr-12",
                if logged_in {
                    if user_info.is_admin {
                        NavButton {
                            label: "Admin",
                            destination: AppRoute::AdminPanel {},
                        }
                    }
                    NavButton {
                        label: user_info.username.clone(),
                        destination: AppRoute::UserPanel {},
                    }
                } else {
                    NavButton { label: "Login", destination: AppRoute::Login {} }
                    NavButton { label: "Register", destination: AppRoute::Register {} }
                }
            }
        }
    }
}
