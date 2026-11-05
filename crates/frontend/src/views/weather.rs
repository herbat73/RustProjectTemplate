use dioxus::prelude::*;
use types::weather::WeatherForecast;

use crate::components::{
    button::Button,
    card::{Card, CardContent},
    error_message::ErrorMessage,
};
use crate::services::{self, AuthError};

#[component]
pub fn Weather() -> Element {
    let mut forecasts = use_signal(Vec::<WeatherForecast>::new);
    let mut error_state = use_signal(|| None::<AuthError>);

    let refresh = move || {
        spawn(async move {
            match services::weather::get_weather_forecasts().await {
                Ok(data) => {
                    error_state.set(None);
                    forecasts.set(data);
                }
                Err(error) => error_state.set(Some(error)),
            }
        });
    };

    // Fetch fresh forecasts every time the page is opened
    use_hook(refresh);

    rsx! {
        div { class: "col-span-12 row-span-24 flex flex-col justify-center items-center h-full space-y-2",
            if let Some(error) = error_state.read().clone() {
                ErrorMessage { message: error.body().message }
            }
            div { class: "w-11/12 overflow-y-auto",
                Card {
                    CardContent {
                        table {
                            thead {
                                tr { class: "text-left",
                                    th { "Date" }
                                    th { "Temp. (C)" }
                                    th { "Temp. (F)" }
                                    th { "Summary" }
                                }
                            }
                            tbody {
                                for forecast in forecasts.read().iter().cloned() {
                                    tr { key: "{forecast.date}",
                                        td { "{forecast.date}" }
                                        td { "{forecast.temperature_c}" }
                                        td { "{forecast.temperature_f}" }
                                        td { "{forecast.summary}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Button { onclick: move |_| refresh(), "Refresh" }
        }
    }
}
