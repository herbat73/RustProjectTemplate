use dioxus::prelude::*;

#[component]
pub fn ExternalButton(label: String, destination: String) -> Element {
    rsx! {
        a {
            class: "flex items-center justify-center h-full px-4 py-2 text-sm font-medium
                text-[var(--secondary-color-4)] hover:bg-[var(--primary-color-5)]
                hover:text-[var(--secondary-color-1)] transition-colors
                focus-visible:outline-none focus-visible:ring-2",
            href: "{destination}",
            "{label}"
        }
    }
}
