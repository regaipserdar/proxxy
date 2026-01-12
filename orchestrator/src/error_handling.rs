//! Comprehensive error handling for the orchestrator
//! 
//! This module provides centralized error handling, logging, and recovery
//! mechanisms for all orchestrator operations including repeater and intruder.

use attack_engine::{
    AttackError, AttackResult, ErrorRecoveryStrategy, BackoffStrategy, 
    ErrorSeverity, ErrorCategory, CircuitBreaker, ErrorContext, SecurityManager
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{error, warn, info, debug};
use uuid::Uuid;

/// Centralized error handler for the orchestrator
pub struct ErrorHandler {
    /// Security manager for sensitive data masking
    security_manager: Arc<SecurityManager>,
    
    /// Circuit breakers for different components
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    
    /// Error recovery strategies by component
    recovery_strategies: Arc<RwLock<HashMap<String, ErrorRecoveryStrategy>>>,
    
    /// Error event broadcaster
    error_broadcaster: broadcast::Sender<ErrorEvent>,
    
    /// Error statistics tracking
    error_stats: Arc<RwLock<ErrorStatistics>>,
    
    /// Agent failure tracking for quick failure detection
    agent_failures: Arc<RwLock<HashMap<String, AgentFailureTracker>>>,
    
    /// Resource exhaustion monitoring
    resource_monitor: Arc<RwLock<ResourceExhaustionMonitor>>,
}

/// Error event for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error_type: String,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub component: String,
    pub operation: String,
    pub message: String,
    pub remediation: String,
    pub agent_id: Option<String>,
    pub request_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub is_recoverable: bool,
    pub retry_count: u32,
}

/// Error statistics for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: u64,
    pub errors_by_severity: HashMap<String, u64>,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_component: HashMap<String, u64>,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: f64,
    pub circuit_breaker_trips: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Agent failure tracking for quick failure detection
#[derive(Debug, Clone)]
struct AgentFailureTracker {
    consecutive_failures: u32,
    last_failure_time: chrono::DateTime<chrono::Utc>,
    failure_rate: f64,
    is_marked_unhealthy: bool,
    recovery_attempts: u32,
}

/// Resource exhaustion monitoring
#[derive(Debug, Clone)]
struct ResourceExhaustionMonitor {
    memory_usage_mb: f64,
    cpu_usage_percent: f64,
    disk_usage_percent: f64,
    network_connections: u32,
    concurrent_requests: u32,
    last_check: chrono::DateTime<chrono::Utc>,
}

/// Input validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Input validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_code: String,
    pub suggested_fix: String,
}

