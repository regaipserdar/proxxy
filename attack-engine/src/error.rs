//! Error types for the attack engine

use thiserror::Error;
use uuid::Uuid;

/// Main error type for attack engine operations
#[derive(Debug, Error, Clone, serde::Serialize, serde::Deserialize)]
pub enum AttackError {
    #[error("Agent not available: {agent_id}")]
    AgentUnavailable { agent_id: String },
    
    #[error("Invalid payload configuration: {reason}")]
    InvalidPayloadConfig { reason: String },
    
    #[error("Request execution failed: {error}")]
    ExecutionFailed { error: String },
    
    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: Uuid },
    
    #[error("Payload generation failed: {reason}")]
    PayloadGenerationFailed { reason: String },
    
    #[error("Database operation failed: {operation}")]
    DatabaseError { operation: String },
    
    #[error("Network error: {details}")]
    NetworkError { details: String },
    
    #[error("Resource allocation failed: {reason}")]
    ResourceAllocationFailed { reason: String },
    
    #[error("Attack configuration invalid: {reason}")]
    InvalidAttackConfig { reason: String },
    
    #[error("Serialization error: {error}")]
    SerializationError { error: String },
    
    #[error("Input validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Resource exhaustion: {resource_type} - {details}")]
    ResourceExhaustion { resource_type: String, details: String },
    
    #[error("Authentication failure: {reason}")]
    AuthenticationFailure { reason: String },
    
    #[error("Permission denied: {operation} - {reason}")]
    PermissionDenied { operation: String, reason: String },
    
    #[error("Timeout occurred: {operation} after {duration_ms}ms")]
    Timeout { operation: String, duration_ms: u64 },
    
    #[error("Rate limit exceeded: {limit_type} - {retry_after_ms:?}ms")]
    RateLimitExceeded { limit_type: String, retry_after_ms: Option<u64> },
    
    #[error("Configuration error: {component} - {reason}")]
    ConfigurationError { component: String, reason: String },
    
    #[error("Security violation: {violation_type} - {details}")]
    SecurityViolation { violation_type: String, details: String },
}

impl AttackError {
    /// Create a validation error with field and reason
    pub fn validation(field: &str, reason: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }
    
    /// Create a resource exhaustion error
    pub fn resource_exhaustion(resource_type: &str, details: &str) -> Self {
        Self::ResourceExhaustion {
            resource_type: resource_type.to_string(),
            details: details.to_string(),
        }
    }
    
