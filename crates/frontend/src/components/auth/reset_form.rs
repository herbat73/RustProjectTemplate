use std::str::FromStr;

use dioxus::prelude::*;
use email_address::EmailAddress;
use types::{
    auth::AuthErrorType,
    user::{ResetUser, UserInfo},
};

use crate::app::AppRoute;
use crate::components::{button::Button, error_message::ErrorMessage, input::Input};
use crate::hooks::USER_INFO;
use crate::services::{self, AuthError, AuthStorage};

/// Reads `email` and `key` from the current URL query string (`/reset?key=..&email=..`).
fn query_params() -> Option<(EmailAddress, String)> {
    let search = web_sys::window()?.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    let email = EmailAddress::from_str(&params.get("email")?).ok()?;
    let key = params.get("key")?;
    Some((email, key))
}

#[component]
pub fn ResetForm() -> Element {
    let Some((email, key)) = use_hook(query_params) else {
        return rsx! {
            ErrorMessage { message: "Invalid or missing password reset link." }
        };
    };

    let initial_user = ResetUser {
        email_address: email,
        pass: String::new(),
    };
    let mut error_state = use_signal(|| None::<AuthError>);
    let mut reset_user = use_signal(|| initial_user.clone());
    let mut confirm_pass = use_signal(String::new);

    let handle_reset = use_callback(move |_: ()| {
        let key = key.clone();
        let initial_user = initial_user.clone();
        spawn(async move {
            let response = services::auth::reset_user(reset_user.read().clone(), key).await;
            if let Some(error) = error_state.read().clone() {
                if matches!(error.body().error_type, AuthErrorType::PasswordDoesNotMatch) {
                    return;
                }
            }
            match response {
                Ok(_) => {
                    *USER_INFO.write() = UserInfo::default();
                    reset_user.set(initial_user);
                    AuthStorage::clear();
                    navigator().push(AppRoute::Login {});
                }
                Err(error) => error_state.set(Some(error)),
            }
        });
    });

    rsx! {
        form {
            class: "flex flex-col w-64 space-y-2 text-center
            text-slate-800 dark:text-slate-100",
            onsubmit: move |ev| {
                ev.prevent_default();
                handle_reset(());
            },
            p { "Enter your new password" }
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            Input {
                r#type: "password",
                placeholder: "Password",
                oninput: move |ev: FormEvent| {
                    error_state.set(None);
                    reset_user.write().pass = ev.value();
                },
                value: reset_user.read().pass.clone(),
            }
            Input {
                r#type: "password",
                placeholder: "Confirm password",
                oninput: move |ev: FormEvent| {
                    let value = ev.value();
                    if value != reset_user.read().pass {
                        error_state
                            .set(Some(AuthError::from_error_type(AuthErrorType::PasswordDoesNotMatch)));
                    } else {
                        error_state.set(None);
                    }
                    confirm_pass.set(value);
                },
                value: confirm_pass.read().clone(),
            }
            Button { r#type: "submit", "Confirm" }
        }
    }
}
