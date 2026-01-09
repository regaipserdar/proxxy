//! Proxy Agent Binary Entry Point

use clap::Parser;
use proxy_agent::{run_agent, Args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    tokio::select! {
        result = run_agent(args) => {
            if let Err(e) = result {
                tracing::error!("Proxy server failed: {}", e);
                return Err(e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
             tracing::info!("Shutdown signal received, stopping proxy server...");
        }
    }

    Ok(())
}
