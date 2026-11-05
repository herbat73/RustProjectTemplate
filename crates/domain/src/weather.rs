#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeatherForecast {
    /// Forecast date as seconds since the unix epoch (UTC).
    pub date_unix_secs: u64,
    pub temperature_c: i32,
    pub summary: String,
}
