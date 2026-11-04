use types::weather::WeatherForecast;

use super::{get_http_client, get_base_url, AuthError, AuthRequest};
use types::auth::AuthErrorType;

/// Fetches a fresh set of weather forecasts from the server.
pub async fn get_weather_forecasts() -> Result<Vec<WeatherForecast>, AuthError> {
    let response = AuthRequest::new(
        get_http_client().get(get_base_url() + "/api/WeatherForecasts"),
    )
    .send()
    .await?;

    if !response.status().is_success() {
        return Err(AuthError::from_response(response).await);
    }

    response
        .json::<Vec<WeatherForecast>>()
        .await
        .map_err(|_| AuthError::from_error_type(AuthErrorType::BadRequest))
}
