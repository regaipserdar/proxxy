use orchestrator::{OrchestratorConfig, Orchestrator, LoggingConfig};
use orchestrator::pb::proxy_service_client::ProxyServiceClient;
use orchestrator::pb::RegisterAgentRequest;
use tokio::time::Duration;

#[tokio::test]
async fn test_orchestrator_grpc_flow() {
    // 1. Start Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50051,
        http_port: 9091,
        database_url: "sqlite::memory:".to_string(),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig { level: "info".into() },
    };
    
    let orchestrator = Orchestrator::new(config.clone()).await.unwrap();
    
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });
    
    // Allow server to start
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 2. Connect Client
    let mut client = ProxyServiceClient::connect(format!("http://127.0.0.1:{}", config.grpc_port))
        .await
        .expect("Failed to connect to Orchestrator");
        
    // 3. Register Agent
    let req = tonic::Request::new(RegisterAgentRequest {
        agent_id: "test-agent-1".to_string(),
        hostname: "test-host".to_string(),
        version: "0.1.0".to_string(),
    });
    
    let resp = client.register_agent(req).await.expect("Registration failed");
    assert!(resp.into_inner().success, "Registration should return success");
}
