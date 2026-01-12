//! Core data types for the attack engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use proxy_common::Session;

/// HTTP request data structure compatible with protobuf definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestData {
    pub method: String,
    pub url: String,
    pub headers: Option<HttpHeaders>,
    pub body: Vec<u8>,
    pub tls: Option<TlsDetails>,
}

/// HTTP response data structure compatible with protobuf definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponseData {
    pub status_code: i32,
    pub headers: Option<HttpHeaders>,
    pub body: Vec<u8>,
    pub tls: Option<TlsDetails>,
}

/// HTTP headers structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeaders {
    pub headers: HashMap<String, String>,
}

/// TLS connection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsDetails {
    pub version: String,
    pub cipher: String,
}

/// Core attack request that can be executed by agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackRequest {
    pub id: Uuid,
    pub request_template: HttpRequestData,
    pub target_agents: Vec<String>,
    pub execution_config: ExecutionConfig,
    pub session_data: Option<Session>,
}

/// Configuration for attack execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub concurrent_requests_per_agent: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub distribution_strategy: DistributionStrategy,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            concurrent_requests_per_agent: 10,
            timeout_seconds: 30,
            retry_attempts: 3,
            distribution_strategy: DistributionStrategy::RoundRobin,
        }
    }
}

/// Strategy for distributing payloads across agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionStrategy {
    RoundRobin,
    Batch { batch_size: usize },
    LoadBalanced,
}

/// Result of an attack request execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    pub id: Uuid,
    pub request_id: Uuid,
    pub agent_id: String,
    pub request_data: HttpRequestData,
    pub response_data: Option<HttpResponseData>,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

/// Agent information for attack execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub hostname: String,
    pub status: AgentStatus,
    pub load: f64, // 0.0 to 1.0
    pub response_time_ms: Option<u64>,
}

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Online,
    Offline,
    Busy,
    Error,
}

/// Attack execution context
#[derive(Debug, Clone)]
pub struct AttackContext {
    pub attack_id: Uuid,
    pub module_type: ModuleType,
    pub priority: Priority,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Module types that can execute attacks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Repeater,
    Intruder,
}

/// Attack priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl HttpRequestData {
    /// Create a new HTTP request
    pub fn new(method: String, url: String) -> Self {
        Self {
            method,
            url,
            headers: None,
            body: Vec::new(),
            tls: None,
        }
    }
    
    /// Apply session data to the request
    pub fn apply_session(&mut self, session: &Session) {
        let session_headers = session.get_http_headers();
        
        // Initialize headers if not present
        if self.headers.is_none() {
            self.headers = Some(HttpHeaders {
                headers: HashMap::new(),
            });
        }
        
        // Apply session headers
        if let Some(ref mut headers) = self.headers {
            for (key, value) in session_headers {
                headers.headers.insert(key, value);
            }
        }
    }
    
    /// Set request body from string
    pub fn set_body_string(&mut self, body: String) {
        self.body = body.into_bytes();
    }
    
    /// Get request body as string
    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
    
    /// Add or update a header
    pub fn set_header(&mut self, key: String, value: String) {
        if self.headers.is_none() {
            self.headers = Some(HttpHeaders {
                headers: HashMap::new(),
            });
        }
        
        if let Some(ref mut headers) = self.headers {
            headers.headers.insert(key, value);
        }
    }
    
    /// Get a header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.as_ref()?.headers.get(key)
    }
}

impl HttpResponseData {
    /// Check if response indicates success (2xx status code)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    /// Get response body as string
    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
    
    /// Get response body length
    pub fn body_length(&self) -> usize {
        self.body.len()
    }
    
    /// Get a header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.as_ref()?.headers.get(key)
    }
}

impl AttackRequest {
    /// Create a new attack request
    pub fn new(
        request_template: HttpRequestData,
        target_agents: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_template,
            target_agents,
            execution_config: ExecutionConfig::default(),
            session_data: None,
        }
    }
    
    /// Set session data for authenticated requests
    pub fn with_session(mut self, session: Session) -> Self {
        self.session_data = Some(session);
        self
    }
    
    /// Set execution configuration
    pub fn with_config(mut self, config: ExecutionConfig) -> Self {
        self.execution_config = config;
        self
    }
}

impl AttackResult {
    /// Create a new attack result
    pub fn new(
        request_id: Uuid,
        agent_id: String,
        request_data: HttpRequestData,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            agent_id,
            request_data,
            response_data: None,
            executed_at: chrono::Utc::now(),
            duration_ms: None,
            error: None,
        }
    }
    
    /// Set successful response
    pub fn with_response(mut self, response: HttpResponseData, duration_ms: u64) -> Self {
        self.response_data = Some(response);
        self.duration_ms = Some(duration_ms);
        self
    }
    
    /// Set error result
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
    
    /// Check if the result indicates success
    pub fn is_success(&self) -> bool {
        self.error.is_none() && 
        self.response_data.as_ref().map_or(false, |r| r.is_success())
    }
}