use axum::{response::Json, routing::get, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Serialize)]
struct TestResponse {
    message: String,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(handler))
        .route("/test", get(handler))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(false));

    let addr: SocketAddr = "127.0.0.1:8000".parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("🚀 Test server running on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handler() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Hello from test server".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}
