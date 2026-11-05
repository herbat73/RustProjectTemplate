use dioxus::prelude::*;

use crate::app::AppRoute;

#[component]
pub fn NavButton(
    label: String,
    destination: AppRoute,
    #[props(default = false)] disabled: bool,
) -> Element {
    let disabled_class = if disabled {
        "pointer-events-none opacity-50"
    } else {
        ""
    };

    rsx! {
        Link { to: destination,
            div { class: "flex items-center justify-center h-full px-4 py-2 text-sm font-medium
                    text-[var(--secondary-color-4)] hover:bg-[var(--primary-color-5)]
                    hover:text-[var(--secondary-color-1)] transition-colors
                    focus-visible:outline-none focus-visible:ring-2 {disabled_class}",
                "{label}"
            }
        }
    }
}
