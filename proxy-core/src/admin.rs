use crate::Result;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tracing::info;

/// Shared state for metrics
#[derive(Debug, Default)]
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub active_connections: AtomicU64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    total_requests: u64,
    active_connections: u64,
}

pub async fn start_admin_server(port: u16, metrics: Arc<Metrics>) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(move || metrics_handler(metrics)));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting Admin API on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        crate::error::ProxyError::Network(format!("Failed to bind admin port {}: {}", port, e))
    })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::ProxyError::Network(format!("Admin server failed: {}", e)))?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn metrics_handler(metrics: Arc<Metrics>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        total_requests: metrics.total_requests.load(Ordering::Relaxed),
        active_connections: metrics.active_connections.load(Ordering::Relaxed),
    })
}
