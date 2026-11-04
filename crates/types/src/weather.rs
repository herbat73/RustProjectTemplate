use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WeatherForecast {
    /// Forecast date, RFC 3339 (UTC).
    #[cfg_attr(feature = "openapi", schema(format = DateTime))]
    pub date: String,
    pub temperature_c: i32,
    pub temperature_f: i32,
    pub summary: String,
}

impl WeatherForecast {
    pub fn new(date: String, temperature_c: i32, summary: String) -> Self {
        Self {
            date,
            temperature_c,
            temperature_f: 32 + (temperature_c as f64 / 0.5556) as i32,
            summary,
        }
    }
}
