//! Proxy Agent Binary
//! 
//! Standalone executable that runs on servers to intercept network traffic.
//! Communicates with the orchestrator via gRPC and uses proxy-core for traffic handling.

use proxy_core::{ProxyServer, ProxyConfig, ProxyError, CertificateAuthority};
use std::path::PathBuf;
use clap::Parser;
use tokio;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to listen on for HTTP/HTTPS traffic
    #[arg(long, default_value = "127.0.0.1")]
    listen_addr: String,

    /// Port to listen on for HTTP/HTTPS traffic
    #[arg(long, default_value_t = 9095)]
    listen_port: u16,

    /// Port to expose the Admin API (health/metrics)
    #[arg(long, default_value_t = 9091)]
    admin_port: u16,

    /// URL of the Orchestrator gRPC service
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    orchestrator_url: String,

    /// Friendly name for this agent (e.g., "AWS-Worker-1")
    #[arg(long)]
    name: Option<String>,

    /// Path to the CA certificate (PEM)
    #[arg(long)]
    ca_cert: Option<PathBuf>,

    /// Path to the CA private key (PEM)
    #[arg(long)]
    ca_key: Option<PathBuf>,
}

mod client;
use client::OrchestratorClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();

    tracing::info!("Starting Proxy Agent...");
    tracing::info!("  Listen: {}:{}", args.listen_addr, args.listen_port);
    tracing::info!("  Admin:  {}:{}", args.listen_addr, args.admin_port);
    tracing::info!("  Orch:   {}", args.orchestrator_url);

    // channel for traffic logs
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Generate ephemeral Agent ID
    let agent_id = Uuid::new_v4().to_string();
    tracing::info!("Agent ID: {}", agent_id);

    // Determine agent name (from CLI or auto-generate)
    let agent_name = args.name.unwrap_or_else(|| {
        format!("Agent-{}", &agent_id[..8])
    });
    tracing::info!("Agent Name: {}", agent_name);

    // Start Orchestrator Client
    let client = OrchestratorClient::new(
        args.orchestrator_url.clone(), 
        agent_id.clone(),
        agent_name.clone()
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
    let client_for_run = OrchestratorClient::new(
        args.orchestrator_url.clone(), 
        agent_id.clone(),
        agent_name
    );

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

    // Create the proxy server
    let proxy_server = ProxyServer::new(config, ca)
        .with_log_sender(tx);
    
    tracing::info!("Starting proxy server...");
    
    tokio::select! {
        result = proxy_server.run() => {
            if let Err(e) = result {
                tracing::error!("Proxy server failed: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }
        }
        _ = tokio::signal::ctrl_c() => {
             tracing::info!("Shutdown signal received, stopping proxy server...");
        }
    }
    
    tracing::info!("Proxy Agent stopped successfully");
    Ok(())
}