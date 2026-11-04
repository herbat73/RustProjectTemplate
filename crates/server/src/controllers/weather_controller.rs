use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::Json, http::StatusCode, middleware, routing::get, Router};
use rand::{seq::IndexedRandom, RngExt};
use types::{auth::AuthErrorBody, weather::WeatherForecast};

use crate::{
    middleware::token_authentication,
    strategies::authentication::AuthClaims,
};

const SUMMARIES: [&str; 10] = [
    "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching",
];

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_weather_forecasts))
        .layer(middleware::from_fn(token_authentication::authenticate_token::<AuthClaims>))
}

/// Formats a unix timestamp (seconds) as an RFC 3339 UTC string.
fn rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Retrieves a list of weather forecasts for the next few days.
#[utoipa::path(
    get,
    path = "/api/WeatherForecasts",
    tag = "WeatherForecasts",
    operation_id = "GetWeatherForecasts",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "OK", body = Vec<WeatherForecast>),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
    )
)]
pub async fn get_weather_forecasts() -> (StatusCode, Json<Vec<WeatherForecast>>) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut rng = rand::rng();
    let forecasts = (1..=5u64)
        .map(|i| {
            WeatherForecast::new(
                rfc3339(now + i * 86_400),
                rng.random_range(-20..55),
                SUMMARIES.choose(&mut rng).copied().unwrap_or("Mild").to_string(),
            )
        })
        .collect();
    (StatusCode::OK, Json(forecasts))
}

#[cfg(test)]
mod tests {
    use super::{get_weather_forecasts, rfc3339, SUMMARIES};
    use axum::http::StatusCode;

    #[test]
    fn formats_unix_time_as_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z"); // leap day
        assert_eq!(rfc3339(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn formats_end_of_day_and_year_boundaries() {
        assert_eq!(rfc3339(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(rfc3339(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(rfc3339(951_868_800), "2000-03-01T00:00:00Z"); // day after leap day
    }

    #[tokio::test]
    async fn handler_returns_five_consecutive_valid_forecasts() {
        let (status, axum::Json(forecasts)) = get_weather_forecasts().await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(forecasts.len(), 5);
        for f in &forecasts {
            assert!((-20..55).contains(&f.temperature_c));
            assert!(SUMMARIES.contains(&f.summary.as_str()));
            assert!(f.date.ends_with('Z') && f.date.len() == 20);
        }
        let dates: Vec<&str> = forecasts.iter().map(|f| f.date.as_str()).collect();
        let mut sorted = dates.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(dates, sorted, "dates strictly increasing");
    }
}