    /// Create a timeout error
    pub fn timeout(operation: &str, duration_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.to_string(),
            duration_ms,
        }
    }
    
    /// Create a rate limit error
    pub fn rate_limit(limit_type: &str, retry_after_ms: Option<u64>) -> Self {
        Self::RateLimitExceeded {
            limit_type: limit_type.to_string(),
            retry_after_ms,
        }
    }
    
    /// Create a configuration error
    pub fn configuration(component: &str, reason: &str) -> Self {
        Self::ConfigurationError {
            component: component.to_string(),
            reason: reason.to_string(),
        }
    }
    
    /// Create a security violation error
    pub fn security_violation(violation_type: &str, details: &str) -> Self {
        Self::SecurityViolation {
            violation_type: violation_type.to_string(),
            details: details.to_string(),
        }
    }
    
    /// Check if the error is recoverable (can be retried)
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors - can be retried
            AttackError::NetworkError { .. } => true,
            AttackError::AgentUnavailable { .. } => true,
            AttackError::Timeout { .. } => true,
            AttackError::RateLimitExceeded { .. } => true,
            AttackError::ResourceExhaustion { .. } => true,
            AttackError::ExecutionFailed { .. } => true,
            
            // Non-recoverable errors - should not be retried
            AttackError::InvalidPayloadConfig { .. } => false,
            AttackError::SessionExpired { .. } => false,
            AttackError::ValidationError { .. } => false,
            AttackError::AuthenticationFailure { .. } => false,
            AttackError::PermissionDenied { .. } => false,
            AttackError::ConfigurationError { .. } => false,
            AttackError::SecurityViolation { .. } => false,
            AttackError::InvalidAttackConfig { .. } => false,
            
            // Context-dependent errors
            AttackError::PayloadGenerationFailed { .. } => false,
            AttackError::DatabaseError { .. } => true,
            AttackError::ResourceAllocationFailed { .. } => true,
            AttackError::SerializationError { .. } => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AttackError::SecurityViolation { .. } => ErrorSeverity::Critical,
            AttackError::PermissionDenied { .. } => ErrorSeverity::Critical,
            AttackError::AuthenticationFailure { .. } => ErrorSeverity::High,
            AttackError::ConfigurationError { .. } => ErrorSeverity::High,
            AttackError::InvalidAttackConfig { .. } => ErrorSeverity::High,
            AttackError::ResourceExhaustion { .. } => ErrorSeverity::High,
            AttackError::DatabaseError { .. } => ErrorSeverity::Medium,
            AttackError::NetworkError { .. } => ErrorSeverity::Medium,
            AttackError::Timeout { .. } => ErrorSeverity::Medium,
            AttackError::RateLimitExceeded { .. } => ErrorSeverity::Medium,
            AttackError::AgentUnavailable { .. } => ErrorSeverity::Low,
            AttackError::ExecutionFailed { .. } => ErrorSeverity::Low,
            AttackError::ValidationError { .. } => ErrorSeverity::Low,
            AttackError::InvalidPayloadConfig { .. } => ErrorSeverity::Low,
            AttackError::SessionExpired { .. } => ErrorSeverity::Low,
            AttackError::PayloadGenerationFailed { .. } => ErrorSeverity::Low,
            AttackError::ResourceAllocationFailed { .. } => ErrorSeverity::Low,
            AttackError::SerializationError { .. } => ErrorSeverity::Low,
        }
    }
    
    /// Get suggested remediation for the error
    pub fn remediation(&self) -> String {
        match self {
            AttackError::AgentUnavailable { agent_id } => {
                format!("Check agent '{}' status and network connectivity. Consider using a different agent or waiting for the agent to come online.", agent_id)
            }
            AttackError::InvalidPayloadConfig { reason } => {
                format!("Fix payload configuration: {}. Check payload format, file paths, and parameter values.", reason)
            }
            AttackError::ExecutionFailed { error } => {
                format!("Request execution failed: {}. Check target URL, network connectivity, and request format.", error)
            }
            AttackError::SessionExpired { session_id } => {
                format!("Session '{}' has expired. Refresh the session or use a different active session.", session_id)
            }
            AttackError::PayloadGenerationFailed { reason } => {
                format!("Payload generation failed: {}. Check payload configuration, file permissions, and available memory.", reason)
            }
            AttackError::DatabaseError { operation } => {
                format!("Database operation '{}' failed. Check database connectivity, permissions, and available disk space.", operation)
            }
            AttackError::NetworkError { details } => {
                format!("Network error: {}. Check internet connectivity, DNS resolution, and firewall settings.", details)
            }
            AttackError::ResourceAllocationFailed { reason } => {
                format!("Resource allocation failed: {}. Reduce concurrent operations or increase system resources.", reason)
            }
            AttackError::InvalidAttackConfig { reason } => {
                format!("Attack configuration invalid: {}. Review attack parameters and fix configuration errors.", reason)
            }
            AttackError::SerializationError { error } => {
                format!("Data serialization failed: {}. Check data format and encoding.", error)
            }
            AttackError::ValidationError { field, reason } => {
                format!("Input validation failed for '{}': {}. Correct the input and try again.", field, reason)
            }
            AttackError::ResourceExhaustion { resource_type, details } => {
                format!("Resource exhaustion ({}): {}. Reduce load, increase limits, or wait for resources to become available.", resource_type, details)
            }
            AttackError::AuthenticationFailure { reason } => {
                format!("Authentication failed: {}. Check credentials, session validity, and authentication configuration.", reason)
            }
            AttackError::PermissionDenied { operation, reason } => {
                format!("Permission denied for '{}': {}. Check user permissions and access controls.", operation, reason)
            }
            AttackError::Timeout { operation, duration_ms } => {
                format!("Operation '{}' timed out after {}ms. Increase timeout value or check target responsiveness.", operation, duration_ms)
            }
            AttackError::RateLimitExceeded { limit_type, retry_after_ms } => {
                if let Some(retry_ms) = retry_after_ms {
                    format!("Rate limit exceeded for '{}'. Retry after {}ms or reduce request rate.", limit_type, retry_ms)
                } else {
                    format!("Rate limit exceeded for '{}'. Reduce request rate and try again later.", limit_type)
                }
            }
            AttackError::ConfigurationError { component, reason } => {
                format!("Configuration error in '{}': {}. Check configuration file and fix the error.", component, reason)
            }
            AttackError::SecurityViolation { violation_type, details } => {
                format!("Security violation ({}): {}. Review security policies and fix the violation immediately.", violation_type, details)
            }
        }
    }
    
    /// Get error category for grouping and filtering
    pub fn category(&self) -> ErrorCategory {
        match self {
            AttackError::AgentUnavailable { .. } => ErrorCategory::Infrastructure,
            AttackError::NetworkError { .. } => ErrorCategory::Infrastructure,
            AttackError::DatabaseError { .. } => ErrorCategory::Infrastructure,
            AttackError::ResourceAllocationFailed { .. } => ErrorCategory::Infrastructure,
            AttackError::ResourceExhaustion { .. } => ErrorCategory::Infrastructure,
            AttackError::Timeout { .. } => ErrorCategory::Infrastructure,
            
            AttackError::InvalidPayloadConfig { .. } => ErrorCategory::Configuration,
            AttackError::InvalidAttackConfig { .. } => ErrorCategory::Configuration,
            AttackError::ConfigurationError { .. } => ErrorCategory::Configuration,
            AttackError::ValidationError { .. } => ErrorCategory::Configuration,
            
            AttackError::SessionExpired { .. } => ErrorCategory::Authentication,
            AttackError::AuthenticationFailure { .. } => ErrorCategory::Authentication,
            AttackError::PermissionDenied { .. } => ErrorCategory::Authentication,
            
            AttackError::SecurityViolation { .. } => ErrorCategory::Security,
            
            AttackError::ExecutionFailed { .. } => ErrorCategory::Runtime,
            AttackError::PayloadGenerationFailed { .. } => ErrorCategory::Runtime,
            AttackError::SerializationError { .. } => ErrorCategory::Runtime,
            AttackError::RateLimitExceeded { .. } => ErrorCategory::Runtime,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error categories for grouping and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Infrastructure,
    Configuration,
    Authentication,
    Security,
    Runtime,
}

impl From<serde_json::Error> for AttackError {
    fn from(error: serde_json::Error) -> Self {
        AttackError::SerializationError {
            error: error.to_string(),
        }
    }
}

impl From<std::io::Error> for AttackError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::TimedOut => AttackError::Timeout {
                operation: "I/O operation".to_string(),
                duration_ms: 0, // Duration not available from std::io::Error
            },
            std::io::ErrorKind::PermissionDenied => AttackError::PermissionDenied {
                operation: "File system operation".to_string(),
                reason: error.to_string(),
            },
            std::io::ErrorKind::NotFound => AttackError::ConfigurationError {
                component: "File system".to_string(),
                reason: format!("File not found: {}", error),
            },
            _ => AttackError::ExecutionFailed {
                error: error.to_string(),
            },
        }
    }
}

