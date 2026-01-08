//! Proxy Core Library
//! 
//! This library provides core functionality for HTTP/HTTPS proxy operations,
//! including traffic interception, certificate management, and request/response handling.

/// Core proxy functionality modules
pub mod proxy;
pub mod certificates;
pub mod handlers;
pub mod ca;
pub mod admin;
pub mod filter;
pub mod controller;
pub mod system_metrics;

/// Configuration types and utilities
pub mod config;

/// Runtime traffic policy (dynamic rules)
pub mod policy;

/// Error types for proxy operations
pub mod error;

/// Re-export commonly used types
pub use proxy::ProxyServer;
pub use certificates::CertificateManager;
pub use handlers::LogHandler;
pub use ca::CertificateAuthority;
pub use admin::Metrics;
pub use filter::ScopeMatcher;
pub use controller::InterceptController;
pub use config::{ProxyConfig, ProxyStartupConfig};
pub use policy::{TrafficPolicy, ScopeConfig, InterceptionRule, RuleAction, RuleCondition};
pub use system_metrics::{SystemMetricsCollector, SystemMetricsCollectorConfig};
pub use error::ProxyError;

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