use dioxus::prelude::*;

use crate::components::auth::request_reset_form::RequestResetForm;

#[component]
pub fn RequestReset() -> Element {
    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col justify-center items-center h-full space-y-4",
            RequestResetForm {}
        }
    }
}
