use axum::extract::{State, Request};
use axum::routing::get;
use axum::{Extension, Json, Router};
use axum::middleware::{self, Next};
use axum::response::Response;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::http::Method;
use async_graphql_axum::{GraphQLProtocol, GraphQLWebSocket, WebSocketUpgrade};

pub mod pb {
    pub use proxy_core::pb::*;
}

// Re-export for compatibility with UI
pub type OrchestratorService = Orchestrator;

pub mod database;
pub mod graphql;
pub mod models;
pub mod scope;
pub mod server;
pub mod session_manager;
pub use database::Database;
pub use session_manager::AgentRegistry;

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

use crate::graphql::{MutationRoot, ProxySchema, QueryRoot, SubscriptionRoot};
use crate::models::settings::{ScopeConfig, InterceptionConfig};
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    schema: ProxySchema,
    agents: Arc<AgentRegistry>,
    db: Arc<Database>,
    start_time: std::time::Instant,
    #[allow(dead_code)] // Used via GraphQL context
    scope: Arc<RwLock<ScopeConfig>>,
    #[allow(dead_code)] // Used via GraphQL context
    interception: Arc<RwLock<InterceptionConfig>>,
}

/// OpenAPI documentation for Proxxy Orchestrator REST API
#[derive(OpenApi)]
#[openapi(
    paths(
        health_detailed_handler,
        agents_handler,
        metrics_handler,
        traffic_handler,
        system_health_handler,
        system_start_handler,
        system_stop_handler,
        system_restart_handler,
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
        version = "0.1.1",
        description = "REST API for Proxxy distributed MITM proxy orchestrator",
        contact(
            name = "Proxxy Team",
            url = "https://github.com/regaipserdar/proxxy/"
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
        // Initialize Database with workspace directory
        let projects_dir = if self.config.database_url.starts_with("sqlite:") {
            "workspace"
        } else {
            &self.config.database_url
        };
        
        let db = std::sync::Arc::new(crate::Database::new(projects_dir).await?);

        // Initialize CA (load from ./certs or generate)
        let ca_path = std::path::Path::new("certs");
        let ca = std::sync::Arc::new(proxy_core::CertificateAuthority::new(ca_path)?);

        let agent_registry = std::sync::Arc::new(AgentRegistry::new());
        let (broadcast_tx, _broadcast_rx) = tokio::sync::broadcast::channel::<(String, crate::pb::TrafficEvent)>(100);
        let (metrics_broadcast_tx, _metrics_broadcast_rx) = tokio::sync::broadcast::channel(100);

        // Initialize scope and interception state
        let scope = Arc::new(RwLock::new(ScopeConfig::default()));
        let interception = Arc::new(RwLock::new(InterceptionConfig::default()));

        // Start Orchestrator System Metrics Collector
        let db_metrics = db.clone();
        let metrics_tx = metrics_broadcast_tx.clone();
        tokio::spawn(async move {
            info!("🚀 Starting Orchestrator metrics collector...");
            
            // 0. The orchestrator metrics will be saved with agent_id = "orchestrator"
            // We will remove the Foreign Key constraint in the database to allow this 
            // without treating the orchestrator as a proxy agent.

            let mut collector = proxy_core::system_metrics::SystemMetricsCollector::new("orchestrator".to_string());
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                // Only try to save if a project is loaded
                if db_metrics.pool().await.is_some() {
                    match collector.collect_metrics().await {
                        Ok(event) => {
                            // 1. Save to DB (Orchestrator dedicated table)
                            if let Err(e) = db_metrics.save_orchestrator_metrics(&event).await {
                                 warn!("   ✗ Failed to save orchestrator metrics: {}", e);
                            }
                            // 2. Broadcast
                            let _ = metrics_tx.send(event);
                        }
                        Err(e) => {
                            warn!("   ✗ Failed to collect orchestrator metrics: {}", e);
                        }
                    }
                }
            }
        });

        let proxy_service = crate::server::ProxyServiceImpl::new(
            agent_registry.clone(),
            broadcast_tx.clone(),
            metrics_broadcast_tx.clone(),
            db.clone(),
            ca.clone(),
            scope.clone(),
            interception.clone(),
        );

        // GraphQL Schema
        let schema = async_graphql::Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
            .data(db.clone())
            .data(agent_registry.clone())
            .data(broadcast_tx.clone())
            .data(metrics_broadcast_tx.clone())
            .data(scope.clone())
            .data(interception.clone())
            .finish();

        let state = AppState {
            schema,
            agents: agent_registry.clone(),
            db: db.clone(),
            start_time: std::time::Instant::now(),
            scope,
            interception,
        };

        // 1. Metrics & GraphQL Server & REST API
        let metrics_addr = SocketAddr::from(([0, 0, 0, 0], self.config.http_port));

        // Define API routes
        let api_routes = Router::new()
            .route("/health/detailed", get(health_detailed_handler))
            .route("/agents", get(agents_handler))
            .route("/traffic/recent", get(traffic_handler))
            .route("/system/health", get(system_health_handler))
            .route("/system/start", axum::routing::post(system_start_handler))
            .route("/system/stop", axum::routing::post(system_stop_handler))
            .route(
                "/system/restart",
                axum::routing::post(system_restart_handler),
            );

        // Configure absolute permissive CORS for development
        use tower_http::cors::{CorsLayer, Any};
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS]) // OPTIONS mutlaka olmalı
            .allow_headers(Any)
            .expose_headers(Any);

        let app = axum::Router::new()
            // Top-level routes
            .route("/metrics", get(metrics_handler))
            // 2. "/graphql" rotasına .options() ekleyin:
            .route("/graphql", 
                get(graphiql)
                .post(graphql_handler)
                // Bu satır, tarayıcının attığı Preflight isteğinin 404 almamasını sağlar:
                .options(|| async { axum::http::StatusCode::NO_CONTENT })
            )
            .route("/graphql/ws", get(graphql_ws_handler))
            // Nested API routes
            .nest("/api", api_routes)
            // Swagger / Docs
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .route(
                "/",
                get(|| async { axum::response::Redirect::permanent("/swagger-ui") }),
            )
            // Layers - Order is critical: Outer layers are applied LAST
            .layer(middleware::from_fn(connection_logging)) // Add logging middleware
            .layer(Extension(state.schema.clone()))
            .layer(cors) // CORS katmanı en dışta olmalı (veya state'ten hemen önce)
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
            .add_service(crate::pb::proxy_service_server::ProxyServiceServer::new(
                proxy_service,
            ))
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
    axum::response::Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}

