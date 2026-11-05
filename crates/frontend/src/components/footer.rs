use dioxus::prelude::*;

use crate::app::AppRoute;
use crate::components::buttons::nav_button::NavButton;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "flex flex-row justify-self-end w-full bg-[var(--primary-color-3)] z-50
                bottom-0 left-0 right-0 fixed border-[var(--primary-color-6)] border-t
                justify-evenly justify-items-center items-center shadow-md h-12",
            NavButton {
                label: "WeatherForecasts",
                destination: AppRoute::WeatherPage {},
            }
            NavButton { label: "ToDoLists", destination: AppRoute::TodoPage {} }
        }
    }
}
