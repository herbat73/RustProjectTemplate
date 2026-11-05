use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::components::{button::Button, error_message::ErrorMessage, input::Input};
use crate::services::{self, AuthError, AuthStorage};

#[component]
pub fn RequestResetForm() -> Element {
    let mut error_state = use_signal(|| None::<AuthError>);
    let mut reset_email = use_signal(String::new);

    let handle_reset = move || {
        spawn(async move {
            let email = reset_email.read().clone();
            match services::auth::request_reset(email).await {
                Ok(_) => {
                    reset_email.set(String::new());
                    AuthStorage::clear();
                    navigator().push(AppRoute::Login {});
                }
                Err(error) => error_state.set(Some(error)),
            }
        });
    };

    rsx! {
        form {
            class: "flex flex-col w-64 space-y-2 text-center
            text-slate-800 dark:text-slate-100",
            onsubmit: move |ev| {
                ev.prevent_default();
                handle_reset();
            },
            p { "Enter your email to reset your password" }
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            Input {
                r#type: "email",
                placeholder: "Email",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    reset_email.set(ev.value());
                },
                value: reset_email.read().clone(),
            }
            Button { r#type: "submit", "Confirm" }
        }
    }
}
