//! Proxy Core Library
//!
//! This library provides core functionality for HTTP/HTTPS proxy operations,
//! including traffic interception, certificate management, and request/response handling.

pub mod admin;
pub mod ca;
pub mod certificates;
pub mod controller;
pub mod filter;
pub mod handlers;
/// Core proxy functionality modules
pub mod proxy;
pub mod system_metrics;

/// Configuration types and utilities
pub mod config;

/// Runtime traffic policy (dynamic rules)
pub mod policy;

/// Error types for proxy operations
pub mod error;

/// Memory management for response body capture
pub mod memory_manager;

/// Integration tests for memory management
#[cfg(test)]
pub mod memory_manager_integration_test;

pub use admin::Metrics;
pub use ca::CertificateAuthority;
pub use certificates::CertificateManager;
pub use config::{BodyCaptureConfig, ContentTypeFilterMode, ProxyConfig, ProxyStartupConfig};
pub use controller::InterceptController;
pub use error::{BodyCaptureError, ProxyError};
pub use filter::ScopeMatcher;
pub use handlers::LogHandler;
pub use memory_manager::{MemoryManager, MemoryStats};
pub use policy::{InterceptionRule, RuleAction, RuleCondition, ScopeConfig, TrafficPolicy};
/// Re-export commonly used types
pub use proxy::ProxyServer;
pub use system_metrics::{SystemMetricsCollector, SystemMetricsCollectorConfig};

/// Result type alias for proxy operations
pub type Result<T> = std::result::Result<T, ProxyError>;

pub mod pb {
    tonic::include_proto!("proxy");
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_library_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
