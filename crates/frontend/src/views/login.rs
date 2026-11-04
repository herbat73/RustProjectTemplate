use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::components::auth::login_form::LoginForm;
use crate::hooks::use_user_info;

#[component]
pub fn Login() -> Element {
    let user_info = use_user_info();

    use_effect(move || {
        if !user_info.uuid.is_empty() {
            navigator().push(AppRoute::Home {});
        }
    });

    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col justify-center items-center h-full space-y-4",
            LoginForm {}
            Link { to: AppRoute::Register {},
                div {
                    class: "cursor-pointer text-blue-600 dark:text-blue-400 underline
                    focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-3",
                    "Need an account?"
                }
            }
            Link { to: AppRoute::RequestReset {},
                div {
                    class: "cursor-pointer text-blue-600 dark:text-blue-400 underline
                    focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-3",
                    "Forgot password?"
                }
            }
        }
    }
}