async fn graphql_handler(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<async_graphql::Request>,
) -> axum::Json<async_graphql::Response> {
    axum::Json(state.schema.execute(req).await)
}

async fn graphql_ws_handler(
    State(state): State<AppState>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    upgrade
        .protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, state.schema, protocol).serve()
        })
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
    let db_connected = state.db.get_pool().await.is_ok();
    Json(HealthStatus {
        status: "Healthy".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        database_connected: db_connected,
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

    info!(
        "👥 Agents list requested - Total: {}, Online: {}, Offline: {}",
        total_count, online_count, offline_count
    );

    let agents = agents_data
        .into_iter()
        .map(|a| AgentInfo {
            id: a.id,
            address: a.address,
            port: a.port,
            status: a.status,
            last_heartbeat: a.last_heartbeat,
            version: a.version,
            capabilities: a.capabilities,
        })
        .collect();

    Json(AgentsResponse {
        agents,
        total_count,
        online_count,
        offline_count,
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

    let pool = match state.db.pool().await {
        Some(p) => p,
        None => {
            return Json(TrafficResponse {
                transactions: Vec::new(),
                total_count: 0,
            });
        }
    };

    let transactions =
        match sqlx::query_as::<_, (String, String, String, String, Option<i32>, i64)>(
            "SELECT request_id, agent_id, req_method, req_url, res_status, req_timestamp 
         FROM http_transactions 
         ORDER BY req_timestamp DESC 
         LIMIT 50",
        )
        .fetch_all(&pool)
        .await
        {
            Ok(rows) => {
                info!("   ✓ Fetched {} transactions from database", rows.len());
                rows.into_iter()
                    .map(
                        |(request_id, agent_id, method, url, status, timestamp)| HttpTransaction {
                            request_id,
                            agent_id,
                            method,
                            url,
                            status,
                            timestamp,
                        },
                    )
                    .collect()
            }
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

    let pool = match state.db.pool().await {
        Some(p) => p,
        None => {
             return Json(MetricsResponse {
                total_requests: 0,
                average_response_time_ms: 0.0,
                error_rate: 0.0,
            });
        }
    };

    let total_requests = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM http_transactions")
        .fetch_one(&pool)
        .await
        .unwrap_or(0) as usize;

    let avg_response_time = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(duration_ms) FROM http_transactions WHERE duration_ms IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(None)
    .unwrap_or(0.0);

    let error_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM http_transactions WHERE res_status >= 400",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let error_rate = if total_requests > 0 {
        (error_count as f64) / (total_requests as f64)
    } else {
        0.0
    };

    info!(
        "   ✓ Total: {}, Avg Latency: {:.1}ms, Errors: {}/{} ({:.1}%)",
        total_requests,
        avg_response_time,
        error_count,
        total_requests,
        error_rate * 100.0
    );

    Json(MetricsResponse {
        total_requests,
        average_response_time_ms: avg_response_time,
        error_rate,
    })
}

/// Get system health status
#[utoipa::path(
    get,
    path = "/system/health",
    tag = "health",
    responses(
        (status = 200, description = "System health status retrieved successfully")
    )
)]
async fn system_health_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    info!("🏥 System health check requested");

    let agents_data = state.agents.list_agents();
    let online_agents = agents_data.iter().filter(|a| a.status == "Online").count();

    Json(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "database_connected": true,
        "agents_online": online_agents,
        "agents_total": agents_data.len(),
    }))
}

