//! Repeater Module - Manual Request Manipulation
//! 
//! This module provides the RepeaterManager for handling manual request editing,
//! agent selection, and request execution through the distributed agent infrastructure.

use crate::Database;
use crate::session_manager::AgentRegistry;
use crate::session_integration::{SessionManager, SessionApplicationResult, ExpirationHandling, SessionSelectionCriteria, SessionRefreshResult};
use attack_engine::{HttpRequestData, HttpResponseData, AttackError, AttackResult};
use proxy_common::session::Session;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Request for creating a new repeater tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepeaterTabRequest {
    pub name: String,
    pub request_template: HttpRequestData,
    pub target_agent_id: Option<String>,
}

/// Request for executing a repeater request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterExecutionRequest {
    pub tab_id: String,
    pub request_data: HttpRequestData,
    pub target_agent_id: String,
    pub session_id: Option<String>, // Use String instead of Uuid for serialization
}

/// Response from repeater execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterExecutionResponse {
    pub execution_id: String,
    pub tab_id: String, // Add tab_id to the response
    pub request_data: HttpRequestData,
    pub response_data: Option<HttpResponseData>,
    pub agent_id: String,
    pub duration_ms: Option<u64>,
    pub status_code: Option<i32>,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

/// Repeater tab configuration with validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterTabConfig {
    pub id: String,
    pub name: String,
    pub request_template: HttpRequestData,
    pub target_agent_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub validation_status: ValidationStatus,
}

/// Validation status for repeater configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    InvalidRequest { reason: String },
    InvalidAgent { reason: String },
    Unknown,
}

/// Agent information for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSelectionInfo {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub status: String,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
    pub capabilities: Vec<String>,
    pub is_available: bool,
}

/// Agent health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthStatus {
    pub agent_id: String,
    pub status: HealthStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub heartbeat_age_seconds: i64,
    pub capabilities: Vec<String>,
    pub address: String,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Detailed error information for user notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub message: String,
    pub remediation: Vec<String>,
    pub is_retryable: bool,
}

/// Execution statistics for a repeater tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub error_count: usize,
    pub average_duration_ms: f64,
    pub status_code_distribution: HashMap<i32, usize>,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Main RepeaterManager for handling all repeater operations
pub struct RepeaterManager {
    database: Arc<Database>,
    agent_registry: Arc<AgentRegistry>,
    session_manager: Arc<SessionManager>,
    active_tabs: Arc<RwLock<HashMap<String, RepeaterTabConfig>>>,
}

impl RepeaterManager {
    /// Create a new RepeaterManager instance
    pub fn new(
        database: Arc<Database>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            database,
            agent_registry,
            session_manager: Arc::new(SessionManager::new()),
            active_tabs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the RepeaterManager by loading existing tabs
    pub async fn initialize(&self) -> AttackResult<()> {
        info!("🔄 Initializing RepeaterManager...");
        
        match self.database.get_repeater_tabs().await {
            Ok(tabs) => {
                let mut active_tabs = self.active_tabs.write().await;
                for tab in tabs {
                    let config = RepeaterTabConfig {
                        id: tab.id.clone(),
                        name: tab.name,
                        request_template: serde_json::from_str(&tab.request_template)
                            .map_err(|e| AttackError::InvalidPayloadConfig { 
                                reason: format!("Failed to deserialize request template: {}", e) 
                            })?,
                        target_agent_id: tab.target_agent_id,
                        created_at: chrono::DateTime::from_timestamp(tab.created_at, 0)
                            .unwrap_or_else(chrono::Utc::now),
                        updated_at: chrono::DateTime::from_timestamp(tab.updated_at, 0)
                            .unwrap_or_else(chrono::Utc::now),
                        is_active: tab.is_active,
                        validation_status: ValidationStatus::Unknown,
                    };
                    active_tabs.insert(tab.id, config);
                }
                info!("   ✓ Loaded {} repeater tabs", active_tabs.len());
            }
            Err(e) => {
                warn!("   ⚠ Failed to load repeater tabs: {}", e);
            }
        }

        Ok(())
    }

    /// Create a new repeater tab
    pub async fn create_tab(&self, request: CreateRepeaterTabRequest) -> AttackResult<String> {
        info!("📝 Creating new repeater tab: {}", request.name);

        // Validate request template
        self.validate_request_template(&request.request_template)?;

        // Validate target agent if specified
        if let Some(agent_id) = &request.target_agent_id {
            self.validate_agent_availability(agent_id).await?;
        }

        // Serialize request template
        let request_template_json = serde_json::to_string(&request.request_template)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize request template: {}", e),
            })?;

