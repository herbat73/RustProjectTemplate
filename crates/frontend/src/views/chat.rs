use dioxus::prelude::*;

use crate::components::chat_window::ChatWindow;

#[component]
pub fn Chat() -> Element {
    rsx! {
        main { class: "col-span-12 row-span-24 h-full",
            ChatWindow { class: "flex shrink flex-col w-full h-full
            ring-offset-background disabled:pointer-events-none
            p-4 space-y-2 text-sm" }
        }
    }
}
