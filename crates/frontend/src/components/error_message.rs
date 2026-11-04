use dioxus::prelude::*;

#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "px-4 py-2 rounded-md bg-red-300 text-red-600 border border-red-600 text-center shadow-md",
            "{message}"
        }
    }
}
