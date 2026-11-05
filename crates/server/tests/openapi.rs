//! Tests for the generated OpenAPI document. No database needed.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[test]
fn spec_is_openapi_3_2_and_documents_every_route() {
    let spec = serde_json::to_value(server::openapi::openapi()).unwrap();
    assert!(
        spec["openapi"].as_str().unwrap().starts_with("3.2."),
        "version: {}",
        spec["openapi"]
    );

    let paths = spec["paths"].as_object().unwrap();
    for (path, method) in [
        ("/auth/login", "post"),
        ("/auth/register", "post"),
        ("/auth/request", "get"),
        ("/auth/test", "get"),
        ("/auth/reset", "post"),
        ("/auth/reset/{reset_key}", "post"),
        ("/user/info", "get"),
        ("/user/all", "get"),
        ("/user", "delete"),
        ("/ws", "get"),
        ("/api/WeatherForecasts", "get"),
        ("/api/TodoLists", "get"),
        ("/api/TodoLists", "post"),
        ("/api/TodoLists/{id}", "put"),
        ("/api/TodoLists/{id}", "delete"),
        ("/api/TodoItems", "post"),
        ("/api/TodoItems/{id}", "put"),
        ("/api/TodoItems/{id}", "delete"),
        ("/api/TodoItems/UpdateDetail/{id}", "patch"),
    ] {
        assert!(
            paths.get(path).is_some_and(|p| p.get(method).is_some()),
            "missing {method} {path}"
        );
    }

    let schemes = spec["components"]["securitySchemes"].as_object().unwrap();
    assert!(schemes.contains_key("access_token") && schemes.contains_key("requester_token"));
    // protected operations reference the matching scheme
    assert_eq!(
        spec["paths"]["/user/all"]["get"]["security"][0]["access_token"],
        serde_json::json!([])
    );
    assert_eq!(
        spec["paths"]["/auth/request"]["get"]["security"][0]["requester_token"],
        serde_json::json!([])
    );
}

#[tokio::test]
async fn spec_and_ui_are_served() {
    // The documentation routes need no application state (and thus no database).
    let app = server::openapi::routes::<()>();

    let response = app
        .clone()
        .oneshot(
            Request::get("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let served: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        served,
        serde_json::to_value(server::openapi::openapi()).unwrap()
    );

    let response = app
        .oneshot(Request::get("/api-docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn committed_spec_is_up_to_date() {
    let committed = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/openapi.json"
    ))
    .expect("docs/openapi.json exists");
    let committed: Value = serde_json::from_str(&committed).unwrap();
    assert_eq!(
        committed,
        serde_json::to_value(server::openapi::openapi()).unwrap(),
        "docs/openapi.json is stale; regenerate with `cargo run -q --bin openapi > docs/openapi.json`"
    );
}
