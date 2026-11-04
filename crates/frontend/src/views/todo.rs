use dioxus::prelude::*;
use types::todo::TodoListDto;

use crate::components::{
    buttons::button::Button, error_message::ErrorMessage, input::Input,
};
use crate::services::{self, AuthError};

#[component]
pub fn Todo() -> Element {
    let mut lists = use_signal(Vec::<TodoListDto>::new);
    let mut error_state = use_signal(|| None::<AuthError>);
    let mut new_list = use_signal(String::new);

    let refresh = move || {
        spawn(async move {
            match services::todo::get_todos().await {
                Ok(data) => {
                    error_state.set(None);
                    lists.set(data.lists);
                }
                Err(error) => error_state.set(Some(error)),
            }
        });
    };
    use_hook(refresh);

    // Runs a mutation, then reloads the lists (or shows the error).
    let run = move |fut: std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AuthError>>>>| {
        spawn(async move {
            match fut.await {
                Ok(()) => refresh(),
                Err(error) => error_state.set(Some(error)),
            }
        });
    };

    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col items-center h-full space-y-2 overflow-y-auto",
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            div { class: "w-11/12 flex flex-row space-x-2",
                Input {
                    value: new_list(),
                    placeholder: "New list title",
                    oninput: move |ev: FormEvent| new_list.set(ev.value()),
                }
                Button {
                    label: "Add list",
                    onclick: move |_| {
                        let title = new_list().trim().to_string();
                        if !title.is_empty() {
                            new_list.set(String::new());
                            run(Box::pin(services::todo::create_list(title)));
                        }
                    },
                }
            }
            for list in lists.read().iter().cloned() {
                TodoListCard { key: "{list.id}", list, run }
            }
        }
    }
}

type Job = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AuthError>>>>;

#[component]
fn TodoListCard(list: TodoListDto, run: Callback<Job>) -> Element {
    let mut new_item = use_signal(String::new);
    let list_id = list.id;
    let title = list.title.clone().unwrap_or_default();

    rsx! {
        div { class: "w-11/12 flex flex-col rounded-md text-lg border-slate-300 dark:border-slate-700 border
                px-4 py-2 space-y-2 bg-slate-100 text-slate-800 shadow-md dark:bg-slate-900 dark:text-slate-100",
            div { class: "flex flex-row justify-between items-center",
                span { class: "font-bold", "{title}" }
                Button {
                    label: "Delete list",
                    onclick: move |_| run.call(Box::pin(services::todo::delete_list(list_id))),
                }
            }
            for item in list.items.iter().cloned() {
                div { key: "{item.id}", class: "flex flex-row items-center space-x-2",
                    input {
                        r#type: "checkbox",
                        checked: item.done,
                        onchange: {
                            let item = item.clone();
                            move |ev: FormEvent| {
                                let title = item.title.clone().unwrap_or_default();
                                run.call(Box::pin(services::todo::update_item(item.id, title, ev.checked())));
                            }
                        },
                    }
                    span { class: if item.done { "grow line-through opacity-60" } else { "grow" },
                        "{item.title.clone().unwrap_or_default()}"
                    }
                    Button {
                        label: "Delete",
                        onclick: move |_| run.call(Box::pin(services::todo::delete_item(item.id))),
                    }
                }
            }
            div { class: "flex flex-row space-x-2",
                Input {
                    value: new_item(),
                    placeholder: "New item",
                    oninput: move |ev: FormEvent| new_item.set(ev.value()),
                }
                Button {
                    label: "Add",
                    onclick: move |_| {
                        let title = new_item().trim().to_string();
                        if !title.is_empty() {
                            new_item.set(String::new());
                            run.call(Box::pin(services::todo::create_item(list_id, title)));
                        }
                    },
                }
            }
        }
    }
}
