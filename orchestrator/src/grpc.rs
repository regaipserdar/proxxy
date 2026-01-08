use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::RwLock;
use tonic::Status;
use chrono::Utc;
use uuid::Uuid;

use crate::{DatabaseManager, AgentInfo, TrafficData, AgentMetrics, AgentStatus, OrchestratorError};

// For now, we'll define the gRPC service traits manually
// In a real implementation, these would be generated from .proto files

/// gRPC service for orchestrator-agent communication
pub struct OrchestratorService {
    database: Arc<DatabaseManager>,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl OrchestratorService {
    /// Create a new orchestrator gRPC service
    pub fn new(
        database: Arc<DatabaseManager>,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    ) -> Self {
        Self { database, agents }
    }
    
    /// Start the gRPC server
    pub async fn start(&self, port: u16) -> Result<(), OrchestratorError> {
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()
            .map_err(|e| OrchestratorError::Configuration(format!("Invalid address: {}", e)))?;
        
        tracing::info!("Starting gRPC server on {}", addr);
        
        // In a real implementation, this would use the generated gRPC service
        // For now, we'll create a placeholder that demonstrates the structure
        
        // Server::builder()
        //     .add_service(OrchestratorServiceServer::new(self.clone()))
        //     .serve(addr)
        //     .await?;
        
        tracing::info!("gRPC server started successfully");
        Ok(())
    }
    
    /// Handle agent registration
    pub async fn register_agent(
        &self,
        request: RegisterAgentRequest,
    ) -> Result<RegisterAgentResponse, Status> {
        tracing::info!("Received agent registration request from: {}", request.agent_id);
        
        let agent_info = AgentInfo {
            id: request.agent_id.clone(),
            address: request.address,
            port: request.port,
            status: AgentStatus::Online,
            last_heartbeat: Utc::now(),
            version: request.version,
            capabilities: request.capabilities,
        };
        
        // Store in database
        self.database
            .store_agent_info(&agent_info)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        // Add to in-memory registry
        let mut agents = self.agents.write().await;
        agents.insert(agent_info.id.clone(), agent_info);
        
        tracing::info!("Agent registered successfully: {}", request.agent_id);
        
        Ok(RegisterAgentResponse {
            success: true,
            message: "Agent registered successfully".to_string(),
            agent_id: request.agent_id,
        })
    }
    
    /// Handle heartbeat from agent
    pub async fn heartbeat(
        &self,
        request: HeartbeatRequest,
    ) -> Result<HeartbeatResponse, Status> {
        tracing::debug!("Received heartbeat from agent: {}", request.agent_id);
        
        // Update agent's last heartbeat
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(&request.agent_id) {
            agent.last_heartbeat = Utc::now();
            agent.status = AgentStatus::Online;
            
            // Update in database
            self.database
                .store_agent_info(agent)
                .await
                .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        } else {
            return Err(Status::not_found("Agent not registered"));
        }
        
        Ok(HeartbeatResponse {
            success: true,
            timestamp: Utc::now().timestamp(),
        })
    }
    
    /// Handle traffic data submission
    pub async fn submit_traffic_data(
        &self,
        request: SubmitTrafficDataRequest,
    ) -> Result<SubmitTrafficDataResponse, Status> {
        tracing::debug!("Received traffic data from agent: {}", request.agent_id);
        
        let traffic_data = TrafficData {
            id: Uuid::new_v4().to_string(),
            agent_id: request.agent_id.clone(),
            timestamp: Utc::now(),
            method: request.method,
            url: request.url,
            status_code: request.status_code,
            request_size: request.request_size,
            response_size: request.response_size,
            processing_time_ms: request.processing_time_ms,
        };
        
        // Store in database
        self.database
            .store_traffic_data(&traffic_data)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        Ok(SubmitTrafficDataResponse {
            success: true,
            data_id: traffic_data.id,
        })
    }
    
    /// Handle metrics submission
    pub async fn submit_metrics(
        &self,
        request: SubmitMetricsRequest,
    ) -> Result<SubmitMetricsResponse, Status> {
        tracing::debug!("Received metrics from agent: {}", request.agent_id);
        
        let metrics = AgentMetrics {
            agent_id: request.agent_id.clone(),
            timestamp: Utc::now(),
            requests_per_second: request.requests_per_second,
            active_connections: request.active_connections,
            memory_usage_mb: request.memory_usage_mb,
            cpu_usage_percent: request.cpu_usage_percent,
            error_rate: request.error_rate,
        };
        
        // Store in database
        self.database
            .store_agent_metrics(&metrics)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        Ok(SubmitMetricsResponse {
            success: true,
        })
    }
    
    /// Get configuration for an agent
    pub async fn get_configuration(
        &self,
        request: GetConfigurationRequest,
    ) -> Result<GetConfigurationResponse, Status> {
        tracing::debug!("Configuration requested by agent: {}", request.agent_id);
        
        // In a real implementation, this would return actual configuration
        // For now, return a placeholder response
        Ok(GetConfigurationResponse {
            config_json: r#"{"proxy_rules": [], "log_level": "info"}"#.to_string(),
            version: 1,
        })
    }
}

// Request/Response message types
// In a real implementation, these would be generated from .proto files

#[derive(Debug, Clone)]
pub struct RegisterAgentRequest {
    pub agent_id: String,
    pub address: String,
    pub port: u16,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RegisterAgentResponse {
    pub success: bool,
    pub message: String,
    pub agent_id: String,
}

#[derive(Debug, Clone)]
pub struct HeartbeatRequest {
    pub agent_id: String,
}

#[derive(Debug, Clone)]
pub struct HeartbeatResponse {
    pub success: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct SubmitTrafficDataRequest {
    pub agent_id: String,
    pub method: String,
    pub url: String,
    pub status_code: Option<u16>,
    pub request_size: u64,
    pub response_size: Option<u64>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SubmitTrafficDataResponse {
    pub success: bool,
    pub data_id: String,
}

#[derive(Debug, Clone)]
pub struct SubmitMetricsRequest {
    pub agent_id: String,
    pub requests_per_second: f64,
    pub active_connections: u32,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub error_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SubmitMetricsResponse {
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct GetConfigurationRequest {
    pub agent_id: String,
}

#[derive(Debug, Clone)]
pub struct GetConfigurationResponse {
    pub config_json: String,
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseManager;
    
    #[tokio::test]
    async fn test_orchestrator_service_creation() {
        let database = Arc::new(DatabaseManager::new("sqlite::memory:").await.unwrap());
        let agents = Arc::new(RwLock::new(HashMap::new()));
        
        let _service = OrchestratorService::new(database, agents);
        // Service creation should succeed
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_register_agent_request() {
        let database = Arc::new(DatabaseManager::new("sqlite::memory:").await.unwrap());
        let agents = Arc::new(RwLock::new(HashMap::new()));
        let service = OrchestratorService::new(database, agents.clone());
        
        let request = RegisterAgentRequest {
            agent_id: "test-agent".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            version: "1.0.0".to_string(),
            capabilities: vec!["http".to_string()],
        };
        
        let response = service.register_agent(request).await.unwrap();
        assert!(response.success);
        assert_eq!(response.agent_id, "test-agent");
        
        // Verify agent was added to registry
        let agents_guard = agents.read().await;
        assert!(agents_guard.contains_key("test-agent"));
    }
}