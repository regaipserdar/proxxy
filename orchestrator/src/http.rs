use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    AgentInfo, DatabaseManager, HealthChecker, HealthStatus, 
    OrchestratorError, TrafficData
};

/// HTTP server for the orchestrator REST API
pub struct HttpServer {
    database: Arc<DatabaseManager>,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    health_checker: Arc<HealthChecker>,
}

/// Application state shared across HTTP handlers
#[derive(Clone)]
pub struct AppState {
    database: Arc<DatabaseManager>,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    health_checker: Arc<HealthChecker>,
}

/// Response for the root endpoint
#[derive(Serialize)]
pub struct WelcomeResponse {
    pub service: String,
    pub version: String,
    pub status: String,
    pub endpoints: Vec<EndpointInfo>,
}

/// Information about available endpoints
#[derive(Serialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub description: String,
}

/// Response for agent list endpoint
#[derive(Serialize)]
pub struct AgentsResponse {
    pub agents: Vec<AgentInfo>,
    pub total_count: usize,
    pub online_count: usize,
    pub offline_count: usize,
}

/// Query parameters for traffic data
#[derive(Deserialize)]
pub struct TrafficQuery {
    pub agent_id: Option<String>,
    pub limit: Option<i64>,
}

/// Response for traffic data endpoint
#[derive(Serialize)]
pub struct TrafficResponse {
    pub traffic_data: Vec<TrafficData>,
    pub total_count: usize,
}

/// Request body for registering a new agent
#[derive(Deserialize)]
pub struct RegisterAgentRequest {
    pub agent_id: String,
    pub address: String,
    pub port: u16,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Response for agent registration
#[derive(Serialize)]
pub struct RegisterAgentResponse {
    pub success: bool,
    pub message: String,
    pub agent_id: String,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(
        database: Arc<DatabaseManager>,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
        health_checker: Arc<HealthChecker>,
    ) -> Self {
        Self {
            database,
            agents,
            health_checker,
        }
    }
    
    /// Start the HTTP server
    pub async fn start(&self, port: u16) -> Result<(), OrchestratorError> {
        let app_state = AppState {
            database: self.database.clone(),
            agents: self.agents.clone(),
            health_checker: self.health_checker.clone(),
        };
        
        let app = Router::new()
            // Root endpoint - welcome page
            .route("/", get(root_handler))
            
            // Health endpoints
            .route("/health", get(health_handler))
            .route("/health/detailed", get(detailed_health_handler))
            
            // Agent management endpoints
            .route("/agents", get(list_agents_handler))
            .route("/agents", post(register_agent_handler))
            .route("/agents/:agent_id", get(get_agent_handler))
            
            // Traffic data endpoints
            .route("/traffic", get(list_traffic_handler))
            .route("/traffic/:agent_id", get(get_agent_traffic_handler))
            
            // Metrics endpoints
            .route("/metrics", get(system_metrics_handler))
            .route("/metrics/:agent_id", get(agent_metrics_handler))
            
            // Add middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
            )
            .with_state(app_state);
        
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| OrchestratorError::Io(e))?;
        
        tracing::info!("HTTP server starting on {}", addr);
        
        axum::serve(listener, app).await
            .map_err(|e| OrchestratorError::Io(e))?;
        
        Ok(())
    }
}

/// Root endpoint handler - provides welcome message and API documentation
async fn root_handler() -> Json<WelcomeResponse> {
    let endpoints = vec![
        EndpointInfo {
            path: "/".to_string(),
            method: "GET".to_string(),
            description: "This welcome page with API documentation".to_string(),
        },
        EndpointInfo {
            path: "/health".to_string(),
            method: "GET".to_string(),
            description: "Basic health check".to_string(),
        },
        EndpointInfo {
            path: "/health/detailed".to_string(),
            method: "GET".to_string(),
            description: "Detailed system health status".to_string(),
        },
        EndpointInfo {
            path: "/agents".to_string(),
            method: "GET".to_string(),
            description: "List all registered proxy agents".to_string(),
        },
        EndpointInfo {
            path: "/agents".to_string(),
            method: "POST".to_string(),
            description: "Register a new proxy agent".to_string(),
        },
        EndpointInfo {
            path: "/agents/{agent_id}".to_string(),
            method: "GET".to_string(),
            description: "Get information about a specific agent".to_string(),
        },
        EndpointInfo {
            path: "/traffic".to_string(),
            method: "GET".to_string(),
            description: "Get recent traffic data (query params: agent_id, limit)".to_string(),
        },
        EndpointInfo {
            path: "/traffic/{agent_id}".to_string(),
            method: "GET".to_string(),
            description: "Get traffic data for a specific agent".to_string(),
        },
        EndpointInfo {
            path: "/metrics".to_string(),
            method: "GET".to_string(),
            description: "Get system-wide metrics".to_string(),
        },
        EndpointInfo {
            path: "/metrics/{agent_id}".to_string(),
            method: "GET".to_string(),
            description: "Get metrics for a specific agent".to_string(),
        },
    ];
    
    Json(WelcomeResponse {
        service: "Distributed MITM Proxxy Orchestrator".to_string(),
        version: "0.1.1".to_string(),
        status: "running".to_string(),
        endpoints,
    })
}

