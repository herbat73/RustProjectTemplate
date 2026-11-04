//! End-to-end API tests. They drive the real router in-process against a Postgres database
//! (`TEST_DATABASE_URL`, falling back to `DATABASE_URL` from the environment or `.env`).
//! Start one with `docker compose up -d` and run `scripts/migrate.sh postgres`.
//! If no database URL is configured the test is skipped. Test users get unique names and are
//! deleted at the end.

use axum::{
    body::{to_bytes, Body},
    http::{header::AUTHORIZATION, Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

struct Response {
    status: StatusCode,
    authorization: Option<String>,
    body: String,
}

impl Response {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|e| panic!("invalid JSON body {:?}: {e}", self.body))
    }
}

async fn send(app: &Router, method: Method, uri: &str, bearer: Option<&str>, body: Body, json_body: bool) -> Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    if json_body {
        builder = builder.header("content-type", "application/json");
    }
    let response = app.clone().oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let authorization = response
        .headers()
        .get(AUTHORIZATION)
        .map(|v| v.to_str().unwrap().trim_start_matches("Bearer ").to_string());
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Response { status, authorization, body: String::from_utf8_lossy(&bytes).into_owned() }
}

async fn post_json(app: &Router, uri: &str, value: Value) -> Response {
    send(app, Method::POST, uri, None, Body::from(value.to_string()), true).await
}

async fn get(app: &Router, uri: &str, bearer: Option<&str>) -> Response {
    send(app, Method::GET, uri, bearer, Body::empty(), false).await
}

async fn setup() -> Option<Router> {
    let _ = dotenv::dotenv();
    let Some(database_url) = std::env::var("TEST_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL")).ok() else {
        eprintln!("skipping API tests: set TEST_DATABASE_URL or DATABASE_URL");
        return None;
    };
    // SAFETY: set before any other thread reads the environment (single test in this binary).
    unsafe {
        std::env::set_var("DATABASE_URL", database_url);
        std::env::set_var("PASSWORD_SALT", "0123456789abcdef");
        std::env::set_var("AUTH_TOKEN_SECRET", "test-secret-test-secret-test-secret");
        std::env::set_var("AUTH_TOKEN_EXPIRE", "60");
        std::env::set_var("AUTH_REQUEST_TOKEN_EXPIRE", "3600");
        std::env::set_var("COMPANY_NAME", "TestCo");
        std::env::set_var("COMPANY_DOMAIN", "test.example.com");
    }
    server::pool::create_pool().await;
    sqlx::migrate!("../../migrations/postgres")
        .run(&server::pool::get_pool())
        .await
        .expect("migrations should apply");
    Some(server::app())
}

/// Decodes the (unverified) payload of a JWT.
fn jwt_payload(token: &str) -> Value {
    use base64::prelude::*;
    let payload = token.split('.').nth(1).expect("JWT payload segment");
    serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(payload).expect("base64url payload")).expect("JSON payload")
}

/// Signs arbitrary claims with the test secret, to forge tokens with bad claims.
fn forge(claims: Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"test-secret-test-secret-test-secret"),
    )
    .unwrap()
}

/// Exchanges a requester token for a short-lived access token.
async fn access_token(app: &Router, requester: &str) -> String {
    let response = get(app, "/auth/request", Some(requester)).await;
    assert_eq!(response.status, StatusCode::CREATED, "body: {}", response.body);
    response.authorization.clone().expect("access token header")
}

