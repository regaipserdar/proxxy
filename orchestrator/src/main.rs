use clap::Parser;
use orchestrator::{Orchestrator, OrchestratorConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Proxxy Orchestrator - Central management server for distributed MITM proxy
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// gRPC server port for agent connections
    #[arg(long, default_value_t = 50051)]
    grpc_port: u16,

    /// HTTP API port for REST/GraphQL endpoints
    #[arg(long, default_value_t = 9090)]
    http_port: u16,

    /// Database connection URL
    #[arg(long, default_value = "sqlite:./proxxy.db")]
    database_url: String,

    /// Health check interval in seconds
    #[arg(long, default_value_t = 30)]
    health_check_interval: u64,

    /// Agent timeout in seconds
    #[arg(long, default_value_t = 300)]
    agent_timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging with proper configuration
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orchestrator=info,proxy_core=info,flow_engine=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create configuration
    let config = OrchestratorConfig {
        grpc_port: args.grpc_port,
        http_port: args.http_port,
        database_url: args.database_url.clone(),
        health_check_interval: args.health_check_interval,
        agent_timeout: args.agent_timeout,
        logging: orchestrator::LoggingConfig::default(),
    };

    // Create and start orchestrator
    let orchestrator = Orchestrator::new(config).await?;

    println!("🚀 Orchestrator starting...");
    println!(
        "📡 gRPC server will be available at: http://127.0.0.1:{}",
        args.grpc_port
    );
    println!(
        "🌐 HTTP API will be available at: http://127.0.0.1:{}",
        args.http_port
    );
    println!(
        "📊 GraphiQL Playground: http://127.0.0.1:{}/graphql",
        args.http_port
    );
    println!("💾 Database: {}", args.database_url);
    println!();
    println!("💡 Tip: Use --help to see all available options");
    println!();

    orchestrator.start().await?;

    Ok(())
}
