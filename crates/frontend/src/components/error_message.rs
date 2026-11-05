use dioxus::prelude::*;

use crate::components::badge::{Badge, BadgeVariant};

#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "flex justify-center", role: "alert",
            Badge { variant: BadgeVariant::Destructive, "{message}" }
        }
    }
}