/// Result type for attack engine operations
pub type AttackResult<T> = Result<T, AttackError>;

/// Error recovery strategy configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorRecoveryStrategy {
    pub max_retries: u32,
    pub backoff_strategy: BackoffStrategy,
    pub fallback_agents: Vec<String>,
    pub quick_failure_detection: bool,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_ms: u64,
}

impl Default for ErrorRecoveryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential {
                initial_delay_ms: 1000,
                multiplier: 2.0,
                max_delay_ms: 30000,
            },
            fallback_agents: Vec::new(),
            quick_failure_detection: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_ms: 60000,
        }
    }
}

/// Backoff strategy for retry logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BackoffStrategy {
    Fixed { delay_ms: u64 },
    Exponential { 
        initial_delay_ms: u64, 
        multiplier: f64, 
        max_delay_ms: u64 
    },
    Linear { 
        initial_delay_ms: u64, 
        increment_ms: u64 
    },
}

impl BackoffStrategy {
    /// Calculate delay for the given attempt number (0-based)
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        match self {
            BackoffStrategy::Fixed { delay_ms } => *delay_ms,
            BackoffStrategy::Exponential { 
                initial_delay_ms, 
                multiplier, 
                max_delay_ms 
            } => {
                let delay = (*initial_delay_ms as f64) * multiplier.powi(attempt as i32);
                (delay as u64).min(*max_delay_ms)
            }
            BackoffStrategy::Linear { 
                initial_delay_ms, 
                increment_ms 
            } => {
                initial_delay_ms + (increment_ms * attempt as u64)
            }
        }
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    failure_count: u32,
    threshold: u32,
    timeout_ms: u64,
    last_failure_time: Option<std::time::Instant>,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,  // Normal operation
    Open,    // Failing fast
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(threshold: u32, timeout_ms: u64) -> Self {
        Self {
            failure_count: 0,
            threshold,
            timeout_ms,
            last_failure_time: None,
            state: CircuitBreakerState::Closed,
        }
    }
    
