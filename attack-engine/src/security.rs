//! Security utilities for sensitive data masking and secure handling
//! 
//! This module provides comprehensive security features including:
//! - Sensitive data masking for logs and UI displays
//! - Secure handling of authentication data
//! - Data sanitization utilities
//! - Security policy enforcement

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use tracing::{warn, debug};
use crate::types::{HttpRequestData, HttpResponseData, HttpHeaders};
use proxy_common::Session;

/// Configuration for sensitive data masking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfig {
    /// Headers that should be masked (case-insensitive)
    pub sensitive_headers: HashSet<String>,
    
    /// Cookie names that should be masked (case-insensitive)
    pub sensitive_cookies: HashSet<String>,
    
    /// URL parameters that should be masked (case-insensitive)
    pub sensitive_url_params: HashSet<String>,
    
    /// Body field patterns that should be masked (regex patterns)
    pub sensitive_body_patterns: Vec<String>,
    
    /// Replacement string for masked values
    pub mask_replacement: String,
    
    /// Whether to mask entire values or show partial values
    pub partial_masking: bool,
    
    /// Number of characters to show when partial masking is enabled
    pub partial_show_chars: usize,
    
    /// Whether masking is enabled globally
    pub masking_enabled: bool,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        let mut sensitive_headers = HashSet::new();
        sensitive_headers.insert("authorization".to_string());
        sensitive_headers.insert("cookie".to_string());
        sensitive_headers.insert("set-cookie".to_string());
        sensitive_headers.insert("x-auth-token".to_string());
        sensitive_headers.insert("x-api-key".to_string());
        sensitive_headers.insert("x-csrf-token".to_string());
        sensitive_headers.insert("x-xsrf-token".to_string());
        sensitive_headers.insert("authentication".to_string());
        sensitive_headers.insert("proxy-authorization".to_string());
        sensitive_headers.insert("www-authenticate".to_string());
        
        let mut sensitive_cookies = HashSet::new();
        sensitive_cookies.insert("sessionid".to_string());
        sensitive_cookies.insert("session".to_string());
        sensitive_cookies.insert("auth".to_string());
        sensitive_cookies.insert("token".to_string());
        sensitive_cookies.insert("csrf".to_string());
        sensitive_cookies.insert("xsrf".to_string());
        sensitive_cookies.insert("jsessionid".to_string());
        sensitive_cookies.insert("phpsessid".to_string());
        sensitive_cookies.insert("asp.net_sessionid".to_string());
        
        let mut sensitive_url_params = HashSet::new();
        sensitive_url_params.insert("token".to_string());
        sensitive_url_params.insert("api_key".to_string());
        sensitive_url_params.insert("apikey".to_string());
        sensitive_url_params.insert("access_token".to_string());
        sensitive_url_params.insert("auth".to_string());
        sensitive_url_params.insert("password".to_string());
        sensitive_url_params.insert("secret".to_string());
        sensitive_url_params.insert("key".to_string());
        
        let sensitive_body_patterns = vec![
            r#""password"\s*:\s*"[^"]*""#.to_string(),
            r#""token"\s*:\s*"[^"]*""#.to_string(),
            r#""secret"\s*:\s*"[^"]*""#.to_string(),
            r#""api_key"\s*:\s*"[^"]*""#.to_string(),
            r#""apikey"\s*:\s*"[^"]*""#.to_string(),
            r#""auth"\s*:\s*"[^"]*""#.to_string(),
            r#""authorization"\s*:\s*"[^"]*""#.to_string(),
            r#"password=[\w\-\.%]+"#.to_string(),
            r#"token=[\w\-\.%]+"#.to_string(),
            r#"api_key=[\w\-\.%]+"#.to_string(),
        ];
        
        Self {
            sensitive_headers,
            sensitive_cookies,
            sensitive_url_params,
            sensitive_body_patterns,
            mask_replacement: "***MASKED***".to_string(),
            partial_masking: true,
            partial_show_chars: 4,
            masking_enabled: true,
        }
    }
}

/// Security manager for handling sensitive data masking and security policies
pub struct SecurityManager {
    config: MaskingConfig,
    body_patterns: Vec<Regex>,
}

impl SecurityManager {
    /// Create a new security manager with default configuration
    pub fn new() -> Self {
        let config = MaskingConfig::default();
        let body_patterns = Self::compile_body_patterns(&config.sensitive_body_patterns);
        
        Self {
            config,
            body_patterns,
        }
    }
    
