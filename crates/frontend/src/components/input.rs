use dioxus::prelude::*;

#[component]
pub fn Input(
    value: String,
    oninput: EventHandler<FormEvent>,
    #[props(default)] placeholder: String,
    #[props(default = "text".to_string())] input_type: String,
    #[props(default = false)] disabled: bool,
) -> Element {
    rsx! {
        input {
            class: "rounded-md text-sm font-medium ring-offset-background w-full
                        transition-colors focus-visible:outline-none
                        border-slate-300 dark:border-slate-800 border
                        focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
                        disabled:pointer-events-none disabled:opacity-50 px-4 py-2
                        bg-slate-100 text-slate-800 hover:bg-slate-200 shadow-md
                        dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800",
            oninput: move |ev| oninput.call(ev),
            r#type: "{input_type}",
            placeholder: "{placeholder}",
            disabled,
            value: "{value}",
        }
    }
}
