use axum::{routing::get, Router, Json, extract::State};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use serde::Serialize;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod pb {
    tonic::include_proto!("proxy");
}

// Re-export for compatibility with UI
pub type OrchestratorService = Orchestrator;

pub mod session_manager;
pub mod server;
pub mod database;
pub mod graphql;
pub use session_manager::AgentRegistry;
pub use database::Database;

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub grpc_port: u16,
    pub http_port: u16,
    pub database_url: String,
    pub health_check_interval: u64,
    pub agent_timeout: u64,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Default)]
pub struct LoggingConfig {
    pub level: String,
}

pub struct Orchestrator {
    config: OrchestratorConfig,
}

use crate::graphql::{ProxySchema, QueryRoot, MutationRoot, SubscriptionRoot};

#[derive(Clone)]
struct AppState {
    schema: ProxySchema,
    agents: Arc<AgentRegistry>,
    db: Arc<Database>,
    start_time: std::time::Instant,
}

/// OpenAPI documentation for Proxxy Orchestrator REST API
#[derive(OpenApi)]
#[openapi(
    paths(
        health_detailed_handler,
        agents_handler,
        metrics_handler,
        traffic_handler,
    ),
    components(
        schemas(HealthStatus, AgentsResponse, AgentInfo, MetricsResponse, TrafficResponse, HttpTransaction)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "agents", description = "Agent management endpoints"),
        (name = "metrics", description = "Traffic metrics endpoints"),
        (name = "traffic", description = "HTTP traffic data endpoints")
    ),
    info(
        title = "Proxxy Orchestrator API",
        version = "0.1.0",
        description = "REST API for Proxxy distributed MITM proxy orchestrator",
        contact(
            name = "Proxxy Team",
            url = "https://github.com/proxxy"
        )
    )
)]
struct ApiDoc;

#[derive(Serialize, utoipa::ToSchema)]
struct HealthStatus {
    status: String,
    uptime_seconds: u64,
    database_connected: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
struct AgentInfo {
    id: String,
    address: String,
    port: u16,
    status: String,
    last_heartbeat: String,
    version: String,
    capabilities: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct AgentsResponse {
    agents: Vec<AgentInfo>,
    total_count: usize,
    online_count: usize,
    offline_count: usize,
}

#[derive(Serialize, utoipa::ToSchema)]
struct MetricsResponse {
    total_requests: usize,
    average_response_time_ms: f64,
    error_rate: f64,
}

impl Orchestrator {
    pub async fn new(config: OrchestratorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { config })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Database
        let db = std::sync::Arc::new(crate::Database::new(&self.config.database_url).await?);

        // Initialize CA (load from ./certs or generate)
        let ca_path = std::path::Path::new("certs");
        let ca = std::sync::Arc::new(proxy_core::CertificateAuthority::new(ca_path)?);

        let agent_registry = std::sync::Arc::new(AgentRegistry::new());
        // Create broadcast channel for traffic events (capacity 100)
        let (broadcast_tx, _broadcast_rx) = tokio::sync::broadcast::channel(100);
        // Create broadcast channel for system metrics events (capacity 100)
        let (metrics_broadcast_tx, _metrics_broadcast_rx) = tokio::sync::broadcast::channel(100);
        
        let proxy_service = crate::server::ProxyServiceImpl::new(
            agent_registry.clone(),
            broadcast_tx.clone(),
            metrics_broadcast_tx.clone(),
            db.clone(),
            ca.clone(),
        );
        
        // GraphQL Schema
        let schema = async_graphql::Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
            .data(db.clone())
            .data(agent_registry.clone())
            .data(broadcast_tx.clone())
            .data(metrics_broadcast_tx.clone())
            .finish();

        let state = AppState {
            schema,
            agents: agent_registry.clone(),
            db: db.clone(),
            start_time: std::time::Instant::now(),
        };

