use dioxus::prelude::*;

const DEFAULT_COLOR: &str = "bg-slate-100 text-slate-800 hover:bg-slate-200
    dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800";

#[component]
pub fn Button(
    #[props(default)] label: String,
    #[props(default = VNode::empty())] icon: Element,
    onclick: EventHandler<MouseEvent>,
    #[props(default = false)] disabled: bool,
    #[props(default = DEFAULT_COLOR.to_string())] color: String,
) -> Element {
    rsx! {
        button {
            class: "inline-flex items-center justify-center
                    rounded-md text-sm font-medium ring-offset-background
                    border-slate-300 dark:border-slate-800 border
                    transition-colors focus-visible:outline-none
                    focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
                    disabled:pointer-events-none disabled:opacity-50 px-4 py-2 shadow-md {color}",
            onclick: move |ev| {
                ev.prevent_default();
                onclick.call(ev);
            },
            disabled,
            "{label}"
            {icon}
        }
    }
}
