use dioxus::prelude::*;

use crate::components::card::{Card, CardContent};
use crate::graphics::ferris::Ferris;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "col-span-12 row-span-24 flex justify-center items-center",
            div { class: "mx-10",
                Card {
                    CardContent {
                        div { class: "flex flex-col md:flex-row justify-center items-center space-y-10 p-10",
                            div { class: "flex flex-col justify-center text-center space-y-2",
                                h1 { class: "text-5xl", "Welcome!" }
                                p {
                                    "This template is built using Dioxus, Tailwinds, Axum, Sqlx, and Tauri."
                                }
                                p {
                                    "This project is built to make full stack development with user authentication easily accessible for developers within an entirely Rust ecosystem."
                                }
                                p { "In here you'll find some quick demos to get you started!" }
                            }
                            Ferris { class: "max-h-64" }
                        }
                    }
                }
            }
        }
    }
}
