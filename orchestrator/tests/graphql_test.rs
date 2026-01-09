use orchestrator::{LoggingConfig, Orchestrator, OrchestratorConfig};
use reqwest::Client;
use tokio::time::Duration;

#[tokio::test]
async fn test_graphql_endpoint() {
    // 1. Setup Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50055, // distinct port for test
        http_port: 50056,
        database_url: "sqlite::memory:".to_string(),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    };

    let orchestrator = Orchestrator::new(config).await.unwrap();

    // 2. Spawn Server
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });

    // Give it a moment to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 3. Query GraphQL
    let client = Client::new();
    let query = r#"{"query": "{ hello }"}"#;

    let res = client
        .post("http://127.0.0.1:50056/graphql")
        .header("Content-Type", "application/json")
        .body(query)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    println!("GraphQL Response: {}", body);
    assert!(body.contains("Hello from Proxxy!"));
}
