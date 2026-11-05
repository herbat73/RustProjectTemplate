use dioxus::prelude::*;
use types::user::RegisterUser;

use crate::app::AppRoute;
use crate::components::{button::Button, error_message::ErrorMessage, input::Input};
use crate::hooks::USER_INFO;
use crate::services::{self, AuthError};

#[component]
pub fn RegisterForm() -> Element {
    let mut register_user = use_signal(RegisterUser::default);
    let mut error_state = use_signal(|| None::<AuthError>);

    let handle_register = move || {
        spawn(async move {
            let user = register_user.read().clone();
            match services::auth::register_user(user).await {
                Ok(user_info) => {
                    *USER_INFO.write() = user_info;
                    register_user.set(RegisterUser::default());
                    navigator().push(AppRoute::Login {});
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
                handle_register();
            },
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            Input {
                r#type: "text",
                placeholder: "Username",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    register_user.write().username = ev.value();
                },
                value: register_user.read().username.clone(),
            }
            Input {
                r#type: "password",
                placeholder: "Password",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    register_user.write().pass = ev.value();
                },
                value: register_user.read().pass.clone(),
            }
            Input {
                r#type: "email",
                placeholder: "Email",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    register_user.write().email = ev.value();
                },
                value: register_user.read().email.clone(),
            }
            Button { r#type: "submit", "Register" }
        }
    }
}