        // 1. Metrics & GraphQL Server & REST API
        let metrics_addr = SocketAddr::from(([0, 0, 0, 0], self.config.http_port));
        let app = axum::Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health/detailed", get(health_detailed_handler))
            .route("/agents", get(agents_handler))
            .route("/traffic", get(traffic_handler))
            .merge(SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()))
            .route("/", get(|| async { axum::response::Redirect::permanent("/swagger-ui") }))
            .route("/graphql", get(graphiql).post(graphql_handler))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        info!("REST & GraphQL server listening on http://{}", metrics_addr);
        let metrics_server = async move {
            let listener = TcpListener::bind(metrics_addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        };

        // 2. gRPC Server
        let grpc_addr = SocketAddr::from(([0, 0, 0, 0], self.config.grpc_port));
        info!("Orchestrator gRPC listening on {}", grpc_addr);
        
        let grpc_server = tonic::transport::Server::builder()
            .add_service(crate::pb::proxy_service_server::ProxyServiceServer::new(proxy_service))
            .serve(grpc_addr);

        // Run both
        tokio::select! {
            _ = metrics_server => {},
            res = grpc_server => {
                if let Err(e) = res {
                    tracing::error!("gRPC server failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

async fn graphiql() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<async_graphql::Request>,
) -> axum::Json<async_graphql::Response> {
    axum::Json(state.schema.execute(req).await)
}

#[derive(Serialize, utoipa::ToSchema)]
struct TrafficResponse {
    transactions: Vec<HttpTransaction>,
    total_count: usize,
}

#[derive(Serialize, utoipa::ToSchema)]
struct HttpTransaction {
    request_id: String,
    agent_id: String,
    method: String,
    url: String,
    status: Option<i32>,
    timestamp: i64,
}

/// Get detailed health status
#[utoipa::path(
    get,
    path = "/health/detailed",
    tag = "health",
    responses(
        (status = 200, description = "Health status retrieved successfully", body = HealthStatus)
    )
)]
async fn health_detailed_handler(State(state): State<AppState>) -> Json<HealthStatus> {
    info!("📊 Health check requested");
    Json(HealthStatus {
        status: "Healthy".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        database_connected: true, // Simplified check
    })
}

/// List all registered agents
#[utoipa::path(
    get,
    path = "/agents",
    tag = "agents",
    responses(
        (status = 200, description = "Agents list retrieved successfully", body = AgentsResponse)
    )
)]
async fn agents_handler(State(state): State<AppState>) -> Json<AgentsResponse> {
    let agents_data = state.agents.list_agents();
    let total_count = agents_data.len();
    let online_count = agents_data.iter().filter(|a| a.status == "Online").count();
    let offline_count = total_count - online_count;

    info!("👥 Agents list requested - Total: {}, Online: {}, Offline: {}", 
        total_count, online_count, offline_count);

    // Convert to AgentInfo
    let agents = agents_data.into_iter().map(|a| AgentInfo {
        id: a.id,
        address: a.address,
        port: a.port,
        status: a.status,
        last_heartbeat: a.last_heartbeat,
        version: a.version,
        capabilities: a.capabilities,
    }).collect();

    Json(AgentsResponse {
        agents,
        total_count,
        online_count,
        offline_count
    })
}

/// Get recent HTTP traffic transactions
#[utoipa::path(
    get,
    path = "/traffic",
    tag = "traffic",
    responses(
        (status = 200, description = "Traffic data retrieved successfully", body = TrafficResponse)
    )
)]
async fn traffic_handler(State(state): State<AppState>) -> Json<TrafficResponse> {
    info!("🚦 Traffic data requested");
    
    // Fetch recent transactions from database
    let transactions = match sqlx::query_as::<_, (String, String, String, String, Option<i32>, i64)>(
        "SELECT request_id, agent_id, req_method, req_url, res_status, req_timestamp 
         FROM http_transactions 
         ORDER BY req_timestamp DESC 
         LIMIT 50"
    )
    .fetch_all(state.db.pool())
    .await
    {
        Ok(rows) => {
            info!("   ✓ Fetched {} transactions from database", rows.len());
            rows.into_iter().map(|(request_id, agent_id, method, url, status, timestamp)| {
                HttpTransaction {
                    request_id,
                    agent_id,
                    method,
                    url,
                    status,
                    timestamp,
                }
            }).collect()
        },
        Err(e) => {
            tracing::error!("   ✗ Failed to fetch traffic: {}", e);
            Vec::new()
        }
    };

    let total_count = transactions.len();

    Json(TrafficResponse {
        transactions,
        total_count,
    })
}

/// Get traffic metrics and statistics
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Metrics retrieved successfully", body = MetricsResponse)
    )
)]
async fn metrics_handler(State(state): State<AppState>) -> Json<MetricsResponse> {
    info!("📈 Metrics requested - querying database...");
    
    // Fetch real metrics from database
    let total_requests = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM http_transactions")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0) as usize;

    // Calculate average response time (duration_ms)
    let avg_response_time = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(duration_ms) FROM http_transactions WHERE duration_ms IS NOT NULL"
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(None)
    .unwrap_or(0.0);

    // Calculate error rate (4xx and 5xx status codes)
    let error_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM http_transactions WHERE res_status >= 400"
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let error_rate = if total_requests > 0 {
        (error_count as f64) / (total_requests as f64)
    } else {
        0.0
    };

    info!("   ✓ Total: {}, Avg Latency: {:.1}ms, Errors: {}/{} ({:.1}%)", 
        total_requests, avg_response_time, error_count, total_requests, error_rate * 100.0);

    Json(MetricsResponse {
        total_requests,
        average_response_time_ms: avg_response_time,
        error_rate,
    })
}

pub async fn run_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/metrics", get(|| async { "orchestrator_metrics{status=\"up\"} 1" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Orchestrator metrics listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}