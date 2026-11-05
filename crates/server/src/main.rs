use std::{net::SocketAddr, path::PathBuf};

#[tokio::main]
async fn main() {
    // load environment variables with dotenv if debug
    if cfg!(debug_assertions) {
        match dotenv::dotenv() {
            Ok(path) => path,
            Err(error) => {
                println!("Cannot access .env file: {}", error);
                PathBuf::from("")
            }
        };
    }

    let state = server::bootstrap::state_from_env().await;
    let app = server::app(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    match axum::serve(listener, app).await {
        Ok(_) => {}
        Err(error) => panic!("Could not bind to {}: {}", addr, error),
    }
}