/// Graceful degradation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationConfig {
    pub enable_degradation: bool,
    pub memory_threshold_mb: f64,
    pub cpu_threshold_percent: f64,
    pub max_concurrent_requests: u32,
    pub reduced_functionality_mode: bool,
    pub emergency_shutdown_threshold: f64,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            enable_degradation: true,
            memory_threshold_mb: 1024.0, // 1GB
            cpu_threshold_percent: 80.0,
            max_concurrent_requests: 100,
            reduced_functionality_mode: false,
            emergency_shutdown_threshold: 95.0,
        }
    }
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        let (error_broadcaster, _) = broadcast::channel(1000);
        
        Self {
            security_manager: Arc::new(SecurityManager::new()),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            recovery_strategies: Arc::new(RwLock::new(HashMap::new())),
            error_broadcaster,
            error_stats: Arc::new(RwLock::new(ErrorStatistics::default())),
            agent_failures: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor: Arc::new(RwLock::new(ResourceExhaustionMonitor::default())),
        }
    }
    
    /// Handle an error with full context and recovery
    pub async fn handle_error(
        &self,
        error: AttackError,
        component: &str,
        operation: &str,
        context: Option<ErrorContext>,
    ) -> ErrorHandlingResult {
        let error_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();
        
        // Create error context if not provided
        let error_context = context.unwrap_or_else(|| {
            ErrorContext::new(error.clone(), operation, component)
        });
        
        // Log error with masked sensitive data
        let masked_message = self.security_manager.mask_text(&error_context.formatted_message());
        match error.severity() {
            ErrorSeverity::Critical => error!("🚨 CRITICAL ERROR [{}]: {}", error_id, masked_message),
            ErrorSeverity::High => error!("❌ HIGH SEVERITY ERROR [{}]: {}", error_id, masked_message),
            ErrorSeverity::Medium => warn!("⚠️ MEDIUM SEVERITY ERROR [{}]: {}", error_id, masked_message),
            ErrorSeverity::Low => info!("ℹ️ LOW SEVERITY ERROR [{}]: {}", error_id, masked_message),
        }
        
        // Update error statistics
        self.update_error_statistics(&error).await;
        
        // Check circuit breaker
        let should_fail_fast = self.check_circuit_breaker(component, &error).await;
        if should_fail_fast {
            warn!("🔌 Circuit breaker OPEN for component '{}' - failing fast", component);
            return ErrorHandlingResult {
                should_retry: false,
                retry_delay_ms: None,
                fallback_action: Some(FallbackAction::FailFast),
                degradation_applied: false,
                error_id,
            };
        }
        
        // Determine recovery strategy
        let recovery_strategy = self.get_recovery_strategy(component).await;
        let should_retry = error.is_recoverable() && recovery_strategy.max_retries > 0;
        
        // Handle agent-specific failures
        if let Some(agent_id) = &error_context.agent_id {
            self.handle_agent_failure(agent_id, &error).await;
        }
        
        // Check for resource exhaustion and apply degradation
        let degradation_applied = self.check_and_apply_degradation(&error).await;
        
        // Broadcast error event
        let error_event = ErrorEvent {
            id: error_id,
            timestamp,
            error_type: format!("{:?}", error),
            severity: error.severity(),
            category: error.category(),
            component: component.to_string(),
            operation: operation.to_string(),
            message: self.security_manager.mask_text(&error.to_string()),
            remediation: error.remediation(),
            agent_id: error_context.agent_id.clone(),
            request_id: error_context.request_id,
            session_id: error_context.session_id,
            is_recoverable: error.is_recoverable(),
            retry_count: 0, // Will be updated by retry logic
        };
        
        let _ = self.error_broadcaster.send(error_event);
        
        // Calculate retry delay
        let retry_delay_ms = if should_retry {
            Some(recovery_strategy.backoff_strategy.calculate_delay(0))
        } else {
            None
        };
        
        // Determine fallback action
        let fallback_action = self.determine_fallback_action(&error, &recovery_strategy).await;
        
        ErrorHandlingResult {
            should_retry,
            retry_delay_ms,
            fallback_action,
            degradation_applied,
            error_id,
        }
    }
    
    /// Validate input data with comprehensive checks
    pub async fn validate_input<T>(&self, input: &T, validator_name: &str) -> ValidationResult
    where
        T: serde::Serialize,
    {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Serialize input for validation
        let input_json = match serde_json::to_string(input) {
            Ok(json) => json,
            Err(e) => {
                errors.push(ValidationError {
                    field: "input".to_string(),
                    message: format!("Failed to serialize input: {}", e),
                    error_code: "SERIALIZATION_ERROR".to_string(),
                    suggested_fix: "Check input data structure and ensure all fields are serializable".to_string(),
                });
                return ValidationResult {
                    is_valid: false,
                    errors,
                    warnings,
                };
            }
        };
        
        // Check for sensitive data in input
        if let Err(violation) = self.security_manager.validate_masked_output(&input_json) {
            warnings.push(format!("Potential sensitive data detected: {}", violation.description));
        }
        
        // Perform validator-specific checks
        match validator_name {
            "repeater_request" => self.validate_repeater_request(&input_json, &mut errors, &mut warnings).await,
            "intruder_attack" => self.validate_intruder_attack(&input_json, &mut errors, &mut warnings).await,
            "payload_config" => self.validate_payload_config(&input_json, &mut errors, &mut warnings).await,
            "session_data" => self.validate_session_data(&input_json, &mut errors, &mut warnings).await,
            _ => {
                warnings.push(format!("Unknown validator '{}' - performing basic validation only", validator_name));
            }
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    
    /// Detect agent failures quickly
    pub async fn detect_agent_failure(&self, agent_id: &str, error: &AttackError) -> bool {
        let mut agent_failures = self.agent_failures.write().await;
        let tracker = agent_failures.entry(agent_id.to_string()).or_insert_with(|| {
            AgentFailureTracker {
                consecutive_failures: 0,
                last_failure_time: chrono::Utc::now(),
                failure_rate: 0.0,
                is_marked_unhealthy: false,
                recovery_attempts: 0,
            }
        });
        
        // Update failure tracking
        tracker.consecutive_failures += 1;
        tracker.last_failure_time = chrono::Utc::now();
        
        // Calculate failure rate (failures per minute)
        let time_window_minutes = 5.0;
        tracker.failure_rate = tracker.consecutive_failures as f64 / time_window_minutes;
        
        // Quick failure detection thresholds
        let quick_failure_threshold = 3;
        let high_failure_rate_threshold = 2.0; // 2 failures per minute
        
        let is_quick_failure = tracker.consecutive_failures >= quick_failure_threshold ||
                              tracker.failure_rate >= high_failure_rate_threshold;
        
        if is_quick_failure && !tracker.is_marked_unhealthy {
            warn!("🚨 Quick failure detected for agent '{}': {} consecutive failures, rate: {:.2}/min", 
                  agent_id, tracker.consecutive_failures, tracker.failure_rate);
            tracker.is_marked_unhealthy = true;
        }
        
        is_quick_failure
    }
    
    /// Apply graceful degradation under resource exhaustion
    pub async fn apply_graceful_degradation(&self, config: &DegradationConfig) -> bool {
        if !config.enable_degradation {
            return false;
        }
        
        let mut resource_monitor = self.resource_monitor.write().await;
        
        // Update resource usage (in a real implementation, this would query system metrics)
        resource_monitor.last_check = chrono::Utc::now();
        
        // Check if degradation is needed
        let needs_degradation = resource_monitor.memory_usage_mb > config.memory_threshold_mb ||
                               resource_monitor.cpu_usage_percent > config.cpu_threshold_percent ||
                               resource_monitor.concurrent_requests > config.max_concurrent_requests;
        
        if needs_degradation {
            warn!("🔧 Applying graceful degradation: Memory: {:.1}MB, CPU: {:.1}%, Requests: {}", 
                  resource_monitor.memory_usage_mb, 
                  resource_monitor.cpu_usage_percent,
                  resource_monitor.concurrent_requests);
            
            // Apply degradation measures
            if resource_monitor.memory_usage_mb > config.emergency_shutdown_threshold {
                error!("🚨 EMERGENCY: Memory usage critical - initiating emergency procedures");
                // In a real implementation, this would trigger emergency shutdown
            }
            
            return true;
        }
        
        false
    }
    
    /// Get error statistics for monitoring
    pub async fn get_error_statistics(&self) -> ErrorStatistics {
        self.error_stats.read().await.clone()
    }
    
    /// Subscribe to error events
    pub fn subscribe_to_errors(&self) -> broadcast::Receiver<ErrorEvent> {
        self.error_broadcaster.subscribe()
    }
    
    /// Update recovery strategy for a component
    pub async fn update_recovery_strategy(&self, component: &str, strategy: ErrorRecoveryStrategy) {
        self.recovery_strategies.write().await.insert(component.to_string(), strategy);
        info!("🔧 Updated recovery strategy for component '{}'", component);
    }
    
    /// Reset circuit breaker for a component
    pub async fn reset_circuit_breaker(&self, component: &str) {
        if let Some(breaker) = self.circuit_breakers.write().await.get_mut(component) {
            breaker.record_success();
            info!("🔌 Reset circuit breaker for component '{}'", component);
        }
    }
    
    /// Mark agent as healthy (reset failure tracking)
    pub async fn mark_agent_healthy(&self, agent_id: &str) {
        if let Some(tracker) = self.agent_failures.write().await.get_mut(agent_id) {
            tracker.consecutive_failures = 0;
            tracker.failure_rate = 0.0;
            tracker.is_marked_unhealthy = false;
            tracker.recovery_attempts = 0;
            info!("✅ Marked agent '{}' as healthy", agent_id);
        }
    }
    
    // Private helper methods
    
    async fn update_error_statistics(&self, error: &AttackError) {
        let mut stats = self.error_stats.write().await;
        stats.total_errors += 1;
        
        let severity_key = format!("{:?}", error.severity());
        *stats.errors_by_severity.entry(severity_key).or_insert(0) += 1;
        
        let category_key = format!("{:?}", error.category());
        *stats.errors_by_category.entry(category_key).or_insert(0) += 1;
        
        stats.last_updated = chrono::Utc::now();
    }
    
    async fn check_circuit_breaker(&self, component: &str, error: &AttackError) -> bool {
        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers.entry(component.to_string()).or_insert_with(|| {
            CircuitBreaker::new(5, 60000) // 5 failures, 60 second timeout
        });
        
        if !breaker.can_execute() {
            return true;
        }
        
        // Record failure for certain error types
        match error {
            AttackError::AgentUnavailable { .. } |
            AttackError::NetworkError { .. } |
            AttackError::Timeout { .. } |
            AttackError::ExecutionFailed { .. } => {
                breaker.record_failure();
            }
            _ => {
                // Don't count configuration errors, etc. as circuit breaker failures
            }
        }
        
        false
    }
    
    async fn get_recovery_strategy(&self, component: &str) -> ErrorRecoveryStrategy {
        self.recovery_strategies.read().await
            .get(component)
            .cloned()
            .unwrap_or_default()
    }
    
    async fn handle_agent_failure(&self, agent_id: &str, error: &AttackError) {
        let is_quick_failure = self.detect_agent_failure(agent_id, error).await;
        
        if is_quick_failure {
            // Broadcast agent failure event
            let error_event = ErrorEvent {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                error_type: "AgentQuickFailure".to_string(),
                severity: ErrorSeverity::High,
                category: ErrorCategory::Infrastructure,
                component: "AgentManager".to_string(),
                operation: "agent_health_check".to_string(),
                message: format!("Agent '{}' experiencing quick failures", agent_id),
                remediation: format!("Check agent '{}' connectivity and health. Consider removing from active pool.", agent_id),
                agent_id: Some(agent_id.to_string()),
                request_id: None,
                session_id: None,
                is_recoverable: true,
                retry_count: 0,
            };
            
            let _ = self.error_broadcaster.send(error_event);
        }
    }
    
    async fn check_and_apply_degradation(&self, error: &AttackError) -> bool {
        match error {
            AttackError::ResourceExhaustion { .. } => {
                let config = DegradationConfig::default();
                self.apply_graceful_degradation(&config).await
            }
            _ => false,
        }
    }
    
    async fn determine_fallback_action(&self, error: &AttackError, strategy: &ErrorRecoveryStrategy) -> Option<FallbackAction> {
        match error {
            AttackError::AgentUnavailable { .. } if !strategy.fallback_agents.is_empty() => {
                Some(FallbackAction::UseFallbackAgent {
                    agent_ids: strategy.fallback_agents.clone(),
                })
            }
            AttackError::ResourceExhaustion { .. } => {
                Some(FallbackAction::ReduceLoad)
            }
            AttackError::RateLimitExceeded { retry_after_ms, .. } => {
                Some(FallbackAction::BackoffAndRetry {
                    delay_ms: retry_after_ms.unwrap_or(60000),
                })
            }
            _ => None,
        }
    }
    
    async fn validate_repeater_request(&self, _input: &str, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<String>) {
        // Implement repeater-specific validation
        // This is a placeholder - real implementation would validate request structure
        if _input.is_empty() {
            errors.push(ValidationError {
                field: "request".to_string(),
                message: "Request cannot be empty".to_string(),
                error_code: "EMPTY_REQUEST".to_string(),
                suggested_fix: "Provide a valid HTTP request".to_string(),
            });
        }
    }
    
    async fn validate_intruder_attack(&self, _input: &str, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<String>) {
        // Implement intruder-specific validation
        // This is a placeholder - real implementation would validate attack configuration
        if _input.is_empty() {
            errors.push(ValidationError {
                field: "attack_config".to_string(),
                message: "Attack configuration cannot be empty".to_string(),
                error_code: "EMPTY_ATTACK_CONFIG".to_string(),
                suggested_fix: "Provide a valid attack configuration".to_string(),
            });
        }
    }
    
    async fn validate_payload_config(&self, _input: &str, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<String>) {
        // Implement payload-specific validation
        // This is a placeholder - real implementation would validate payload configuration
        if _input.is_empty() {
            errors.push(ValidationError {
                field: "payload_config".to_string(),
                message: "Payload configuration cannot be empty".to_string(),
                error_code: "EMPTY_PAYLOAD_CONFIG".to_string(),
                suggested_fix: "Provide a valid payload configuration".to_string(),
            });
        }
    }
    
    async fn validate_session_data(&self, _input: &str, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<String>) {
        // Implement session-specific validation
        // This is a placeholder - real implementation would validate session data
        if _input.is_empty() {
            errors.push(ValidationError {
                field: "session_data".to_string(),
                message: "Session data cannot be empty".to_string(),
                error_code: "EMPTY_SESSION_DATA".to_string(),
                suggested_fix: "Provide valid session data".to_string(),
            });
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ErrorStatistics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_severity: HashMap::new(),
            errors_by_category: HashMap::new(),
            errors_by_component: HashMap::new(),
            recovery_success_rate: 0.0,
            average_recovery_time_ms: 0.0,
            circuit_breaker_trips: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

impl Default for ResourceExhaustionMonitor {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            disk_usage_percent: 0.0,
            network_connections: 0,
            concurrent_requests: 0,
            last_check: chrono::Utc::now(),
        }
    }
}

/// Result of error handling operation
#[derive(Debug, Clone)]
pub struct ErrorHandlingResult {
    pub should_retry: bool,
    pub retry_delay_ms: Option<u64>,
    pub fallback_action: Option<FallbackAction>,
    pub degradation_applied: bool,
    pub error_id: Uuid,
}

/// Fallback actions for error recovery
#[derive(Debug, Clone)]
pub enum FallbackAction {
    FailFast,
    UseFallbackAgent { agent_ids: Vec<String> },
    ReduceLoad,
    BackoffAndRetry { delay_ms: u64 },
    SwitchToOfflineMode,
    NotifyUser { message: String },
}