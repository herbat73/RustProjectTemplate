use std::net::SocketAddr;

use axum::{Router, http::Method};
use tower_http::cors::{Any, CorsLayer};

pub async fn app(port: u16) {
    let app = Router::new().layer(CorsLayer::new().allow_origin(Any).allow_methods(vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
    ]));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Backend is listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