// One test function: the pool is a process-wide singleton bound to this test's runtime.
#[tokio::test]
async fn api_flow() {
    let Some(app) = setup().await else { return };
    let suffix = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let alice = format!("alice_{suffix}");
    let bob = format!("bob_{suffix}");
    let alice_email = format!("{alice}@example.com");
    let bob_email = format!("{bob}@example.com");

    // --- register ---
    let register = json!({"username": alice, "pass": "pw12345!", "email": alice_email});
    let response = post_json(&app, "/auth/register", register.clone()).await;
    assert_eq!(response.status, StatusCode::CREATED, "body: {}", response.body);
    assert!(response.authorization.is_some(), "register returns a requester token");
    let info = response.json();
    assert_eq!(info["username"], alice.as_str());
    assert_eq!(info["email"], alice_email.as_str());
    assert_eq!(info["is_admin"], false);
    assert!(!info["uuid"].as_str().unwrap().is_empty());

    // registering the same user again is rejected
    let response = post_json(&app, "/auth/register", register).await;
    assert!(response.status.is_client_error(), "duplicate register: {} {}", response.status, response.body);

    // --- login ---
    let response = post_json(&app, "/auth/login", json!({"username": alice, "pass": "wrong"})).await;
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.json()["error_type"], "WrongCredentials");

    let response = post_json(&app, "/auth/login", json!({"username": "nobody", "pass": "pw12345!"})).await;
    assert!(response.status.is_client_error(), "unknown user: {}", response.status);

    let response = post_json(&app, "/auth/login", json!({"username": alice, "pass": "pw12345!"})).await;
    assert_eq!(response.status, StatusCode::CREATED, "body: {}", response.body);
    let requester = response.authorization.clone().expect("requester token");
    assert_eq!(response.json()["username"], alice.as_str());

    // --- token-protected routes ---
    let access = access_token(&app, &requester).await;

    let response = get(&app, "/auth/test", Some(&access)).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, "Auth verified");

    let response = get(&app, "/user/info", Some(&access)).await;
    assert_eq!(response.status, StatusCode::OK, "body: {}", response.body);
    assert_eq!(response.json()["username"], alice.as_str());

    // both tokens carry the issuer claim, matching the company name
    for token in [&requester, &access] {
        let claims = jwt_payload(token);
        assert_eq!(claims["iss"], "TestCo");
        assert_eq!(claims["com"], "TestCo");
        assert_eq!(claims["aud"], "test.example.com");
    }

    // forged tokens with a wrong / missing issuer or wrong audience are rejected
    let exp = jsonwebtoken::get_current_timestamp() + 60;
    let sub = jwt_payload(&access)["sub"].clone();
    let base = json!({"aud": "test.example.com", "iss": "TestCo", "com": "TestCo", "sub": sub, "exp": exp, "acc": false});
    assert_eq!(get(&app, "/auth/test", Some(&forge(base.clone()))).await.status, StatusCode::OK, "sanity: forged valid token works");
    let mut bad_iss = base.clone();
    bad_iss["iss"] = json!("OtherCo");
    assert_eq!(get(&app, "/auth/test", Some(&forge(bad_iss))).await.status, StatusCode::FORBIDDEN);
    let mut no_iss = base.clone();
    no_iss.as_object_mut().unwrap().remove("iss");
    assert_eq!(get(&app, "/auth/test", Some(&forge(no_iss))).await.status, StatusCode::FORBIDDEN);
    let mut bad_aud = base.clone();
    bad_aud["aud"] = json!("evil.example.com");
    assert_eq!(get(&app, "/auth/test", Some(&forge(bad_aud))).await.status, StatusCode::FORBIDDEN);
    let mut expired = base;
    expired["exp"] = json!(1_000_000);
    assert_eq!(get(&app, "/auth/test", Some(&forge(expired))).await.status, StatusCode::FORBIDDEN);

    // each token type is only accepted where it belongs
    assert_eq!(get(&app, "/auth/request", Some(&access)).await.status, StatusCode::FORBIDDEN, "access token must not work as a requester token");
    assert_eq!(get(&app, "/auth/test", Some(&requester)).await.status, StatusCode::FORBIDDEN, "requester token must not work as an access token");
    assert_eq!(get(&app, "/user/info", Some(&requester)).await.status, StatusCode::FORBIDDEN);

    // missing / garbage tokens
    let response = get(&app, "/user/info", None).await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);
    assert_eq!(response.json()["error_type"], "InvalidToken");
    assert_eq!(get(&app, "/user/info", Some("garbage")).await.status, StatusCode::FORBIDDEN);

    // --- weather forecasts (random data, token-protected) ---
    let response = get(&app, "/api/WeatherForecasts", None).await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);
    assert_eq!(get(&app, "/api/WeatherForecasts", Some(&requester)).await.status, StatusCode::FORBIDDEN);

    let response = get(&app, "/api/WeatherForecasts", Some(&access)).await;
    assert_eq!(response.status, StatusCode::OK, "body: {}", response.body);
    let forecasts = response.json();
    let forecasts = forecasts.as_array().expect("array of forecasts");
    assert_eq!(forecasts.len(), 5);
    for forecast in forecasts {
        let date = forecast["date"].as_str().expect("date");
        assert!(date.len() == 20 && date.ends_with('Z') && date.as_bytes()[10] == b'T', "date: {date}");
        let c = forecast["temperatureC"].as_i64().expect("temperatureC");
        assert!((-20..55).contains(&c), "temperatureC: {c}");
        let f = forecast["temperatureF"].as_i64().expect("temperatureF");
        assert_eq!(f, 32 + (c as f64 / 0.5556) as i64, "temperatureF consistent with temperatureC");
        assert!(!forecast["summary"].as_str().expect("summary").is_empty());
    }

    // --- admin-only routes ---
    let response = get(&app, "/user/all", Some(&access)).await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);
    assert_eq!(response.json()["error_type"], "AccessDenied");

    let response = send(&app, Method::DELETE, "/user", Some(&access), Body::from("some-uuid"), false).await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);

    // promote alice to admin directly in the database, then get a fresh access token
    sqlx::query("UPDATE users SET is_admin = true WHERE username = $1")
        .bind(&alice)
        .execute(&server::pool::get_pool())
        .await
        .expect("promote alice");
    let admin_access = access_token(&app, &requester).await;

    // a second user to delete
    let response = post_json(&app, "/auth/register", json!({"username": bob, "pass": "pw12345!", "email": bob_email})).await;
    assert_eq!(response.status, StatusCode::CREATED);
    let bob_uuid = response.json()["uuid"].as_str().unwrap().to_string();

    let response = get(&app, "/user/all", Some(&admin_access)).await;
    assert_eq!(response.status, StatusCode::OK, "body: {}", response.body);
    let users = response.json();
    let names: Vec<&str> = users.as_array().unwrap().iter().map(|u| u["username"].as_str().unwrap()).collect();
    assert!(names.contains(&alice.as_str()) && names.contains(&bob.as_str()), "users: {names:?}");

    let response = send(&app, Method::DELETE, "/user", Some(&admin_access), Body::from(bob_uuid), false).await;
    assert!(response.status.is_success(), "delete: {} {}", response.status, response.body);

    let users = get(&app, "/user/all", Some(&admin_access)).await.json();
    let names: Vec<&str> = users.as_array().unwrap().iter().map(|u| u["username"].as_str().unwrap()).collect();
    assert!(!names.contains(&bob.as_str()), "bob should be deleted: {names:?}");

    // --- password reset ---
    let response = send(&app, Method::POST, "/auth/reset", None, Body::from("nobody@example.com"), false).await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.json()["error_type"], "UserDoesNotExist");

    // --- cleanup ---
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&alice)
        .execute(&server::pool::get_pool())
        .await
        .expect("cleanup alice");
}
