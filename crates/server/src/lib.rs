//! Presentation layer: the HTTP and WebSocket API. Handlers only translate requests into
//! calls of the `application` use cases and their results into responses.

use axum::Router;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::{Any, CorsLayer};

pub mod bootstrap;
pub mod controllers;
pub mod error;
pub mod extractors;
pub mod mapping;
pub mod openapi;
pub mod state;

pub use state::AppState;

/// Builds the application router with all routes and the CORS layer.
pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::permissive()
        .allow_origin(Any)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .expose_headers(Any);

    Router::new()
        .nest("/ws", controllers::ws_controller::routes())
        .nest("/auth", controllers::auth_controller::routes())
        .nest("/user", controllers::users_controller::routes())
        .nest(
            "/api/WeatherForecasts",
            controllers::weather_controller::routes(),
        )
        .nest(
            "/api/TodoItems",
            controllers::todo_items_controller::routes(),
        )
        .nest(
            "/api/TodoLists",
            controllers::todo_lists_controller::routes(),
        )
        .merge(openapi::routes())
        .layer(cors)
        .with_state(state)
}
