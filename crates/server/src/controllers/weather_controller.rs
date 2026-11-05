use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    routing::get,
};
use types::{auth::AuthErrorBody, weather::WeatherForecast};

use crate::{
    error::ApiError, extractors::AccessUser, mapping::weather_forecast_dto, state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_weather_forecasts))
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
pub async fn get_weather_forecasts(
    State(state): State<AppState>,
    _user: AccessUser,
) -> Result<(StatusCode, Json<Vec<WeatherForecast>>), ApiError> {
    let forecasts = state.weather.forecasts().await?;
    Ok((
        StatusCode::OK,
        Json(forecasts.into_iter().map(weather_forecast_dto).collect()),
    ))
}
