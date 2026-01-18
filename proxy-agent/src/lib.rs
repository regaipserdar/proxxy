//! Proxy Agent Binary
//!
//! Standalone executable that runs on servers to intercept network traffic.
//! Communicates with the orchestrator via gRPC and uses proxy-core for traffic handling.

use clap::Parser;
use proxy_core::{BodyCaptureConfig, CertificateAuthority, ProxyConfig, ProxyError, ProxyServer};
use std::path::PathBuf;
use tokio;
use uuid::Uuid;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Address to listen on for HTTP/HTTPS traffic
    #[arg(long, default_value = "127.0.0.1")]
    pub listen_addr: String,

    /// Port to listen on for HTTP/HTTPS traffic
    #[arg(long, default_value_t = 9095)]
    pub listen_port: u16,

    /// Port to expose the Admin API (health/metrics)
    #[arg(long, default_value_t = 9091)]
    pub admin_port: u16,

    /// URL of the Orchestrator gRPC service
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    pub orchestrator_url: String,

    /// Friendly name for this agent (e.g., "AWS-Worker-1")
    #[arg(long)]
    pub name: Option<String>,

    /// Path to the CA certificate (PEM)
    #[arg(long)]
    pub ca_cert: Option<PathBuf>,

    /// Path to the CA private key (PEM)
    #[arg(long)]
    pub ca_key: Option<PathBuf>,

    /// Path to configuration file for body capture settings (JSON format)
    #[arg(long)]
    pub body_capture_config: Option<PathBuf>,

    /// Enable response body capture (can be overridden by config file)
    #[arg(long)]
    pub enable_body_capture: Option<bool>,

    /// Maximum response body size to capture in bytes (can be overridden by config file)
    #[arg(long)]
    pub max_body_size: Option<usize>,

    /// Response timeout in seconds (can be overridden by config file)
    #[arg(long)]
    pub response_timeout: Option<u64>,

    /// Stream read timeout in seconds (can be overridden by config file)
    #[arg(long)]
    pub stream_timeout: Option<u64>,
}

pub mod client;
use client::OrchestratorClient;

#[cfg(test)]
mod config_test;

