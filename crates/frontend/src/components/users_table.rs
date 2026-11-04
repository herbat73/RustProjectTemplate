use dioxus::prelude::*;
use types::user::UserInfo;

use crate::{components::buttons::button::Button, services};

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
        div { class: "w-11/12 flex flex-col h-min
        rounded-md text-lg font-strong overflow-y-auto
        border-slate-300 dark:border-slate-700 border
        h-10 px-4 py-2 my-10
        bg-slate-100 text-slate-800 shadow-md
        dark:bg-slate-900 dark:text-slate-100",
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
                            td { "{user.is_admin}" }
                            td {
                                Button {
                                    color: "bg-slate-200 text-slate-800 hover:bg-slate-300 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700",
                                    label: "Delete",
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
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