    /// Create a new security manager with custom configuration
    pub fn with_config(config: MaskingConfig) -> Self {
        let body_patterns = Self::compile_body_patterns(&config.sensitive_body_patterns);
        
        Self {
            config,
            body_patterns,
        }
    }
    
    /// Update masking configuration
    pub fn update_config(&mut self, config: MaskingConfig) {
        self.body_patterns = Self::compile_body_patterns(&config.sensitive_body_patterns);
        self.config = config;
        debug!("🔒 Updated security masking configuration");
    }
    
    /// Get current masking configuration
    pub fn get_config(&self) -> &MaskingConfig {
        &self.config
    }
    
    /// Enable or disable masking globally
    pub fn set_masking_enabled(&mut self, enabled: bool) {
        self.config.masking_enabled = enabled;
        if enabled {
            debug!("🔒 Sensitive data masking enabled");
        } else {
            warn!("⚠️ Sensitive data masking disabled - sensitive data may be exposed!");
        }
    }
    
    /// Check if masking is enabled
    pub fn is_masking_enabled(&self) -> bool {
        self.config.masking_enabled
    }
    
    /// Mask sensitive data in HTTP request for logging/display
    pub fn mask_request(&self, request: &HttpRequestData) -> HttpRequestData {
        if !self.config.masking_enabled {
            return request.clone();
        }
        
        let mut masked_request = request.clone();
        
        // Mask headers
        if let Some(ref mut headers) = masked_request.headers {
            self.mask_headers(&mut headers.headers);
        }
        
        // Mask URL parameters
        masked_request.url = self.mask_url_parameters(&masked_request.url);
        
        // Mask request body
        if !masked_request.body.is_empty() {
            if let Ok(body_str) = String::from_utf8(masked_request.body.clone()) {
                let masked_body = self.mask_body_content(&body_str);
                masked_request.body = masked_body.into_bytes();
            }
        }
        
        masked_request
    }
    
    /// Mask sensitive data in HTTP response for logging/display
    pub fn mask_response(&self, response: &HttpResponseData) -> HttpResponseData {
        if !self.config.masking_enabled {
            return response.clone();
        }
        
        let mut masked_response = response.clone();
        
        // Mask headers (especially Set-Cookie)
        if let Some(ref mut headers) = masked_response.headers {
            self.mask_headers(&mut headers.headers);
        }
        
        // Mask response body (may contain sensitive data in JSON responses)
        if !masked_response.body.is_empty() {
            if let Ok(body_str) = String::from_utf8(masked_response.body.clone()) {
                let masked_body = self.mask_body_content(&body_str);
                masked_response.body = masked_body.into_bytes();
            }
        }
        
        masked_response
    }
    
    /// Mask sensitive data in session for logging/display
    pub fn mask_session(&self, session: &Session) -> Session {
        if !self.config.masking_enabled {
            return session.clone();
        }
        
        let mut masked_session = session.clone();
        
        // Mask session headers
        for (key, value) in masked_session.headers.iter_mut() {
            if self.is_sensitive_header(key) {
                *value = self.mask_value(value);
            }
        }
        
        // Mask session cookies
        for cookie in masked_session.cookies.iter_mut() {
            if self.is_sensitive_cookie(&cookie.name) {
                cookie.value = self.mask_value(&cookie.value);
            }
        }
        
        masked_session
    }
    
    /// Mask sensitive data in arbitrary text (for logs, error messages, etc.)
    pub fn mask_text(&self, text: &str) -> String {
        if !self.config.masking_enabled {
            return text.to_string();
        }
        
        let mut masked_text = text.to_string();
        
        // Apply body patterns to mask sensitive data in text
        for pattern in &self.body_patterns {
            masked_text = pattern.replace_all(&masked_text, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                // Try to preserve structure while masking the value
                if let Some(colon_pos) = full_match.find(':') {
                    let prefix = &full_match[..colon_pos + 1];
                    format!("{} \"{}\"", prefix, self.config.mask_replacement)
                } else if let Some(equals_pos) = full_match.find('=') {
                    let prefix = &full_match[..equals_pos + 1];
                    format!("{}{}", prefix, self.config.mask_replacement)
                } else {
                    self.config.mask_replacement.clone()
                }
            }).to_string();
        }
        
