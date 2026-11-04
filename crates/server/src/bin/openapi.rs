//! Prints the generated OpenAPI document to stdout:
//! `cargo run -q --bin openapi > docs/openapi.json`
fn main() {
    println!(
        "{}",
        server::openapi::openapi()
            .to_pretty_json()
            .expect("serialize OpenAPI document")
    );
}
