use dioxus::prelude::*;
use types::user::UserInfo;

use crate::services;

/// Globally shared info about the signed-in user (empty uuid means signed out).
pub static USER_INFO: GlobalSignal<UserInfo> = Signal::global(UserInfo::default);

/// Returns the current user info, fetching it from the server if it is not loaded yet.
pub fn use_user_info() -> UserInfo {
    use_effect(move || {
        if USER_INFO.read().uuid.is_empty() {
            spawn(async move {
                let user_info = services::user::get_user_info().await;
                if user_info != *USER_INFO.peek() {
                    *USER_INFO.write() = user_info;
                }
            });
        }
    });

    USER_INFO.read().clone()
}
