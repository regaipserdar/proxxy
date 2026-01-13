//! Configuration types and utilities

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::error::BodyCaptureError;

/// Static Proxy Startup Configuration
/// These settings are set at startup and do not change during runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStartupConfig {
    /// Address to listen on
    pub listen_address: String,
    /// Port to listen on
    pub listen_port: u16,
    /// Orchestrator endpoint for communication
    pub orchestrator_endpoint: String,
    /// Admin API port
    pub admin_port: u16,
    /// Certificate configuration
    pub certificate_config: CertificateConfig,
}

impl Default for ProxyStartupConfig {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1".to_string(),
            listen_port: 8080,
            orchestrator_endpoint: "http://127.0.0.1:9090".to_string(),
            admin_port: 9091,
            certificate_config: CertificateConfig::default(),
        }
    }
}

/// Certificate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    /// Path to store generated certificates
    pub cert_store_path: String,
    /// Certificate validity duration in days
    pub validity_days: u32,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            cert_store_path: "./certs".to_string(),
            validity_days: 365,
        }
    }
}

// Legacy ProxyConfig type alias for backward compatibility
pub type ProxyConfig = ProxyStartupConfig;

/// Content-type filtering mode for selective body capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentTypeFilterMode {
    /// Capture all content types (no filtering)
    CaptureAll,
    /// Only capture content types that match the filters (whitelist)
    Whitelist,
    /// Capture all content types except those that match the filters (blacklist)
    Blacklist,
}

impl Default for ContentTypeFilterMode {
    fn default() -> Self {
        ContentTypeFilterMode::CaptureAll
    }
}

/// Configuration for HTTP response body capture functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCaptureConfig {
    /// Whether body capture is enabled
    pub enabled: bool,
    /// Maximum size of a single response body to capture (in bytes)
    pub max_body_size: usize,
    /// Size threshold for truncation warnings (in bytes)
    pub truncate_threshold: usize,
    /// Total memory limit for concurrent body captures (in bytes)
    pub memory_limit: usize,
    /// Maximum number of concurrent body captures allowed
    pub max_concurrent_captures: usize,
    /// Content-type filters for selective capture (empty = capture all)
    pub content_type_filters: Vec<String>,
    /// Content-type filtering mode (whitelist, blacklist, or capture all)
    pub content_type_filter_mode: ContentTypeFilterMode,
    /// Overall timeout for reading a complete response (in seconds)
    pub response_timeout_secs: u64,
    /// Timeout for reading individual chunks from stream (in seconds)
    pub stream_read_timeout_secs: u64,
}

impl Default for BodyCaptureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_body_size: 10 * 1024 * 1024,        // 10MB
            truncate_threshold: 1024 * 1024,         // 1MB
            memory_limit: 100 * 1024 * 1024,         // 100MB total (allows 10 concurrent 10MB captures)
            max_concurrent_captures: 10,             // 10 concurrent captures
            content_type_filters: vec![],
            content_type_filter_mode: ContentTypeFilterMode::default(),
            response_timeout_secs: 30,               // 30 seconds total response timeout
            stream_read_timeout_secs: 5,             // 5 seconds per chunk read timeout
        }
    }
}

