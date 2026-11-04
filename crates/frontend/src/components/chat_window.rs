use dioxus::prelude::*;
use tauri_sys::core::invoke_result;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CloseEvent, MessageEvent, WebSocket};

use crate::{
    components::{buttons::button::Button, input::Input},
    graphics::icons::send_icon::SendIcon,
    services::AuthStorage,
};

#[component]
pub fn ChatWindow(#[props(default)] class: String) -> Element {
    let mut chat_disabled = use_signal(|| true);
    let mut chat_message = use_signal(String::new);
    let mut history = use_signal(Vec::<String>::new);
    let mut ws = use_signal(|| None::<WebSocket>);

    // Get backend port automatically from tauri command.
    let port = use_resource(|| async {
        invoke_result::<String, String>("get_port", &())
            .await
            .map_err(|e| format!("Error: {:?}", e))
    });

    // Connect to the websocket once the port is known.
    use_effect(move || {
        let Some(result) = &*port.read() else { return };
        let mut port = result.clone().unwrap_or_default();
        if cfg!(debug_assertions) && port.is_empty() {
            port = "3001".to_string();
        }
        if ws.peek().is_some() {
            return;
        }
        let Ok(socket) = WebSocket::new(&format!("ws://localhost:{}/ws", port)) else {
            return;
        };

        let onopen = Closure::<dyn FnMut()>::new({
            let socket = socket.clone();
            move || {
                if let Ok(token) = AuthStorage::get_requester_token() {
                    let _ = socket.send_with_str(&token.access_token);
                    chat_disabled.set(false);
                } else {
                    let _ = socket.close();
                }
            }
        });
        socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Some(text) = event.data().as_string() {
                history.write().push(text);
            }
        });
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |_| chat_disabled.set(true));
        socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        ws.set(Some(socket));
    });

    // Close the socket and detach handlers (which hold signals) on unmount.
    use_drop(move || {
        if let Some(socket) = ws.peek().as_ref() {
            socket.set_onopen(None);
            socket.set_onmessage(None);
            socket.set_onclose(None);
            let _ = socket.close();
        }
    });

    let mut send_chat = move || {
        let message = chat_message.read().clone();
        if message.is_empty() {
            return;
        }
        if let Some(socket) = ws.peek().as_ref() {
            let _ = socket.send_with_str(&message);
        }
        chat_message.set(String::new());
    };

    rsx! {
        div { class: "{class}",
            div { class: "h-full px-4 py-2 py-2
            bg-slate-100 text-slate-800
            border-slate-300 dark:border-slate-700 border
            dark:bg-slate-900 dark:text-slate-100
            rounded-md ring-offset-background disabled:pointer-events-none
            overflow-y-auto text-wrap shadow-md",
                for message in history.read().iter() {
                    p { "{message}" }
                }
            }
            form {
                class: "flex flex-row h-12 w-full space-x-2",
                onsubmit: move |ev| {
                    ev.prevent_default();
                    send_chat();
                },
                Input {
                    input_type: "text",
                    placeholder: "Message...",
                    oninput: move |ev: FormEvent| chat_message.set(ev.value()),
                    value: chat_message.read().clone(),
                }
                Button {
                    onclick: move |_| send_chat(),
                    icon: rsx! {
                        SendIcon { class: "fill-slate-600 dark:fill-white" }
                    },
                    disabled: chat_disabled(),
                }
            }
        }
    }
}
