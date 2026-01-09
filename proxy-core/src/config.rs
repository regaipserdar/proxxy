//! Configuration types and utilities

use serde::{Deserialize, Serialize};

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