impl BodyCaptureConfig {
    /// Create a new BodyCaptureConfig with validation
    pub fn new(
        enabled: bool,
        max_body_size: usize,
        truncate_threshold: usize,
        memory_limit: usize,
        max_concurrent_captures: usize,
        content_type_filters: Vec<String>,
        content_type_filter_mode: ContentTypeFilterMode,
        response_timeout_secs: u64,
        stream_read_timeout_secs: u64,
    ) -> Result<Self, BodyCaptureError> {
        let config = Self {
            enabled,
            max_body_size,
            truncate_threshold,
            memory_limit,
            max_concurrent_captures,
            content_type_filters,
            content_type_filter_mode,
            response_timeout_secs,
            stream_read_timeout_secs,
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration settings
    pub fn validate(&self) -> Result<(), BodyCaptureError> {
        // Validate timeout values are positive and reasonable
        if self.response_timeout_secs == 0 {
            return Err(BodyCaptureError::ConfigurationError(
                "Response timeout must be greater than 0 seconds. Recommended range: 5-300 seconds".to_string()
            ));
        }
        
        if self.stream_read_timeout_secs == 0 {
            return Err(BodyCaptureError::ConfigurationError(
                "Stream read timeout must be greater than 0 seconds. Recommended range: 1-60 seconds".to_string()
            ));
        }
        
        // Validate timeout values are not excessively large
        if self.response_timeout_secs > 3600 {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Response timeout of {} seconds is too large. Maximum recommended: 3600 seconds (1 hour)", self.response_timeout_secs)
            ));
        }
        
        if self.stream_read_timeout_secs > 300 {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Stream read timeout of {} seconds is too large. Maximum recommended: 300 seconds (5 minutes)", self.stream_read_timeout_secs)
            ));
        }
        
        // Validate size limits are reasonable
        if self.max_body_size == 0 {
            return Err(BodyCaptureError::ConfigurationError(
                "Max body size must be greater than 0 bytes. Recommended range: 1KB - 100MB".to_string()
            ));
        }
        
        // Check for excessively large body size (> 1GB)
        if self.max_body_size > 1024 * 1024 * 1024 {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Max body size of {} bytes ({:.1} GB) is too large. Maximum recommended: 1GB", 
                    self.max_body_size, self.max_body_size as f64 / (1024.0 * 1024.0 * 1024.0))
            ));
        }
        
        if self.memory_limit == 0 {
            return Err(BodyCaptureError::ConfigurationError(
                "Memory limit must be greater than 0 bytes. Recommended range: 10MB - 1GB".to_string()
            ));
        }
        
        // Check for excessively large memory limit (> 10GB)
        if self.memory_limit > 10 * 1024 * 1024 * 1024 {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Memory limit of {} bytes ({:.1} GB) is too large. Maximum recommended: 10GB", 
                    self.memory_limit, self.memory_limit as f64 / (1024.0 * 1024.0 * 1024.0))
            ));
        }
        
        if self.max_concurrent_captures == 0 {
            return Err(BodyCaptureError::ConfigurationError(
                "Max concurrent captures must be greater than 0. Recommended range: 1-100".to_string()
            ));
        }
        
        // Check for excessively large concurrent captures
        if self.max_concurrent_captures > 1000 {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Max concurrent captures of {} is too large. Maximum recommended: 1000", self.max_concurrent_captures)
            ));
        }
        
        if self.truncate_threshold > self.max_body_size {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Truncate threshold ({} bytes) cannot be larger than max body size ({} bytes)", 
                    self.truncate_threshold, self.max_body_size)
            ));
        }
        
        // Validate memory limit is reasonable compared to max body size
        if self.memory_limit < self.max_body_size {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Memory limit ({} bytes) should be at least as large as max body size ({} bytes). Consider increasing memory limit or decreasing max body size", 
                    self.memory_limit, self.max_body_size)
            ));
        }
        
        // Validate timeout relationships
        if self.stream_read_timeout_secs > self.response_timeout_secs {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Stream read timeout ({} seconds) cannot be larger than response timeout ({} seconds)", 
                    self.stream_read_timeout_secs, self.response_timeout_secs)
            ));
        }
        
        // Validate memory efficiency - warn if memory limit allows too few concurrent captures
        let min_memory_per_capture = self.max_body_size;
        let theoretical_max_captures = self.memory_limit / min_memory_per_capture;
        if theoretical_max_captures < self.max_concurrent_captures {
            return Err(BodyCaptureError::ConfigurationError(
                format!("Memory configuration is inefficient. With memory limit of {} bytes and max body size of {} bytes, only {} concurrent captures are possible, but {} are configured. Consider increasing memory limit or decreasing max body size or max concurrent captures", 
                    self.memory_limit, self.max_body_size, theoretical_max_captures, self.max_concurrent_captures)
            ));
        }
        
        Ok(())
    }
    
    /// Get response timeout as Duration
    pub fn response_timeout(&self) -> Duration {
        Duration::from_secs(self.response_timeout_secs)
    }
    
    /// Get stream read timeout as Duration
    pub fn stream_read_timeout(&self) -> Duration {
        Duration::from_secs(self.stream_read_timeout_secs)
    }
    
    /// Check if a content type should be captured based on filters
    /// 
    /// This method implements content-type based filtering logic that supports:
    /// - CaptureAll: Capture all content types regardless of filters
    /// - Whitelist: Only capture content types that match the filters
    /// - Blacklist: Capture all content types except those that match the filters
    /// 
    /// Content-type matching is case-insensitive and uses substring matching.
    /// For example, a filter "json" will match "application/json", "text/json", etc.
    /// 
    /// # Arguments
    /// * `content_type` - The content-type header value to check
    /// 
    /// # Returns
    /// * `true` if the content type should be captured
    /// * `false` if the content type should be skipped
    /// 
    /// # Requirements Addressed
    /// * 7.3: Support content-type filtering for selective capture
    pub fn should_capture_content_type(&self, content_type: &str) -> bool {
        match self.content_type_filter_mode {
            ContentTypeFilterMode::CaptureAll => {
                // Always capture regardless of filters
                true
            }
            ContentTypeFilterMode::Whitelist => {
                if self.content_type_filters.is_empty() {
                    // No filters in whitelist mode means capture nothing
                    false
                } else {
                    // Check if content type matches any filter (whitelist)
                    self.content_type_filters.iter().any(|filter| {
                        self.content_type_matches(content_type, filter)
                    })
                }
            }
            ContentTypeFilterMode::Blacklist => {
                if self.content_type_filters.is_empty() {
                    // No filters in blacklist mode means capture all
                    true
                } else {
                    // Check if content type does NOT match any filter (blacklist)
                    !self.content_type_filters.iter().any(|filter| {
                        self.content_type_matches(content_type, filter)
                    })
                }
            }
        }
    }
    
    /// Helper method to check if a content type matches a filter pattern
    /// 
    /// Performs case-insensitive substring matching. This allows flexible
    /// filtering where a filter like "json" matches "application/json",
    /// "text/json", etc.
    /// 
    /// # Arguments
    /// * `content_type` - The content-type header value
    /// * `filter` - The filter pattern to match against
    /// 
    /// # Returns
    /// * `true` if the content type matches the filter
    /// * `false` otherwise
    fn content_type_matches(&self, content_type: &str, filter: &str) -> bool {
        // Parse content-type to extract the main type (before semicolon)
        let main_content_type = content_type
            .split(';')
            .next()
            .unwrap_or(content_type)
            .trim();
        
        // Perform case-insensitive substring matching
        main_content_type.to_lowercase().contains(&filter.to_lowercase())
    }
    
    /// Create a configuration with whitelist filtering
    /// 
    /// Only content types matching the provided filters will be captured.
    /// 
    /// # Arguments
    /// * `filters` - List of content-type patterns to whitelist
    /// 
    /// # Returns
    /// * `BodyCaptureConfig` with whitelist filtering enabled
    pub fn with_whitelist_filters(mut self, filters: Vec<String>) -> Self {
        self.content_type_filters = filters;
        self.content_type_filter_mode = ContentTypeFilterMode::Whitelist;
        self
    }
    
    /// Create a configuration with blacklist filtering
    /// 
    /// All content types except those matching the provided filters will be captured.
    /// 
    /// # Arguments
    /// * `filters` - List of content-type patterns to blacklist
    /// 
    /// # Returns
    /// * `BodyCaptureConfig` with blacklist filtering enabled
    pub fn with_blacklist_filters(mut self, filters: Vec<String>) -> Self {
        self.content_type_filters = filters;
        self.content_type_filter_mode = ContentTypeFilterMode::Blacklist;
        self
    }
    
    /// Disable content-type filtering (capture all content types)
    /// 
    /// # Returns
    /// * `BodyCaptureConfig` with no content-type filtering
    pub fn with_no_content_type_filtering(mut self) -> Self {
        self.content_type_filters.clear();
        self.content_type_filter_mode = ContentTypeFilterMode::CaptureAll;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_filtering_capture_all() {
        let config = BodyCaptureConfig::default();
        
        // CaptureAll mode should capture everything
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("text/html"));
        assert!(config.should_capture_content_type("image/png"));
        assert!(config.should_capture_content_type("application/octet-stream"));
    }

    #[test]
    fn test_content_type_filtering_whitelist_empty() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec![]);
        
        // Empty whitelist should capture nothing
        assert!(!config.should_capture_content_type("application/json"));
        assert!(!config.should_capture_content_type("text/html"));
        assert!(!config.should_capture_content_type("image/png"));
    }

    #[test]
    fn test_content_type_filtering_whitelist_with_filters() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["json".to_string(), "html".to_string()]);
        
        // Should capture matching types
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("text/json"));
        assert!(config.should_capture_content_type("text/html"));
        assert!(config.should_capture_content_type("application/html"));
        
        // Should not capture non-matching types
        assert!(!config.should_capture_content_type("image/png"));
        assert!(!config.should_capture_content_type("application/octet-stream"));
        assert!(!config.should_capture_content_type("text/plain"));
    }

    #[test]
    fn test_content_type_filtering_blacklist_empty() {
        let config = BodyCaptureConfig::default()
            .with_blacklist_filters(vec![]);
        
        // Empty blacklist should capture everything
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("text/html"));
        assert!(config.should_capture_content_type("image/png"));
        assert!(config.should_capture_content_type("application/octet-stream"));
    }

    #[test]
    fn test_content_type_filtering_blacklist_with_filters() {
        let config = BodyCaptureConfig::default()
            .with_blacklist_filters(vec!["image".to_string(), "video".to_string()]);
        
        // Should capture non-matching types
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("text/html"));
        assert!(config.should_capture_content_type("text/plain"));
        
        // Should not capture matching types
        assert!(!config.should_capture_content_type("image/png"));
        assert!(!config.should_capture_content_type("image/jpeg"));
        assert!(!config.should_capture_content_type("video/mp4"));
        assert!(!config.should_capture_content_type("video/avi"));
    }

    #[test]
    fn test_content_type_case_insensitive_matching() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["JSON".to_string()]);
        
        // Case insensitive matching
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("APPLICATION/JSON"));
        assert!(config.should_capture_content_type("text/JSON"));
        assert!(config.should_capture_content_type("Text/Json"));
    }

    #[test]
    fn test_content_type_with_parameters() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["json".to_string()]);
        
        // Should match content types with parameters
        assert!(config.should_capture_content_type("application/json; charset=utf-8"));
        assert!(config.should_capture_content_type("application/json;charset=utf-8"));
        assert!(config.should_capture_content_type("text/json; boundary=something"));
        
        // Should not match non-json types even with parameters
        assert!(!config.should_capture_content_type("text/html; charset=utf-8"));
    }

    #[test]
    fn test_content_type_substring_matching() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["xml".to_string()]);
        
        // Should match various XML content types
        assert!(config.should_capture_content_type("application/xml"));
        assert!(config.should_capture_content_type("text/xml"));
        assert!(config.should_capture_content_type("application/soap+xml"));
        assert!(config.should_capture_content_type("application/rss+xml"));
        
        // Should not match non-XML types
        assert!(!config.should_capture_content_type("application/json"));
        assert!(!config.should_capture_content_type("text/html"));
    }

    #[test]
    fn test_configuration_validation() {
        // Valid configuration should pass
        let valid_config = BodyCaptureConfig::new(
            true,
            1024 * 1024,  // 1MB
            512 * 1024,   // 512KB
            10 * 1024 * 1024,  // 10MB
            5,
            vec!["json".to_string()],
            ContentTypeFilterMode::Whitelist,
            30,
            5,
        );
        assert!(valid_config.is_ok());

        // Invalid timeout should fail
        let invalid_timeout = BodyCaptureConfig::new(
            true,
            1024 * 1024,
            512 * 1024,
            10 * 1024 * 1024,
            5,
            vec![],
            ContentTypeFilterMode::CaptureAll,
            0,  // Invalid timeout
            5,
        );
        assert!(invalid_timeout.is_err());
    }

    #[test]
    fn test_enhanced_validation_timeout_bounds() {
        // Test excessively large response timeout
        let config = BodyCaptureConfig {
            response_timeout_secs: 7200, // 2 hours - too large
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));

        // Test excessively large stream timeout
        let config = BodyCaptureConfig {
            stream_read_timeout_secs: 600, // 10 minutes - too large
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_enhanced_validation_size_bounds() {
        // Test excessively large body size (> 1GB)
        let config = BodyCaptureConfig {
            max_body_size: 2 * 1024 * 1024 * 1024, // 2GB - too large
            memory_limit: 3 * 1024 * 1024 * 1024,  // 3GB to satisfy memory >= body size
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));

        // Test excessively large memory limit (> 10GB)
        let config = BodyCaptureConfig {
            memory_limit: 15 * 1024 * 1024 * 1024, // 15GB - too large
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));

        // Test excessively large concurrent captures
        let config = BodyCaptureConfig {
            max_concurrent_captures: 2000, // Too many
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_enhanced_validation_memory_efficiency() {
        // Test inefficient memory configuration
        let config = BodyCaptureConfig {
            max_body_size: 10 * 1024 * 1024,  // 10MB
            memory_limit: 15 * 1024 * 1024,   // 15MB - only allows 1 concurrent capture
            max_concurrent_captures: 5,       // But we want 5 - inefficient
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Memory configuration is inefficient"));
    }

    #[test]
    fn test_enhanced_validation_helpful_error_messages() {
        // Test that error messages include helpful context
        let config = BodyCaptureConfig {
            response_timeout_secs: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Recommended range"));

        // Test size error includes actual values
        let config = BodyCaptureConfig {
            truncate_threshold: 20 * 1024 * 1024, // 20MB
            max_body_size: 10 * 1024 * 1024,      // 10MB - smaller than threshold
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("20971520")); // Should include actual byte values
        assert!(error_msg.contains("10485760"));
    }

    #[test]
    fn test_builder_methods() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["json".to_string(), "xml".to_string()]);
        
        assert_eq!(config.content_type_filter_mode, ContentTypeFilterMode::Whitelist);
        assert_eq!(config.content_type_filters, vec!["json", "xml"]);

        let config = config.with_blacklist_filters(vec!["image".to_string()]);
        assert_eq!(config.content_type_filter_mode, ContentTypeFilterMode::Blacklist);
        assert_eq!(config.content_type_filters, vec!["image"]);

        let config = config.with_no_content_type_filtering();
        assert_eq!(config.content_type_filter_mode, ContentTypeFilterMode::CaptureAll);
        assert!(config.content_type_filters.is_empty());
    }

    #[test]
    fn test_content_type_filtering_edge_cases() {
        // Test empty content type
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["json".to_string()]);
        
        assert!(!config.should_capture_content_type(""));
        
        // Test malformed content type
        assert!(!config.should_capture_content_type("not-a-valid-content-type"));
        
        // Test content type with multiple parameters
        assert!(config.should_capture_content_type("application/json; charset=utf-8; boundary=something"));
        
        // Test content type with spaces
        assert!(config.should_capture_content_type(" application/json "));
    }

    #[test]
    fn test_multiple_filter_matching() {
        let config = BodyCaptureConfig::default()
            .with_whitelist_filters(vec!["json".to_string(), "xml".to_string(), "text".to_string()]);
        
        // Should match any of the filters
        assert!(config.should_capture_content_type("application/json"));
        assert!(config.should_capture_content_type("text/xml"));
        assert!(config.should_capture_content_type("text/plain"));
        assert!(config.should_capture_content_type("application/soap+xml"));
        
        // Should not match if none of the filters match
        assert!(!config.should_capture_content_type("image/png"));
        assert!(!config.should_capture_content_type("video/mp4"));
    }
}
