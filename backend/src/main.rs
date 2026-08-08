use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

async fn hello_world() -> &'static str {
    "Ecology works! This is coming from Rust Backend!"
}

#[tokio::main]
async fn main() {
    let port = std::env::var("SERVER_PORT")
        .expect("SERVER_PORT is required; run eco configure so Eco can assign this service port");
    let app = Router::new()
        .route("/helloworld", get(hello_world))
        .route("/api/helloworld", get(hello_world))
        .layer(CorsLayer::permissive());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Eco starter backend could not bind its port");
    println!("Eco starter Rust backend listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.expect("Eco starter backend stopped unexpectedly");
}