        masked_text
    }
    
    /// Create a secure log entry with masked sensitive data
    pub fn create_secure_log_entry(&self, message: &str, request: Option<&HttpRequestData>, response: Option<&HttpResponseData>) -> String {
        if !self.config.masking_enabled {
            return format!("{} [MASKING DISABLED]", message);
        }
        
        let mut log_parts = vec![message.to_string()];
        
        if let Some(req) = request {
            let masked_req = self.mask_request(req);
            log_parts.push(format!("Request: {} {}", masked_req.method, masked_req.url));
            
            if let Some(headers) = &masked_req.headers {
                for (key, value) in &headers.headers {
                    log_parts.push(format!("  {}: {}", key, value));
                }
            }
        }
        
        if let Some(resp) = response {
            let masked_resp = self.mask_response(resp);
            log_parts.push(format!("Response: {}", masked_resp.status_code));
            
            if let Some(headers) = &masked_resp.headers {
                for (key, value) in &headers.headers {
                    log_parts.push(format!("  {}: {}", key, value));
                }
            }
        }
        
        log_parts.join("\n")
    }
    
    /// Sanitize data for safe storage/transmission
    pub fn sanitize_for_storage(&self, data: &str) -> String {
        if !self.config.masking_enabled {
            return data.to_string();
        }
        
        // Remove potentially dangerous characters and mask sensitive patterns
        let sanitized = data
            .replace('\0', "") // Remove null bytes
            .replace('\r', "") // Remove carriage returns
            .chars()
            .filter(|c| c.is_ascii() || c.is_whitespace()) // Keep only ASCII and whitespace
            .collect::<String>();
        
        self.mask_text(&sanitized)
    }
    
    /// Validate that sensitive data is properly masked in output
    pub fn validate_masked_output(&self, output: &str) -> Result<(), SecurityViolation> {
        if !self.config.masking_enabled {
            return Ok(());
        }
        
        // Check for common sensitive patterns that should be masked
        let sensitive_patterns = vec![
            r"Bearer\s+[A-Za-z0-9\-\._~\+/]+=*",
            r"Basic\s+[A-Za-z0-9\+/]+=*",
            r"[Aa]pi[_-]?[Kk]ey\s*[:=]\s*['\x22]?[A-Za-z0-9\-\._~\+/]+=*['\x22]?",
            r"[Tt]oken\s*[:=]\s*['\x22]?[A-Za-z0-9\-\._~\+/]+=*['\x22]?",
            r"[Pp]assword\s*[:=]\s*['\x22]?[^\s'\x22]+['\x22]?",
            r"sessionid\s*=\s*[A-Za-z0-9\-\._~\+/]+=*",
        ];
        
        for pattern_str in sensitive_patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if pattern.is_match(output) {
                    return Err(SecurityViolation {
                        violation_type: ViolationType::UnmaskedSensitiveData,
                        description: format!("Potentially unmasked sensitive data detected: pattern '{}'", pattern_str),
                        severity: Severity::High,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a header name is considered sensitive
    fn is_sensitive_header(&self, header_name: &str) -> bool {
        self.config.sensitive_headers.contains(&header_name.to_lowercase())
    }
    
    /// Check if a cookie name is considered sensitive
    fn is_sensitive_cookie(&self, cookie_name: &str) -> bool {
        let cookie_lower = cookie_name.to_lowercase();
        self.config.sensitive_cookies.iter().any(|sensitive| {
            cookie_lower.contains(sensitive)
        })
    }
    
    /// Check if a URL parameter name is considered sensitive
    fn is_sensitive_url_param(&self, param_name: &str) -> bool {
        self.config.sensitive_url_params.contains(&param_name.to_lowercase())
    }
    
    /// Mask headers in place
    fn mask_headers(&self, headers: &mut HashMap<String, String>) {
        for (key, value) in headers.iter_mut() {
            if self.is_sensitive_header(key) {
                *value = self.mask_value(value);
            }
        }
    }
    
    /// Mask URL parameters
    fn mask_url_parameters(&self, url: &str) -> String {
        if let Ok(parsed_url) = url::Url::parse(url) {
            let mut new_url = parsed_url.clone();
            
            // Clear existing query parameters
            new_url.set_query(None);
            
            // Re-add parameters with masking
            if let Some(query) = parsed_url.query() {
                let mut masked_params = Vec::new();
                
                for param_pair in query.split('&') {
                    if let Some((key, value)) = param_pair.split_once('=') {
                        if self.is_sensitive_url_param(key) {
                            masked_params.push(format!("{}={}", key, self.config.mask_replacement));
                        } else {
                            masked_params.push(param_pair.to_string());
                        }
                    } else {
                        masked_params.push(param_pair.to_string());
                    }
                }
                
                if !masked_params.is_empty() {
                    new_url.set_query(Some(&masked_params.join("&")));
                }
            }
            
            new_url.to_string()
        } else {
            // If URL parsing fails, return original URL
            url.to_string()
        }
    }
    
    /// Mask body content using regex patterns
    fn mask_body_content(&self, body: &str) -> String {
        let mut masked_body = body.to_string();
        
        for pattern in &self.body_patterns {
            masked_body = pattern.replace_all(&masked_body, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                // Try to preserve JSON/form structure while masking the value
                if full_match.contains(':') {
                    // JSON-like pattern
                    if let Some(colon_pos) = full_match.find(':') {
                        let prefix = &full_match[..colon_pos + 1];
                        format!("{} \"{}\"", prefix, self.config.mask_replacement)
                    } else {
                        self.config.mask_replacement.clone()
                    }
                } else if full_match.contains('=') {
                    // Form parameter pattern
                    if let Some(equals_pos) = full_match.find('=') {
                        let prefix = &full_match[..equals_pos + 1];
                        format!("{}{}", prefix, self.config.mask_replacement)
                    } else {
                        self.config.mask_replacement.clone()
                    }
                } else {
                    self.config.mask_replacement.clone()
                }
            }).to_string();
        }
        
        masked_body
    }
    
    /// Mask a single value based on configuration
    fn mask_value(&self, value: &str) -> String {
        if value.is_empty() {
            return value.to_string();
        }
        
        if self.config.partial_masking && value.len() > self.config.partial_show_chars * 2 {
            let show_chars = self.config.partial_show_chars;
            let prefix = &value[..show_chars];
            let suffix = &value[value.len() - show_chars..];
            format!("{}***{}", prefix, suffix)
        } else {
            self.config.mask_replacement.clone()
        }
    }
    
    /// Compile regex patterns for body masking
    fn compile_body_patterns(patterns: &[String]) -> Vec<Regex> {
        patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        warn!("⚠️ Failed to compile regex pattern '{}': {}", pattern, e);
                        None
                    }
                }
            })
            .collect()
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Security violation detected during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: Severity,
}

