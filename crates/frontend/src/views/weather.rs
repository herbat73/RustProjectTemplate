use dioxus::prelude::*;
use types::weather::WeatherForecast;

use crate::components::{buttons::button::Button, error_message::ErrorMessage};
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
            div { class: "w-11/12 flex flex-col rounded-md text-lg overflow-y-auto
                border-slate-300 dark:border-slate-700 border px-4 py-2
                bg-slate-100 text-slate-800 shadow-md
                dark:bg-slate-900 dark:text-slate-100",
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
            Button { label: "Refresh", onclick: move |_| refresh() }
        }
    }
}
