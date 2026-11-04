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
            temperature_f: (f64::from(temperature_c) * 9.0 / 5.0).round() as i32 + 32,
            summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WeatherForecast;

    #[test]
    fn computes_fahrenheit_from_celsius() {
        assert_eq!(WeatherForecast::new("d".into(), 0, "Mild".into()).temperature_f, 32);
        assert_eq!(WeatherForecast::new("d".into(), 100, "Hot".into()).temperature_f, 212);
        assert_eq!(WeatherForecast::new("d".into(), -20, "Freezing".into()).temperature_f, -4);
        assert_eq!(WeatherForecast::new("d".into(), 21, "Warm".into()).temperature_f, 69);
    }

    /// The weather page deserializes the API response into `Vec<WeatherForecast>`,
    /// so the camelCase wire format must keep round-tripping.
    #[test]
    fn deserializes_api_response_used_by_weather_page() {
        let body = r#"[{"date":"2026-11-05T00:00:00Z","temperatureC":21,"temperatureF":69,"summary":"Warm"}]"#;
        let forecasts: Vec<WeatherForecast> = serde_json::from_str(body).unwrap();
        assert_eq!(forecasts.len(), 1);
        assert_eq!(forecasts[0].temperature_c, 21);
        assert_eq!(forecasts[0].summary, "Warm");

        let json = serde_json::to_value(&forecasts[0]).unwrap();
        assert!(json.get("temperatureC").is_some() && json.get("temperatureF").is_some());
    }
}