/// Load BodyCaptureConfig from multiple sources with precedence:
/// 1. CLI arguments (highest priority)
/// 2. Environment variables
/// 3. Configuration file
/// 4. Default values (lowest priority)
/// 
/// # Arguments
/// * `args` - Command line arguments
/// 
/// # Returns
/// * `Result<BodyCaptureConfig, Box<dyn std::error::Error>>` - Loaded and validated configuration
/// 
/// # Environment Variables
/// * `PROXXY_BODY_CAPTURE_ENABLED` - Enable/disable body capture (true/false)
/// * `PROXXY_MAX_BODY_SIZE` - Maximum body size in bytes
/// * `PROXXY_MEMORY_LIMIT` - Total memory limit for concurrent captures in bytes
/// * `PROXXY_MAX_CONCURRENT_CAPTURES` - Maximum number of concurrent captures
/// * `PROXXY_RESPONSE_TIMEOUT` - Response timeout in seconds
/// * `PROXXY_STREAM_TIMEOUT` - Stream read timeout in seconds
/// * `PROXXY_CONTENT_TYPE_FILTERS` - Comma-separated list of content-type filters
/// * `PROXXY_CONTENT_TYPE_MODE` - Filter mode: "capture_all", "whitelist", or "blacklist"
/// 
/// # Configuration File Format (JSON)
/// ```json
/// {
///   "enabled": true,
///   "max_body_size": 10485760,
///   "truncate_threshold": 1048576,
///   "memory_limit": 52428800,
///   "max_concurrent_captures": 10,
///   "content_type_filters": ["json", "xml", "html"],
///   "content_type_filter_mode": "Whitelist",
///   "response_timeout_secs": 30,
///   "stream_read_timeout_secs": 5
/// }
/// ```
/// 
/// # Requirements Addressed
/// * 7.1: Support enabling/disabling response body capture
/// * 7.2: Support configurable size limits for captured bodies
/// * 7.4: Support truncation thresholds for large responses
/// * 7.5: Support memory usage limits for concurrent captures
/// * 8.5: Validate timeout values are reasonable (not zero or negative)
fn load_body_capture_config(args: &Args) -> Result<BodyCaptureConfig, Box<dyn std::error::Error>> {
    // Start with default configuration
    let mut config = BodyCaptureConfig::default();
    
    // Load from config file if provided
    if let Some(config_path) = &args.body_capture_config {
        tracing::info!("Loading body capture configuration from file: {:?}", config_path);
        
        match std::fs::read_to_string(config_path) {
            Ok(config_content) => {
                match serde_json::from_str::<BodyCaptureConfig>(&config_content) {
                    Ok(file_config) => {
                        // Validate the loaded configuration
                        if let Err(e) = file_config.validate() {
                            return Err(format!("Invalid configuration in file: {}", e).into());
                        }
                        config = file_config;
                        tracing::info!("Successfully loaded configuration from file");
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse configuration file: {}", e).into());
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to read configuration file: {}", e).into());
            }
        }
    }
    
    // Override with environment variables if present
    if let Ok(enabled_str) = std::env::var("PROXXY_BODY_CAPTURE_ENABLED") {
        match enabled_str.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => config.enabled = true,
            "false" | "0" | "no" | "off" => config.enabled = false,
            _ => {
                return Err(format!("Invalid PROXXY_BODY_CAPTURE_ENABLED value: {}. Use true/false", enabled_str).into());
            }
        }
        tracing::info!("Body capture enabled from environment: {}", config.enabled);
    }
    
    if let Ok(max_size_str) = std::env::var("PROXXY_MAX_BODY_SIZE") {
        config.max_body_size = max_size_str.parse()
            .map_err(|e| format!("Invalid PROXXY_MAX_BODY_SIZE '{}': {}. Must be a positive integer (bytes)", max_size_str, e))?;
        tracing::info!("Max body size from environment: {} bytes", config.max_body_size);
    }
    
    if let Ok(memory_limit_str) = std::env::var("PROXXY_MEMORY_LIMIT") {
        config.memory_limit = memory_limit_str.parse()
            .map_err(|e| format!("Invalid PROXXY_MEMORY_LIMIT '{}': {}. Must be a positive integer (bytes)", memory_limit_str, e))?;
        tracing::info!("Memory limit from environment: {} bytes", config.memory_limit);
    }
    
    if let Ok(max_concurrent_str) = std::env::var("PROXXY_MAX_CONCURRENT_CAPTURES") {
        config.max_concurrent_captures = max_concurrent_str.parse()
            .map_err(|e| format!("Invalid PROXXY_MAX_CONCURRENT_CAPTURES '{}': {}. Must be a positive integer", max_concurrent_str, e))?;
        tracing::info!("Max concurrent captures from environment: {}", config.max_concurrent_captures);
    }
    
    if let Ok(response_timeout_str) = std::env::var("PROXXY_RESPONSE_TIMEOUT") {
        config.response_timeout_secs = response_timeout_str.parse()
            .map_err(|e| format!("Invalid PROXXY_RESPONSE_TIMEOUT '{}': {}. Must be a positive integer (seconds)", response_timeout_str, e))?;
        tracing::info!("Response timeout from environment: {} seconds", config.response_timeout_secs);
    }
    
    if let Ok(stream_timeout_str) = std::env::var("PROXXY_STREAM_TIMEOUT") {
        config.stream_read_timeout_secs = stream_timeout_str.parse()
            .map_err(|e| format!("Invalid PROXXY_STREAM_TIMEOUT '{}': {}. Must be a positive integer (seconds)", stream_timeout_str, e))?;
        tracing::info!("Stream timeout from environment: {} seconds", config.stream_read_timeout_secs);
    }
    
    if let Ok(filters_str) = std::env::var("PROXXY_CONTENT_TYPE_FILTERS") {
        config.content_type_filters = filters_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        tracing::info!("Content-type filters from environment: {:?}", config.content_type_filters);
    }
    
    if let Ok(mode_str) = std::env::var("PROXXY_CONTENT_TYPE_MODE") {
        use proxy_core::ContentTypeFilterMode;
        config.content_type_filter_mode = match mode_str.to_lowercase().as_str() {
            "capture_all" => ContentTypeFilterMode::CaptureAll,
            "whitelist" => ContentTypeFilterMode::Whitelist,
            "blacklist" => ContentTypeFilterMode::Blacklist,
            _ => {
                return Err(format!("Invalid PROXXY_CONTENT_TYPE_MODE: {}. Use capture_all, whitelist, or blacklist", mode_str).into());
            }
        };
        tracing::info!("Content-type filter mode from environment: {:?}", config.content_type_filter_mode);
    }
    
    // Override with CLI arguments if present (highest priority)
    if let Some(enabled) = args.enable_body_capture {
        config.enabled = enabled;
        tracing::info!("Body capture enabled from CLI: {}", config.enabled);
    }
    
    if let Some(max_size) = args.max_body_size {
        config.max_body_size = max_size;
        tracing::info!("Max body size from CLI: {} bytes", config.max_body_size);
    }
    
    if let Some(response_timeout) = args.response_timeout {
        config.response_timeout_secs = response_timeout;
        tracing::info!("Response timeout from CLI: {} seconds", config.response_timeout_secs);
    }
    
    if let Some(stream_timeout) = args.stream_timeout {
        config.stream_read_timeout_secs = stream_timeout;
        tracing::info!("Stream timeout from CLI: {} seconds", config.stream_read_timeout_secs);
    }
    
    // Validate the final configuration
    if let Err(e) = config.validate() {
        return Err(format!("Final configuration validation failed: {}", e).into());
    }
    
    tracing::info!("Final body capture configuration: enabled={}, max_body_size={}, memory_limit={}, max_concurrent={}, response_timeout={}s, stream_timeout={}s", 
        config.enabled, config.max_body_size, config.memory_limit, config.max_concurrent_captures, 
        config.response_timeout_secs, config.stream_read_timeout_secs);
    
    Ok(config)
}

pub async fn run_agent(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Logging should be initialized by the caller (main or test)

    tracing::info!("Starting Proxy Agent...");
    tracing::info!("  Listen: {}:{}", args.listen_addr, args.listen_port);
    tracing::info!("  Admin:  {}:{}", args.listen_addr, args.admin_port);
    tracing::info!("  Orch:   {}", args.orchestrator_url);

    // Load body capture configuration from multiple sources
    let body_capture_config = load_body_capture_config(&args)?;
    tracing::info!("Body capture configuration loaded successfully");

    // channel for traffic logs
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Generate ephemeral Agent ID
    let agent_id = Uuid::new_v4().to_string();
    tracing::info!("Agent ID: {}", agent_id);

    // Determine agent name (from CLI or auto-generate)
    let agent_name = args
        .name
        .unwrap_or_else(|| format!("Agent-{}", &agent_id[..8]));
    tracing::info!("Agent Name: {}", agent_name);

    // Start Orchestrator Client
    let client = OrchestratorClient::new(
        args.orchestrator_url.clone(),
        agent_id.clone(),
        agent_name.clone(),
    );

    // Initial Registration to fetch CA
    tracing::info!("Registering with Orchestrator to fetch CA...");
    let (ca_cert, ca_key) = loop {
        match client.register().await {
            Ok(creds) => break creds,
            Err(e) => {
                tracing::warn!("Failed to register/fetch CA: {}. Retrying in 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    };
    tracing::info!("Received CA credentials from Orchestrator");

    // Spawn client run loop for traffic streaming
    let client_for_run =
        OrchestratorClient::new(args.orchestrator_url.clone(), agent_id.clone(), agent_name.clone());

    tokio::spawn(async move {
        // client.run will re-register as part of its loop, which is fine (idempotent).
        client_for_run.run(rx).await;
    });

    // Load configuration
    let config = ProxyConfig {
        listen_address: args.listen_addr,
        listen_port: args.listen_port,
        admin_port: args.admin_port,
        orchestrator_endpoint: args.orchestrator_url,
        ..Default::default()
    };

    // Initialize CA from memory
    let ca = CertificateAuthority::from_pem(&ca_cert, &ca_key)
        .map_err(|e| ProxyError::Configuration(format!("Failed to init CA from network: {}", e)))?;

    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Create the proxy server with body capture configuration
    let proxy_server = ProxyServer::new(config, ca)
        .with_log_sender(tx)
        .with_body_capture_config(body_capture_config)
        .with_agent_info(agent_id, agent_name, env!("CARGO_PKG_VERSION").to_string(), hostname);

    tracing::info!("Starting proxy server...");

    // We return the server future so it can be awaited or raced
    proxy_server
        .run()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
