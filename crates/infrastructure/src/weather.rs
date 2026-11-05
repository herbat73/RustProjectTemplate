use std::time::{SystemTime, UNIX_EPOCH};

use application::{error::AppResult, ports::ForecastProvider};
use async_trait::async_trait;
use domain::weather::WeatherForecast;
use rand::{RngExt, seq::IndexedRandom};

pub const SUMMARIES: [&str; 10] = [
    "Freezing",
    "Bracing",
    "Chilly",
    "Cool",
    "Mild",
    "Warm",
    "Balmy",
    "Hot",
    "Sweltering",
    "Scorching",
];

const SECS_PER_DAY: u64 = 86_400;

/// Invents random forecasts; stands in for a real weather service.
#[derive(Default)]
pub struct RandomForecastProvider;

#[async_trait]
impl ForecastProvider for RandomForecastProvider {
    async fn next_days(&self, days: u32) -> AppResult<Vec<WeatherForecast>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut rng = rand::rng();
        Ok((1..=u64::from(days))
            .map(|day| WeatherForecast {
                date_unix_secs: now + day * SECS_PER_DAY,
                temperature_c: rng.random_range(-20..55),
                summary: SUMMARIES
                    .choose(&mut rng)
                    .copied()
                    .unwrap_or("Mild")
                    .to_string(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_consecutive_valid_forecasts() {
        let forecasts = RandomForecastProvider.next_days(5).await.unwrap();
        assert_eq!(forecasts.len(), 5);
        for f in &forecasts {
            assert!((-20..55).contains(&f.temperature_c));
            assert!(SUMMARIES.contains(&f.summary.as_str()));
        }
        for pair in forecasts.windows(2) {
            assert_eq!(
                pair[1].date_unix_secs - pair[0].date_unix_secs,
                SECS_PER_DAY
            );
        }
    }
}