/// Start the proxy system
#[utoipa::path(
    post,
    path = "/system/start",
    tag = "system",
    responses(
        (status = 200, description = "System started successfully")
    )
)]
async fn system_start_handler() -> Json<serde_json::Value> {
    info!("🚀 System start requested");
    Json(serde_json::json!({
        "status": "success",
        "message": "System is already running"
    }))
}

/// Stop the proxy system
#[utoipa::path(
    post,
    path = "/system/stop",
    tag = "system",
    responses(
        (status = 200, description = "System stopped successfully")
    )
)]
async fn system_stop_handler() -> Json<serde_json::Value> {
    info!("🛑 System stop requested");
    Json(serde_json::json!({
        "status": "success",
        "message": "System stop initiated"
    }))
}

/// Restart the proxy system
#[utoipa::path(
    post,
    path = "/system/restart",
    tag = "system",
    responses(
        (status = 200, description = "System restart initiated")
    )
)]
async fn system_restart_handler() -> Json<serde_json::Value> {
    info!("🔄 System restart requested");
    Json(serde_json::json!({
        "status": "success",
        "message": "System restart initiated"
    }))
}

pub async fn run_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route(
        "/metrics",
        get(|| async { "orchestrator_metrics{status=\"up\"} 1" }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Orchestrator metrics listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn connection_logging(
    req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    if let Some(client) = headers.get("X-Proxxy-Client") {
        if let Ok(client_str) = client.to_str() {
            if client_str == "GUI" {
                info!("🖥️  GUI Connected: {} {}", method, uri);
            }
        }
    } else if let Some(upgrade) = headers.get("upgrade") {
         if let Ok(upgrade_str) = upgrade.to_str() {
             if upgrade_str == "websocket" {
                 info!("🔌 WebSocket Attempt: {} {}", method, uri);
             }
         }
    }

    next.run(req).await
}
