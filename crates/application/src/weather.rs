use std::sync::Arc;

use domain::weather::WeatherForecast;

use crate::{error::AppResult, ports::ForecastProvider};

/// How many days ahead the forecast reaches.
pub const FORECAST_DAYS: u32 = 5;

pub struct WeatherService {
    provider: Arc<dyn ForecastProvider>,
}

impl WeatherService {
    pub fn new(provider: Arc<dyn ForecastProvider>) -> Self {
        Self { provider }
    }

    pub async fn forecasts(&self) -> AppResult<Vec<WeatherForecast>> {
        self.provider.next_days(FORECAST_DAYS).await
    }
}
