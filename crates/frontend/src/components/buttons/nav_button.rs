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
            div {
                class: "flex items-center justify-center h-full
                    text-md font-medium ring-offset-background
                    transition-colors focus-visible:outline-none
                    focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
                    disabled:pointer-events-none disabled:opacity-50 px-4 py-2
                    bg-slate-100 text-slate-800 hover:bg-slate-200
                    dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800 {disabled_class}",
                "{label}"
            }
        }
    }
}
