//! OpenAPI 3.2 document for the server API, generated from the `#[utoipa::path]` annotations
//! on the handlers and the `ToSchema` derives on the shared types.

use axum::{Json, Router, response::Html, routing::get};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        OpenApiVersion,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};

use crate::controllers::{
    auth_controller, todo_items_controller, todo_lists_controller, users_controller,
    weather_controller, ws_controller,
};

struct SecuritySchemes;

impl Modify for SecuritySchemes {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("components are registered");
        components.add_security_scheme(
            "access_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Short-lived access token, obtained from `GET /auth/request`.",
                    ))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "requester_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Long-lived requester token, returned in the `authorization` header by login and register."))
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "RustProjectTemplate API", description = "User authentication, user management and chat."),
    paths(
        auth_controller::login_user,
        auth_controller::register_user,
        auth_controller::request_auth_token,
        auth_controller::test_auth_route,
        auth_controller::request_reset,
        auth_controller::reset_password,
        users_controller::get_user_info,
        users_controller::get_all_user_info,
        users_controller::delete_user,
        weather_controller::get_weather_forecasts,
        todo_lists_controller::get_todo_lists,
        todo_lists_controller::create_todo_list,
        todo_lists_controller::update_todo_list,
        todo_lists_controller::delete_todo_list,
        todo_items_controller::create_todo_item,
        todo_items_controller::update_todo_item,
        todo_items_controller::update_todo_item_detail,
        todo_items_controller::delete_todo_item,
        ws_controller::ws_docs,
    ),
    components(schemas(
        types::user::LoginUser,
        types::user::RegisterUser,
        types::user::ResetUser,
        types::user::UserInfo,
        types::auth::AuthErrorBody,
        types::auth::AuthErrorType,
        types::weather::WeatherForecast,
        types::todo::TodosVm,
        types::todo::TodoListDto,
        types::todo::TodoItemDto,
        types::todo::LookupDto,
        types::todo::ColourDto,
        types::todo::CreateTodoListCommand,
        types::todo::UpdateTodoListCommand,
        types::todo::CreateTodoItemCommand,
        types::todo::UpdateTodoItemCommand,
        types::todo::UpdateTodoItemDetailCommand,
    )),
    modifiers(&SecuritySchemes),
    tags(
        (name = "auth", description = "Authentication and password reset"),
        (name = "user", description = "User management"),
        (name = "chat", description = "Chat WebSocket"),
        (name = "WeatherForecasts", description = "Random weather forecasts"),
        (name = "TodoLists", description = "Todo lists"),
        (name = "TodoItems", description = "Todo items"),
    )
)]
pub struct ApiDoc;

/// Returns the generated OpenAPI document.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    // utoipa defaults to 3.1; opt in to 3.2.
    doc.openapi = OpenApiVersion::Version32;
    doc
}

const SWAGGER_UI: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>API documentation</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.34.1/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5.34.1/swagger-ui-bundle.js"></script>
  <script>
    window.ui = SwaggerUIBundle({ url: "/api-docs/openapi.json", dom_id: "#swagger-ui" });
  </script>
</body>
</html>"##;

/// Routes serving the OpenAPI document (`/api-docs/openapi.json`) and a Swagger UI page (`/api-docs`).
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api-docs/openapi.json", get(|| async { Json(openapi()) }))
        .route("/api-docs", get(|| async { Html(SWAGGER_UI) }))
        .route("/api-docs/", get(|| async { Html(SWAGGER_UI) }))
}
