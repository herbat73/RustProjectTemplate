use dioxus::prelude::*;
use types::user::UserInfo;

use crate::components::{
    buttons::button::Button, error_message::ErrorMessage, user_info_panel::UserInfoPanel,
};
use crate::hooks::USER_INFO;
use crate::services::{self, AuthError};

#[component]
pub fn UserView() -> Element {
    let mut error_state = use_signal(|| None::<AuthError>);

    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col justify-center items-center h-full space-y-2",
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            UserInfoPanel {}
            div { class: "flex flex-row space-x-4",
                Button {
                    label: "Logout",
                    onclick: move |_| {
                        services::auth::logout_user();
                        *USER_INFO.write() = UserInfo::default();
                    },
                }
                Button {
                    label: "Test Auth",
                    onclick: move |_| {
                        spawn(async move {
                            if let Err(error) = services::auth::test_auth_route().await {
                                error_state.set(Some(error));
                            }
                        });
                    },
                }
            }
        }
    }
}