    /// Check if operation should be allowed
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed().as_millis() > self.timeout_ms as u128 {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }
    
    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());
        
        if self.failure_count >= self.threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
    
    /// Get current state
    pub fn state(&self) -> &CircuitBreakerState {
        &self.state
    }
    
    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

/// Enhanced error context for better debugging and monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorContext {
    pub error: AttackError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub component: String,
    pub request_id: Option<uuid::Uuid>,
    pub agent_id: Option<String>,
    pub session_id: Option<uuid::Uuid>,
    pub additional_context: std::collections::HashMap<String, String>,
}

/// Input validation error
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_code: String,
    pub suggested_fix: String,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(error: AttackError, operation: &str, component: &str) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now(),
            operation: operation.to_string(),
            component: component.to_string(),
            request_id: None,
            agent_id: None,
            session_id: None,
            additional_context: std::collections::HashMap::new(),
        }
    }
    
    /// Add request ID to context
    pub fn with_request_id(mut self, request_id: uuid::Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    /// Add agent ID to context
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
    
    /// Add session ID to context
    pub fn with_session_id(mut self, session_id: uuid::Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// Add additional context information
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.additional_context.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get formatted error message with context
    pub fn formatted_message(&self) -> String {
        let mut parts = vec![
            format!("[{}] {}: {}", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"), self.component, self.operation),
            format!("Error: {}", self.error),
            format!("Severity: {:?}", self.error.severity()),
            format!("Category: {:?}", self.error.category()),
            format!("Recoverable: {}", self.error.is_recoverable()),
            format!("Remediation: {}", self.error.remediation()),
        ];
        
        if let Some(request_id) = &self.request_id {
            parts.push(format!("Request ID: {}", request_id));
        }
        
        if let Some(agent_id) = &self.agent_id {
            parts.push(format!("Agent ID: {}", agent_id));
        }
        
        if let Some(session_id) = &self.session_id {
            parts.push(format!("Session ID: {}", session_id));
        }
        
        if !self.additional_context.is_empty() {
            parts.push("Additional Context:".to_string());
            for (key, value) in &self.additional_context {
                parts.push(format!("  {}: {}", key, value));
            }
        }
        
        parts.join("\n")
    }
}