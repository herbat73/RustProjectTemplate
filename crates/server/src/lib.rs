use axum::Router;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

pub mod controllers;
pub mod middleware;
pub mod openapi;
pub mod pool;
pub mod strategies;

/// Builds the application router with all routes and the CORS layer.
/// The database pool must be created (see [`pool::create_pool`]) before serving requests.
pub fn app() -> Router {
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
        .layer(ServiceBuilder::new().layer(cors))
}
