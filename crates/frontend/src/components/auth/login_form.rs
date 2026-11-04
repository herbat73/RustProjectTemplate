use dioxus::prelude::*;
use types::user::LoginUser;

use crate::app::AppRoute;
use crate::components::{buttons::button::Button, error_message::ErrorMessage, input::Input};
use crate::hooks::USER_INFO;
use crate::services::{self, AuthError};

#[component]
pub fn LoginForm() -> Element {
    let mut login_user = use_signal(LoginUser::default);
    let mut error_state = use_signal(|| None::<AuthError>);

    let handle_login = move || {
        spawn(async move {
            let user = login_user.read().clone();
            match services::auth::login_user(user).await {
                Ok(user_info) => {
                    *USER_INFO.write() = user_info;
                    login_user.set(LoginUser::default());
                    navigator().push(AppRoute::Home {});
                }
                Err(error) => error_state.set(Some(error)),
            }
        });
    };

    rsx! {
        form {
            class: "flex flex-col w-64 space-y-2",
            onsubmit: move |ev| {
                ev.prevent_default();
                handle_login();
            },
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            Input {
                input_type: "text",
                placeholder: "Username",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    login_user.write().username = ev.value();
                },
                value: login_user.read().username.clone(),
            }
            Input {
                input_type: "password",
                placeholder: "Password",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    login_user.write().pass = ev.value();
                },
                value: login_user.read().pass.clone(),
            }
            Button { onclick: move |_| handle_login(), label: "Login" }
        }
    }
}