        // Save to database
        let tab_id = self.database
            .create_repeater_tab(
                &request.name,
                &request_template_json,
                request.target_agent_id.as_deref(),
            )
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("create_repeater_tab: {}", e),
            })?;

        // Add to active tabs
        let config = RepeaterTabConfig {
            id: tab_id.clone(),
            name: request.name,
            request_template: request.request_template,
            target_agent_id: request.target_agent_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
            validation_status: ValidationStatus::Valid,
        };

        self.active_tabs.write().await.insert(tab_id.clone(), config);

        info!("   ✓ Created repeater tab: {}", tab_id);
        Ok(tab_id)
    }

    /// Get all active repeater tabs
    pub async fn get_tabs(&self) -> Vec<RepeaterTabConfig> {
        self.active_tabs.read().await.values().cloned().collect()
    }

    /// Get a specific repeater tab by ID
    pub async fn get_tab(&self, tab_id: &str) -> Option<RepeaterTabConfig> {
        self.active_tabs.read().await.get(tab_id).cloned()
    }

    /// Update a repeater tab configuration
    pub async fn update_tab(
        &self,
        tab_id: &str,
        name: Option<String>,
        request_template: Option<HttpRequestData>,
        target_agent_id: Option<Option<String>>,
    ) -> AttackResult<()> {
        info!("✏️ Updating repeater tab: {}", tab_id);

        // Validate request template if provided
        if let Some(ref template) = request_template {
            self.validate_request_template(template)?;
        }

        // Validate target agent if provided
        if let Some(Some(ref agent_id)) = target_agent_id {
            self.validate_agent_availability(agent_id).await?;
        }

        // Serialize request template if provided
        let request_template_json = if let Some(ref template) = request_template {
            Some(serde_json::to_string(template)
                .map_err(|e| AttackError::InvalidPayloadConfig {
                    reason: format!("Failed to serialize request template: {}", e),
                })?)
        } else {
            None
        };

        // Update database
        self.database
            .update_repeater_tab(
                tab_id,
                name.as_deref(),
                request_template_json.as_deref(),
                target_agent_id.as_ref().map(|opt| opt.as_deref()),
            )
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("update_repeater_tab: {}", e),
            })?;

        // Update active tabs
        if let Some(config) = self.active_tabs.write().await.get_mut(tab_id) {
            if let Some(name) = name {
                config.name = name;
            }
            if let Some(template) = request_template {
                config.request_template = template;
            }
            if let Some(agent_id) = target_agent_id {
                config.target_agent_id = agent_id;
            }
            config.updated_at = chrono::Utc::now();
            config.validation_status = ValidationStatus::Valid;
        }

        info!("   ✓ Updated repeater tab: {}", tab_id);
        Ok(())
    }

    /// Delete a repeater tab (soft delete)
    pub async fn delete_tab(&self, tab_id: &str) -> AttackResult<()> {
        info!("🗑️ Deleting repeater tab: {}", tab_id);

        // Soft delete in database
        self.database
            .delete_repeater_tab(tab_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("delete_repeater_tab: {}", e),
            })?;

        // Remove from active tabs
        self.active_tabs.write().await.remove(tab_id);

        info!("   ✓ Deleted repeater tab: {}", tab_id);
        Ok(())
    }

    /// Get available agents for selection
    pub async fn get_available_agents(&self) -> Vec<AgentSelectionInfo> {
        let agents = self.agent_registry.list_agents();
        
        agents.into_iter().map(|agent| {
            let is_available = agent.status == "Online";
            let last_heartbeat = chrono::DateTime::parse_from_rfc3339(&agent.last_heartbeat)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);

            AgentSelectionInfo {
                id: agent.id,
                address: agent.address,
                port: agent.port,
                status: agent.status,
                last_heartbeat,
                response_time_ms: None, // TODO: Add response time tracking
                capabilities: agent.capabilities,
                is_available,
            }
        }).collect()
    }

    /// Validate agent availability
    pub async fn validate_agent_availability(&self, agent_id: &str) -> AttackResult<()> {
        let agents = self.agent_registry.list_agents();
        
        match agents.iter().find(|a| a.id == agent_id) {
            Some(agent) if agent.status == "Online" => {
                info!("   ✓ Agent {} is available", agent_id);
                Ok(())
            }
            Some(_) => {
                warn!("   ⚠ Agent {} is offline", agent_id);
                Err(AttackError::AgentUnavailable {
                    agent_id: agent_id.to_string(),
                })
            }
            None => {
                warn!("   ⚠ Agent {} not found", agent_id);
                Err(AttackError::AgentUnavailable {
                    agent_id: agent_id.to_string(),
                })
            }
        }
    }

    /// Add or update a session for use in repeater requests
    pub async fn add_session(&self, session: Session) -> AttackResult<()> {
        info!("🔐 Adding session to repeater: {} ({})", session.name, session.id);
        self.session_manager.add_session(session).await
    }

    /// Get available sessions
    pub async fn get_sessions(&self) -> Vec<Session> {
        self.session_manager.get_sessions().await
    }

    /// Get active sessions only
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        self.session_manager.get_active_sessions().await
    }

    /// Get a specific session by ID
    pub async fn get_session(&self, session_id: &Uuid) -> Option<Session> {
        self.session_manager.get_session(session_id).await
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &Uuid) -> AttackResult<()> {
        self.session_manager.remove_session(session_id).await
    }

    /// Select best session based on criteria
    pub async fn select_session(&self, criteria: &SessionSelectionCriteria) -> Option<Session> {
        self.session_manager.select_session(criteria).await
    }

    /// Validate session against target URL
    pub async fn validate_session(&self, session_id: &Uuid, validation_url: &str) -> AttackResult<bool> {
        self.session_manager.validate_session(session_id, validation_url).await
    }

    /// Detect authentication failure from response
    pub async fn detect_authentication_failure(
        &self,
        response: &HttpResponseData,
        request_url: &str,
    ) -> bool {
        self.session_manager.detect_authentication_failure(response, request_url).await
    }

    /// Handle authentication failure and attempt refresh
    pub async fn handle_authentication_failure(
        &self,
        session_id: &Uuid,
        failure_url: &str,
        response: &HttpResponseData,
    ) -> AttackResult<SessionRefreshResult> {
        self.session_manager.handle_authentication_failure(session_id, failure_url, response).await
    }

    /// Refresh session manually
    pub async fn refresh_session_manually(
        &self,
        session_id: &Uuid,
        new_session_data: Session,
    ) -> AttackResult<SessionRefreshResult> {
        self.session_manager.refresh_session_manually(session_id, new_session_data).await
    }

    /// Apply session data to a request with enhanced error handling
    pub async fn apply_session_to_request(
        &self,
        request: HttpRequestData,
        session_id: &Uuid,
        expiration_handling: Option<ExpirationHandling>,
    ) -> AttackResult<(HttpRequestData, SessionApplicationResult)> {
        let handling = expiration_handling.unwrap_or(ExpirationHandling::Fail);
        self.session_manager.apply_session_to_request(request, session_id, handling).await
    }

    /// Execute a repeater request through the selected agent
    pub async fn execute_request(
        &self,
        request: RepeaterExecutionRequest,
    ) -> AttackResult<RepeaterExecutionResponse> {
        info!("🚀 Executing repeater request for tab: {}", request.tab_id);

        // Validate tab exists
        let _tab = self.get_tab(&request.tab_id).await
            .ok_or_else(|| AttackError::InvalidPayloadConfig {
                reason: format!("Repeater tab {} not found", request.tab_id),
            })?;

        // Validate agent availability
        self.validate_agent_availability(&request.target_agent_id).await?;

        // Apply session data if provided
        let mut final_request = request.request_data.clone();
        let mut session_result: Option<SessionApplicationResult> = None;
        
        if let Some(session_id_str) = &request.session_id {
            if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                match self.apply_session_to_request(
                    final_request.clone(), 
                    &session_id, 
                    Some(ExpirationHandling::ContinueWithoutSession)
                ).await {
                    Ok((modified_request, app_result)) => {
                        final_request = modified_request;
                        info!("   ✓ Session applied: {}", app_result.session_name);
                        session_result = Some(app_result);
                    }
                    Err(e) => {
                        warn!("   ⚠ Failed to apply session: {}", e);
                        // Continue without session
                    }
                }
            }
        }

        // Validate the final request
        self.validate_request_template(&final_request)?;

        let start_time = std::time::Instant::now();
        let executed_at = chrono::Utc::now();

        // Execute request through agent (placeholder for actual gRPC call)
        let (response_data, error): (Option<HttpResponseData>, Option<String>) = match self.execute_through_agent(&final_request, &request.target_agent_id).await {
            Ok(response) => {
                // Check for authentication failure if session was used
                if let Some(ref session_result) = session_result {
                    if self.detect_authentication_failure(&response, &final_request.url).await {
                        warn!("   🚨 Authentication failure detected in response");
                        
                        // Parse session ID for failure handling
                        if let Some(session_id_str) = &request.session_id {
                            if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                                match self.handle_authentication_failure(&session_id, &final_request.url, &response).await {
                                    Ok(refresh_result) => {
                                        if refresh_result.success {
                                            info!("   ✓ Session refreshed successfully");
                                        } else {
                                            warn!("   ⚠ Session refresh failed: {:?}", refresh_result.error);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("   ⚠ Failed to handle authentication failure: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                
                (Some(response), None)
            }
            Err(e) => {
                error!("   ✗ Request execution failed: {}", e);
                (None, Some(e.to_string()))
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let status_code: Option<i32> = response_data.as_ref().map(|r| r.status_code);

        // Serialize request and response for database storage
        let request_json = serde_json::to_string(&final_request)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize request: {}", e),
            })?;

        let response_json = if let Some(ref response) = response_data {
            Some(serde_json::to_string(response)
                .map_err(|e| AttackError::InvalidPayloadConfig {
                    reason: format!("Failed to serialize response: {}", e),
                })?)
        } else {
            None
        };

        // Save execution to database
        let execution_id = self.database
            .save_repeater_execution(
                &request.tab_id,
                &request_json,
                response_json.as_deref(),
                &request.target_agent_id,
                Some(duration_ms as i64),
                status_code,
            )
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("save_repeater_execution: {}", e),
            })?;

        let response = RepeaterExecutionResponse {
            execution_id,
            tab_id: request.tab_id.clone(), // Include tab_id in response
            request_data: final_request,
            response_data,
            agent_id: request.target_agent_id,
            duration_ms: Some(duration_ms),
            status_code,
            executed_at,
            error,
        };

        if response.error.is_none() {
            info!("   ✓ Request executed successfully in {}ms", duration_ms);
        } else {
            warn!("   ⚠ Request execution completed with error");
        }

        Ok(response)
    }

    /// Get execution history for a repeater tab
    pub async fn get_execution_history(
        &self,
        tab_id: &str,
        limit: Option<i64>,
    ) -> AttackResult<Vec<RepeaterExecutionResponse>> {
        info!("📜 Getting execution history for tab: {}", tab_id);

        let executions = self.database
            .get_repeater_history(tab_id, limit)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_repeater_history: {}", e),
            })?;

        let mut responses = Vec::new();
        for execution in executions {
            let request_data: HttpRequestData = serde_json::from_str(&execution.request_data)
                .map_err(|e| AttackError::InvalidPayloadConfig {
                    reason: format!("Failed to deserialize request: {}", e),
                })?;

            let response_data: Option<HttpResponseData> = if let Some(response_json) = &execution.response_data {
                Some(serde_json::from_str(response_json)
                    .map_err(|e| AttackError::InvalidPayloadConfig {
                        reason: format!("Failed to deserialize response: {}", e),
                    })?)
            } else {
                None
            };

            responses.push(RepeaterExecutionResponse {
                execution_id: execution.id,
                tab_id: execution.tab_id, // Include tab_id from database
                request_data,
                response_data,
                agent_id: execution.agent_id,
                duration_ms: execution.duration_ms.map(|d| d as u64),
                status_code: execution.status_code,
                executed_at: chrono::DateTime::from_timestamp(execution.executed_at, 0)
                    .unwrap_or_else(chrono::Utc::now),
                error: None,
            });
        }

        info!("   ✓ Retrieved {} execution records", responses.len());
        Ok(responses)
    }

    /// Get a specific execution by ID
    pub async fn get_execution(&self, execution_id: &str) -> AttackResult<Option<RepeaterExecutionResponse>> {
        info!("🔍 Getting execution: {}", execution_id);

        let execution = self.database
            .get_repeater_execution(execution_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_repeater_execution: {}", e),
            })?;

        if let Some(execution) = execution {
            let request_data: HttpRequestData = serde_json::from_str(&execution.request_data)
                .map_err(|e| AttackError::InvalidPayloadConfig {
                    reason: format!("Failed to deserialize request: {}", e),
                })?;

            let response_data: Option<HttpResponseData> = if let Some(response_json) = &execution.response_data {
                Some(serde_json::from_str(response_json)
                    .map_err(|e| AttackError::InvalidPayloadConfig {
                        reason: format!("Failed to deserialize response: {}", e),
                    })?)
            } else {
                None
            };

            let response = RepeaterExecutionResponse {
                execution_id: execution.id,
                tab_id: execution.tab_id, // Include tab_id from database
                request_data,
                response_data,
                agent_id: execution.agent_id,
                duration_ms: execution.duration_ms.map(|d| d as u64),
                status_code: execution.status_code,
                executed_at: chrono::DateTime::from_timestamp(execution.executed_at, 0)
                    .unwrap_or_else(chrono::Utc::now),
                error: None,
            };

            info!("   ✓ Retrieved execution record");
            Ok(Some(response))
        } else {
            info!("   ⚠ Execution not found");
            Ok(None)
        }
    }

    /// Execute request through agent (placeholder for actual gRPC implementation)
    async fn execute_through_agent(
        &self,
        request: &HttpRequestData,
        agent_id: &str,
    ) -> AttackResult<HttpResponseData> {
        info!("📡 Executing request through agent: {}", agent_id);

        // Validate agent is still available before execution
        self.validate_agent_availability(agent_id).await?;

        // TODO: Implement actual gRPC call to agent with proper error handling
        // For now, return a mock response to enable testing
        
        // Simulate potential network issues
        if agent_id == "test-offline-agent" {
            return Err(AttackError::AgentUnavailable {
                agent_id: agent_id.to_string(),
            });
        }

        if agent_id == "test-network-error" {
            return Err(AttackError::NetworkError {
                details: "Connection timeout after 30 seconds".to_string(),
            });
        }

        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Create mock response based on request
        let mock_response = HttpResponseData {
            status_code: 200,
            headers: Some(attack_engine::HttpHeaders {
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers.insert("Server".to_string(), "nginx/1.18.0".to_string());
                    headers
                },
            }),
            body: format!(
                r#"{{"message": "Mock response for {} {}", "agent": "{}"}}"#,
                request.method, request.url, agent_id
            ).into_bytes(),
            tls: None,
        };

        info!("   ✓ Mock response generated (status: 200)");
        Ok(mock_response)
    }

    /// Execute request with retry logic and fallback handling
    pub async fn execute_request_with_retry(
        &self,
        request: RepeaterExecutionRequest,
        max_retries: u32,
    ) -> AttackResult<RepeaterExecutionResponse> {
        info!("🔄 Executing repeater request with retry (max: {})", max_retries);

        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                info!("   🔄 Retry attempt {} of {}", attempt, max_retries);
                
                // Exponential backoff
                let delay = std::time::Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                tokio::time::sleep(delay).await;
            }

            match self.execute_request(request.clone()).await {
                Ok(response) => {
                    if attempt > 0 {
                        info!("   ✓ Request succeeded on retry attempt {}", attempt);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    match &e {
                        AttackError::AgentUnavailable { agent_id } => {
                            warn!("   ⚠ Agent {} unavailable on attempt {}", agent_id, attempt + 1);
                            
                            // Try to find alternative agent if this is not the last attempt
                            if attempt < max_retries {
                                if let Some(alternative_agent) = self.find_alternative_agent(&request.target_agent_id).await {
                                    info!("   🔄 Switching to alternative agent: {}", alternative_agent);
                                    let mut retry_request = request.clone();
                                    let alternative_agent_clone = alternative_agent.clone();
                                    retry_request.target_agent_id = alternative_agent;
                                    
                                    match self.execute_request(retry_request).await {
                                        Ok(mut response) => {
                                            response.agent_id = alternative_agent_clone;
                                            info!("   ✓ Request succeeded with alternative agent");
                                            return Ok(response);
                                        }
                                        Err(alt_error) => {
                                            warn!("   ⚠ Alternative agent also failed: {}", alt_error);
                                            last_error = Some(alt_error);
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        AttackError::NetworkError { details } => {
                            warn!("   ⚠ Network error on attempt {}: {}", attempt + 1, details);
                        }
                        _ => {
                            error!("   ✗ Non-retryable error: {}", e);
                            return Err(e);
                        }
                    }
                    
                    last_error = Some(e);
                }
            }
        }

        error!("   ✗ All retry attempts failed");
        Err(last_error.unwrap_or_else(|| AttackError::ExecutionFailed {
            error: "Maximum retry attempts exceeded".to_string(),
        }))
    }

    /// Find an alternative agent when the primary agent fails
    async fn find_alternative_agent(&self, failed_agent_id: &str) -> Option<String> {
        info!("🔍 Looking for alternative agent to replace: {}", failed_agent_id);

        let available_agents = self.get_available_agents().await;
        
        // Find online agents excluding the failed one
        let alternatives: Vec<_> = available_agents
            .into_iter()
            .filter(|agent| agent.is_available && agent.id != failed_agent_id)
            .collect();

        if alternatives.is_empty() {
            warn!("   ⚠ No alternative agents available");
            return None;
        }

        // Prefer agents with better response times (if available)
        let best_agent = alternatives
            .into_iter()
            .min_by_key(|agent| agent.response_time_ms.unwrap_or(u64::MAX));

        if let Some(agent) = best_agent {
            info!("   ✓ Found alternative agent: {} ({}:{})", 
                  agent.id, agent.address, agent.port);
            Some(agent.id)
        } else {
            None
        }
    }

    /// Validate agent availability with detailed error reporting
    async fn validate_agent_availability_internal(&self, agent_id: &str) -> AttackResult<()> {
        let agents = self.agent_registry.list_agents();
        
        match agents.iter().find(|a| a.id == agent_id) {
            Some(agent) if agent.status == "Online" => {
                // Additional health check - verify last heartbeat is recent
                if let Ok(last_heartbeat) = chrono::DateTime::parse_from_rfc3339(&agent.last_heartbeat) {
                    let now = chrono::Utc::now();
                    let heartbeat_age = now.signed_duration_since(last_heartbeat.with_timezone(&chrono::Utc));
                    
                    if heartbeat_age.num_seconds() > 60 {
                        warn!("   ⚠ Agent {} heartbeat is stale ({} seconds old)", 
                              agent_id, heartbeat_age.num_seconds());
                        return Err(AttackError::AgentUnavailable {
                            agent_id: agent_id.to_string(),
                        });
                    }
                }
                
                info!("   ✓ Agent {} is available and healthy", agent_id);
                Ok(())
            }
            Some(agent) => {
                let error_msg = match agent.status.as_str() {
                    "Offline" => format!("Agent {} is offline", agent_id),
                    "Connecting" => format!("Agent {} is still connecting", agent_id),
                    "Error" => format!("Agent {} is in error state", agent_id),
                    _ => format!("Agent {} is not available (status: {})", agent_id, agent.status),
                };
                
                warn!("   ⚠ {}", error_msg);
                Err(AttackError::AgentUnavailable {
                    agent_id: agent_id.to_string(),
                })
            }
            None => {
                let error_msg = format!("Agent {} not found in registry", agent_id);
                warn!("   ⚠ {}", error_msg);
                Err(AttackError::AgentUnavailable {
                    agent_id: agent_id.to_string(),
                })
            }
        }
    }

    /// Get agent health status with detailed information
    pub async fn get_agent_health(&self, agent_id: &str) -> Option<AgentHealthStatus> {
        let agents = self.agent_registry.list_agents();
        
        if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
            let last_heartbeat = chrono::DateTime::parse_from_rfc3339(&agent.last_heartbeat)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            
            let now = chrono::Utc::now();
            let heartbeat_age = now.signed_duration_since(last_heartbeat);
            
            let health_status = if agent.status == "Online" && heartbeat_age.num_seconds() <= 60 {
                HealthStatus::Healthy
            } else if agent.status == "Online" && heartbeat_age.num_seconds() <= 300 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Unhealthy
            };

            Some(AgentHealthStatus {
                agent_id: agent.id.clone(),
                status: health_status,
                last_heartbeat,
                heartbeat_age_seconds: heartbeat_age.num_seconds(),
                capabilities: agent.capabilities.clone(),
                address: format!("{}:{}", agent.address, agent.port),
            })
        } else {
            None
        }
    }

    /// Get comprehensive error information for user notification
    pub fn get_error_details(&self, error: &AttackError) -> ErrorDetails {
        match error {
            AttackError::AgentUnavailable { agent_id } => ErrorDetails {
                error_type: "Agent Unavailable".to_string(),
                message: format!("Agent '{}' is not available for request execution", agent_id),
                remediation: vec![
                    "Check if the agent is online in the Agents panel".to_string(),
                    "Try selecting a different agent".to_string(),
                    "Wait for the agent to come back online".to_string(),
                    "Contact your system administrator if the problem persists".to_string(),
                ],
                is_retryable: true,
            },
            AttackError::NetworkError { details } => ErrorDetails {
                error_type: "Network Error".to_string(),
                message: format!("Network communication failed: {}", details),
                remediation: vec![
                    "Check your network connection".to_string(),
                    "Verify the agent is reachable".to_string(),
                    "Try again in a few moments".to_string(),
                    "Check firewall settings if the problem persists".to_string(),
                ],
                is_retryable: true,
            },
            AttackError::InvalidPayloadConfig { reason } => ErrorDetails {
                error_type: "Invalid Request".to_string(),
                message: format!("Request configuration is invalid: {}", reason),
                remediation: vec![
                    "Review the request URL and method".to_string(),
                    "Check header formatting".to_string(),
                    "Validate request body content".to_string(),
                    "Ensure all required fields are filled".to_string(),
                ],
                is_retryable: false,
            },
            AttackError::SessionExpired { session_id } => ErrorDetails {
                error_type: "Session Expired".to_string(),
                message: format!("Session '{}' has expired or is invalid", session_id),
                remediation: vec![
                    "Refresh the session using LSR".to_string(),
                    "Select a different active session".to_string(),
                    "Remove session data and try without authentication".to_string(),
                ],
                is_retryable: false,
            },
            _ => ErrorDetails {
                error_type: "Unknown Error".to_string(),
                message: error.to_string(),
                remediation: vec![
                    "Try the request again".to_string(),
                    "Check the application logs for more details".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                is_retryable: true,
            },
        }
    }

    /// Get execution statistics for a tab
    pub async fn get_execution_statistics(&self, tab_id: &str) -> AttackResult<ExecutionStatistics> {
        info!("📊 Getting execution statistics for tab: {}", tab_id);

        let executions: Vec<RepeaterExecutionResponse> = self.get_execution_history(tab_id, None).await?;
        
        let total_executions = executions.len();
        let successful_executions = executions.iter()
            .filter(|e| e.error.is_none() && e.status_code.map_or(false, |s| s < 400))
            .count();
        
        let average_duration = if !executions.is_empty() {
            executions.iter()
                .filter_map(|e| e.duration_ms)
                .sum::<u64>() as f64 / executions.len() as f64
        } else {
            0.0
        };

        let status_code_distribution: HashMap<i32, usize> = executions.iter()
            .filter_map(|e| e.status_code)
            .fold(HashMap::new(), |mut acc: HashMap<i32, usize>, status| {
                *acc.entry(status).or_insert(0) += 1;
                acc
            });

        let stats = ExecutionStatistics {
            total_executions,
            successful_executions,
            error_count: total_executions - successful_executions,
            average_duration_ms: average_duration,
            status_code_distribution,
            last_execution: executions.first().map(|e| e.executed_at),
        };

        info!("   ✓ Statistics: {} total, {} successful, {:.1}ms avg", 
              stats.total_executions, stats.successful_executions, stats.average_duration_ms);
        
        Ok(stats)
    }

    /// Validate request template
    fn validate_request_template(&self, request: &HttpRequestData) -> AttackResult<()> {
        // Validate URL
        if request.url.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Request URL cannot be empty".to_string(),
            });
        }

        // Validate URL format
        if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Request URL must start with http:// or https://".to_string(),
            });
        }

        // Validate method
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&request.method.as_str()) {
            return Err(AttackError::InvalidPayloadConfig {
                reason: format!("Invalid HTTP method: {}", request.method),
            });
        }

        info!("   ✓ Request template validation passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_request() -> HttpRequestData {
        HttpRequestData {
            url: "https://example.com/api/test".to_string(),
            method: "GET".to_string(),
            headers: Some(attack_engine::HttpHeaders {
                headers: HashMap::new(),
            }),
            body: Vec::new(),
            tls: None,
        }
    }

    #[tokio::test]
    async fn test_validate_request_template() {
        let db = Arc::new(Database::new("test").await.unwrap());
        let registry = Arc::new(AgentRegistry::new());
        let manager = RepeaterManager::new(db, registry);

        // Valid request
        let valid_request = create_test_request();
        assert!(manager.validate_request_template(&valid_request).is_ok());

        // Invalid URL - empty
        let mut invalid_request = create_test_request();
        invalid_request.url = "".to_string();
        assert!(manager.validate_request_template(&invalid_request).is_err());

        // Invalid URL - no protocol
        invalid_request.url = "example.com".to_string();
        assert!(manager.validate_request_template(&invalid_request).is_err());

        // Invalid method
        invalid_request.url = "https://example.com".to_string();
        invalid_request.method = "INVALID".to_string();
        assert!(manager.validate_request_template(&invalid_request).is_err());
    }

    #[tokio::test]
    async fn test_session_management() {
        let db = Arc::new(Database::new("test").await.unwrap());
        let registry = Arc::new(AgentRegistry::new());
        let manager = RepeaterManager::new(db, registry);

        // Create test session
        let session = Session::new("Test Session".to_string(), None);
        let session_id = session.id;

        // Add session
        manager.add_session(session).await;

        // Retrieve session
        let retrieved: Option<Session> = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Session");

        // Get all sessions
        let sessions: Vec<Session> = manager.get_sessions().await;
        assert_eq!(sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_apply_session_to_request() {
        let db = Arc::new(Database::new("test").await.unwrap());
        let registry = Arc::new(AgentRegistry::new());
        let manager = RepeaterManager::new(db, registry);

        // Create session with headers
        let mut session = Session::new("Test Session".to_string(), None);
        session.headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        let session_id = session.id;
        manager.add_session(session).await;

        // Create request
        let request = create_test_request();

        // Apply session
        let result: AttackResult<(HttpRequestData, SessionApplicationResult)> = manager.apply_session_to_request(request, &session_id, Some(ExpirationHandling::Fail)).await;
        assert!(result.is_ok());
        let (modified_request, _session_result) = result.unwrap();
        assert!(modified_request.headers.is_some());
        
        let headers = modified_request.headers.unwrap();
        assert_eq!(
            headers.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }
}