/// Basic health check handler
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "orchestrator"
    }))
}

/// Detailed health status handler
async fn detailed_health_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthStatus>, StatusCode> {
    let health_status = state.health_checker.get_health_status(&state.agents).await;
    Ok(Json(health_status))
}

/// List all agents handler
async fn list_agents_handler(
    State(state): State<AppState>,
) -> Result<Json<AgentsResponse>, StatusCode> {
    let agents_guard = state.agents.read().await;
    let agents: Vec<AgentInfo> = agents_guard.values().cloned().collect();
    
    let total_count = agents.len();
    let online_count = agents.iter()
        .filter(|a| matches!(a.status, crate::AgentStatus::Online))
        .count();
    let offline_count = agents.iter()
        .filter(|a| matches!(a.status, crate::AgentStatus::Offline))
        .count();
    
    Ok(Json(AgentsResponse {
        agents,
        total_count,
        online_count,
        offline_count,
    }))
}

/// Get specific agent handler
async fn get_agent_handler(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AgentInfo>, StatusCode> {
    let agents_guard = state.agents.read().await;
    
    if let Some(agent) = agents_guard.get(&agent_id) {
        Ok(Json(agent.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Register new agent handler
async fn register_agent_handler(
    State(state): State<AppState>,
    Json(request): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, StatusCode> {
    let agent_info = AgentInfo {
        id: request.agent_id.clone(),
        address: request.address,
        port: request.port,
        status: crate::AgentStatus::Online,
        last_heartbeat: chrono::Utc::now(),
        version: request.version,
        capabilities: request.capabilities,
    };
    
    // Store in database
    if let Err(e) = state.database.store_agent_info(&agent_info).await {
        tracing::error!("Failed to store agent info: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Add to in-memory registry
    let mut agents_guard = state.agents.write().await;
    agents_guard.insert(agent_info.id.clone(), agent_info);
    
    tracing::info!("Agent registered via HTTP API: {}", request.agent_id);
    
    Ok(Json(RegisterAgentResponse {
        success: true,
        message: "Agent registered successfully".to_string(),
        agent_id: request.agent_id,
    }))
}

/// List traffic data handler
async fn list_traffic_handler(
    Query(params): Query<TrafficQuery>,
    State(_state): State<AppState>,
) -> Result<Json<TrafficResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(100);
    
    // For now, return empty data as the database methods would need to be implemented
    // In a real implementation, this would query the database for traffic data
    tracing::debug!("Traffic data requested with limit: {}, agent_id: {:?}", 
                   limit, params.agent_id);
    
    Ok(Json(TrafficResponse {
        traffic_data: Vec::new(),
        total_count: 0,
    }))
}

/// Get agent-specific traffic data handler
async fn get_agent_traffic_handler(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<TrafficResponse>, StatusCode> {
    // Check if agent exists
    let agents_guard = state.agents.read().await;
    if !agents_guard.contains_key(&agent_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    drop(agents_guard);
    
    // For now, return empty data
    // In a real implementation, this would query the database for agent-specific traffic
    tracing::debug!("Traffic data requested for agent: {}", agent_id);
    
    Ok(Json(TrafficResponse {
        traffic_data: Vec::new(),
        total_count: 0,
    }))
}

/// System metrics handler
async fn system_metrics_handler(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return basic metrics
    // In a real implementation, this would use the MetricsCollector
    Ok(Json(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "total_requests": 0,
        "requests_per_second": 0.0,
        "average_response_time_ms": 0.0,
        "error_rate": 0.0,
        "active_agents": 0
    })))
}

/// Agent-specific metrics handler
async fn agent_metrics_handler(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if agent exists
    let agents_guard = state.agents.read().await;
    if !agents_guard.contains_key(&agent_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    drop(agents_guard);
    
    // For now, return basic metrics
    // In a real implementation, this would query the database for agent metrics
    tracing::debug!("Metrics requested for agent: {}", agent_id);
    
    Ok(Json(serde_json::json!({
        "agent_id": agent_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "requests_handled": 0,
        "average_response_time_ms": 0.0,
        "error_rate": 0.0,
        "memory_usage_mb": 0,
        "cpu_usage_percent": 0.0
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseManager;
    
    #[tokio::test]
    async fn test_http_server_creation() {
        let database = Arc::new(DatabaseManager::new("sqlite::memory:").await.unwrap());
        let agents = Arc::new(RwLock::new(HashMap::new()));
        let health_checker = Arc::new(crate::HealthChecker::new(30, 300));
        
        let _server = HttpServer::new(database, agents, health_checker);
        // Server creation should succeed
        assert!(true);
    }
}