//! Common Session Interface for Proxxy Modules
//! 
//! This module defines the unified session interface used across all Proxxy modules:
//! - LSR (Login Sequence Recorder) - Produces sessions
//! - Repeater/Intruder - Consumes sessions for attacks
//! - Nuclei Scanner - Consumes sessions for authenticated scans

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unified session data structure used across all Proxxy modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: Uuid,
    
    /// Human-readable session name
    pub name: String,
    
    /// All HTTP headers including Authorization, X-CSRF-Token, etc.
    /// This includes both authentication headers and session-related headers
    pub headers: HashMap<String, String>,
    
    /// Session cookies extracted from browser
    pub cookies: Vec<Cookie>,
    
    /// When this session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// When this session expires (if known)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Reference to the LSR profile that created this session
    pub profile_id: Option<Uuid>,
    
    /// Session validation status
    pub status: SessionStatus,
    
    /// Additional metadata for debugging and tracking
    pub metadata: SessionMetadata,
}

/// Individual cookie data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<SameSite>,
}

/// Cookie SameSite attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// Session validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is valid and ready to use
    Active,
    /// Session has expired
    Expired,
    /// Session validation failed
    Invalid,
    /// Session is being validated
    Validating,
}

/// Additional session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Agent ID that recorded this session
    pub agent_id: Option<String>,
    
    /// URL where session was validated
    pub validation_url: Option<String>,
    
    /// Success indicators used for validation
    pub success_indicators: Vec<String>,
    
    /// Last validation timestamp
    pub last_validated: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Number of times this session has been used
    pub usage_count: u64,
}

impl Session {
    /// Create a new session
    pub fn new(name: String, profile_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            headers: HashMap::new(),
            cookies: Vec::new(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            profile_id,
            status: SessionStatus::Validating,
            metadata: SessionMetadata {
                agent_id: None,
                validation_url: None,
                success_indicators: Vec::new(),
                last_validated: None,
                usage_count: 0,
            },
        }
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        match &self.expires_at {
            Some(expires) => chrono::Utc::now() > *expires,
            None => false,
        }
    }
    
    /// Get all headers as HTTP header format for requests
    pub fn get_http_headers(&self) -> HashMap<String, String> {
        let mut headers = self.headers.clone();
        
        // Add cookies as Cookie header if not already present
        if !self.cookies.is_empty() && !headers.contains_key("Cookie") {
            let cookie_header = self.cookies
                .iter()
                .map(|c| format!("{}={}", c.name, c.value))
                .collect::<Vec<_>>()
                .join("; ");
            headers.insert("Cookie".to_string(), cookie_header);
        }
        
        headers
    }
    
    /// Increment usage counter
    pub fn increment_usage(&mut self) {
        self.metadata.usage_count += 1;
    }
    
    /// Mark session as validated
    pub fn mark_validated(&mut self, validation_url: String) {
        self.status = SessionStatus::Active;
        self.metadata.validation_url = Some(validation_url);
        self.metadata.last_validated = Some(chrono::Utc::now());
    }
    
    /// Mark session as expired
    pub fn mark_expired(&mut self) {
        self.status = SessionStatus::Expired;
    }
}

/// Session events for event-driven architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// Session was created
    Created { session_id: Uuid },
    
    /// Session was validated successfully
    Validated { session_id: Uuid, validation_url: String },
    
    /// Session validation failed
    ValidationFailed { session_id: Uuid, error: String },
    
    /// Session expired
    Expired { session_id: Uuid },
    
    /// Session was used in a request
    Used { session_id: Uuid, target_url: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("Test Session".to_string(), None);
        assert_eq!(session.name, "Test Session");
        assert!(matches!(session.status, SessionStatus::Validating));
        assert_eq!(session.metadata.usage_count, 0);
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new("Test Session".to_string(), None);
        session.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(session.is_expired());
    }

    #[test]
    fn test_http_headers_with_cookies() {
        let mut session = Session::new("Test Session".to_string(), None);
        session.headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        session.cookies.push(Cookie {
            name: "sessionid".to_string(),
            value: "abc123".to_string(),
            domain: None,
            path: None,
            expires: None,
            http_only: true,
            secure: false,
            same_site: None,
        });
        
        let headers = session.get_http_headers();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(headers.get("Cookie"), Some(&"sessionid=abc123".to_string()));
    }
}