/// Types of security violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    UnmaskedSensitiveData,
    InsecureDataTransmission,
    InvalidDataSanitization,
    ConfigurationViolation,
}

/// Severity levels for security violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Secure string wrapper that automatically masks its content in debug output
#[derive(Clone, Serialize, Deserialize)]
pub struct SecureString {
    value: String,
    masked: bool,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(value: String) -> Self {
        Self {
            value,
            masked: true,
        }
    }
    
    /// Create a secure string that won't be masked (use with caution)
    pub fn unmasked(value: String) -> Self {
        Self {
            value,
            masked: false,
        }
    }
    
    /// Get the actual value (use with caution)
    pub fn expose(&self) -> &str {
        &self.value
    }
    
    /// Get the length of the value
    pub fn len(&self) -> usize {
        self.value.len()
    }
    
    /// Check if the value is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
    
    /// Check if masking is enabled for this string
    pub fn is_masked(&self) -> bool {
        self.masked
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.masked {
            write!(f, "SecureString(***MASKED***)")
        } else {
            write!(f, "SecureString({})", self.value)
        }
    }
}

impl std::fmt::Display for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.masked {
            write!(f, "***MASKED***")
        } else {
            write!(f, "{}", self.value)
        }
    }
}

impl From<String> for SecureString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecureString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HttpHeaders;
    use std::collections::HashMap;

    fn create_test_request_with_sensitive_data() -> HttpRequestData {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string());
        headers.insert("Cookie".to_string(), "sessionid=abc123def456; csrf=xyz789".to_string());
        headers.insert("X-API-Key".to_string(), "sk-1234567890abcdef".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        HttpRequestData {
            method: "POST".to_string(),
            url: "https://api.example.com/login?token=secret123&debug=true".to_string(),
            headers: Some(HttpHeaders { headers }),
            body: r#"{"username":"user","password":"secret123","api_key":"sk-abcdef123456"}"#.as_bytes().to_vec(),
            tls: None,
        }
    }

    #[test]
    fn test_request_masking() {
        let security_manager = SecurityManager::new();
        let request = create_test_request_with_sensitive_data();
        
        let masked_request = security_manager.mask_request(&request);
        
        // Check that sensitive headers are masked
        let headers = masked_request.headers.unwrap();
        assert_eq!(headers.headers.get("Authorization").unwrap(), "***MASKED***");
        assert_eq!(headers.headers.get("Cookie").unwrap(), "***MASKED***");
        assert_eq!(headers.headers.get("X-API-Key").unwrap(), "***MASKED***");
        
        // Check that non-sensitive headers are preserved
        assert_eq!(headers.headers.get("Content-Type").unwrap(), "application/json");
        
        // Check that URL parameters are masked
        assert!(masked_request.url.contains("token=***MASKED***"));
        assert!(masked_request.url.contains("debug=true")); // Non-sensitive param preserved
        
        // Check that body is masked
        let body_str = String::from_utf8(masked_request.body).unwrap();
        assert!(body_str.contains("\"password\": \"***MASKED***\""));
        assert!(body_str.contains("\"api_key\": \"***MASKED***\""));
        assert!(body_str.contains("\"username\":\"user\"")); // Non-sensitive field preserved
    }

    #[test]
    fn test_partial_masking() {
        let mut config = MaskingConfig::default();
        config.partial_masking = true;
        config.partial_show_chars = 2;
        
        let security_manager = SecurityManager::with_config(config);
        
        let value = "very_long_secret_token_12345";
        let masked = security_manager.mask_value(value);
        
        assert!(masked.starts_with("ve"));
        assert!(masked.ends_with("45"));
        assert!(masked.contains("***"));
    }

    #[test]
    fn test_masking_disabled() {
        let mut config = MaskingConfig::default();
        config.masking_enabled = false;
        
        let security_manager = SecurityManager::with_config(config);
        let request = create_test_request_with_sensitive_data();
        
        let masked_request = security_manager.mask_request(&request);
        
        // When masking is disabled, request should be unchanged
        assert_eq!(request.url, masked_request.url);
        assert_eq!(request.body, masked_request.body);
    }

    #[test]
    fn test_security_violation_detection() {
        let security_manager = SecurityManager::new();
        
        // Test with unmasked sensitive data
        let unsafe_output = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = security_manager.validate_masked_output(unsafe_output);
        assert!(result.is_err());
        
        // Test with properly masked data
        let safe_output = "Authorization: ***MASKED***";
        let result = security_manager.validate_masked_output(safe_output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_string() {
        let secure = SecureString::new("secret_password".to_string());
        
        // Debug output should be masked
        let debug_output = format!("{:?}", secure);
        assert!(debug_output.contains("***MASKED***"));
        assert!(!debug_output.contains("secret_password"));
        
        // Display output should be masked
        let display_output = format!("{}", secure);
        assert_eq!(display_output, "***MASKED***");
        
        // Actual value should be accessible via expose()
        assert_eq!(secure.expose(), "secret_password");
    }

    #[test]
    fn test_text_masking() {
        let security_manager = SecurityManager::new();
        
        let text = r#"Login failed with token=abc123 and password="secret123""#;
        let masked = security_manager.mask_text(text);
        
        assert!(masked.contains("token=***MASKED***"));
        assert!(masked.contains("\"password\": \"***MASKED***\""));
        assert!(masked.contains("Login failed")); // Non-sensitive text preserved
    }

    #[test]
    fn test_data_sanitization() {
        let security_manager = SecurityManager::new();
        
        let unsafe_data = "password=secret123\0\r\napi_key=sk-123456";
        let sanitized = security_manager.sanitize_for_storage(unsafe_data);
        
        // Should remove null bytes and carriage returns
        assert!(!sanitized.contains('\0'));
        assert!(!sanitized.contains('\r'));
        
        // Should mask sensitive data
        assert!(sanitized.contains("password=***MASKED***"));
        assert!(sanitized.contains("api_key=***MASKED***"));
    }
}