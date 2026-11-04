use dioxus::prelude::*;

use crate::components::auth::reset_form::ResetForm;

#[component]
pub fn Reset() -> Element {
    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col justify-center items-center h-full space-y-4",
            ResetForm {}
        }
    }
}
