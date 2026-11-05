use dioxus::prelude::*;
use types::user::UserInfo;

use crate::components::{
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardContent},
};
use crate::services;

#[component]
pub fn UsersTable() -> Element {
    let mut users = use_signal(Vec::<UserInfo>::new);

    let refresh = move || {
        spawn(async move {
            if let Ok((_, data)) = services::user::get_all_users().await {
                users.set(data);
            }
        });
    };

    use_hook(refresh);

    rsx! {
        div { class: "w-11/12 my-10 overflow-y-auto",
            Card {
                CardContent {
                    table {
                        thead {
                            tr { class: "text-left",
                                th { "UUID" }
                                th { "Username" }
                                th { "Email" }
                                th { "Admin" }
                                th {}
                            }
                        }
                        tbody {
                            for user in users.read().iter().cloned() {
                                tr { key: "{user.uuid}",
                                    td { "{user.uuid}" }
                                    td { "{user.username}" }
                                    td { "{user.email}" }
                                    td {
                                        if user.is_admin {
                                            Badge { "Admin" }
                                        } else {
                                            Badge { variant: BadgeVariant::Secondary,
                                                "User"
                                            }
                                        }
                                    }
                                    td {
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            size: ButtonSize::Sm,
                                            onclick: {
                                                let uuid = user.uuid.clone();
                                                move |_| {
                                                    let uuid = uuid.clone();
                                                    spawn(async move {
                                                        let _ = services::user::delete_user(uuid).await;
                                                        refresh();
                                                    });
                                                }
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
