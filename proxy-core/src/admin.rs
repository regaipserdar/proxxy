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
    // Body capture performance metrics
    pub body_capture_attempts: AtomicU64,
    pub body_capture_successes: AtomicU64,
    pub body_capture_failures: AtomicU64,
    pub body_capture_timeouts: AtomicU64,
    pub body_capture_memory_errors: AtomicU64,
    pub body_capture_total_latency_ms: AtomicU64,
    pub body_capture_total_bytes: AtomicU64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    total_requests: u64,
    active_connections: u64,
    // Body capture performance metrics
    body_capture: BodyCaptureMetrics,
}

#[derive(Serialize)]
struct BodyCaptureMetrics {
    attempts: u64,
    successes: u64,
    failures: u64,
    timeouts: u64,
    memory_errors: u64,
    success_rate: f64,
    average_latency_ms: f64,
    total_bytes_captured: u64,
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
    let attempts = metrics.body_capture_attempts.load(Ordering::Relaxed);
    let successes = metrics.body_capture_successes.load(Ordering::Relaxed);
    let failures = metrics.body_capture_failures.load(Ordering::Relaxed);
    let timeouts = metrics.body_capture_timeouts.load(Ordering::Relaxed);
    let memory_errors = metrics.body_capture_memory_errors.load(Ordering::Relaxed);
    let total_latency_ms = metrics.body_capture_total_latency_ms.load(Ordering::Relaxed);
    let total_bytes = metrics.body_capture_total_bytes.load(Ordering::Relaxed);
    
    let success_rate = if attempts > 0 {
        (successes as f64 / attempts as f64) * 100.0
    } else {
        0.0
    };
    
    let average_latency_ms = if successes > 0 {
        total_latency_ms as f64 / successes as f64
    } else {
        0.0
    };
    
    Json(MetricsResponse {
        total_requests: metrics.total_requests.load(Ordering::Relaxed),
        active_connections: metrics.active_connections.load(Ordering::Relaxed),
        body_capture: BodyCaptureMetrics {
            attempts,
            successes,
            failures,
            timeouts,
            memory_errors,
            success_rate,
            average_latency_ms,
            total_bytes_captured: total_bytes,
        },
    })
}
