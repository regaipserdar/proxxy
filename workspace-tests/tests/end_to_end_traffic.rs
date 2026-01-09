use orchestrator::{LoggingConfig, Orchestrator, OrchestratorConfig};
use proxy_agent::{run_agent, Args as AgentArgs};
use sqlx::Row;
use std::time::Duration;
use tokio::net::TcpListener;

// Helper to find a free port
async fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn test_end_to_end_traffic_flow() {
    // 1. Setup Environment
    let _ = tracing_subscriber::fmt::try_init();

    // Create temp DB
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_proxxy.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    // Get free ports
    let orch_http_port = get_free_port().await;
    let orch_grpc_port = get_free_port().await;
    let agent_port = get_free_port().await;
    let agent_admin_port = get_free_port().await;

    // 2. Start Orchestrator
    let orch_config = OrchestratorConfig {
        grpc_port: orch_grpc_port,
        http_port: orch_http_port,
        database_url: db_url.clone(), // This will accept file path for sqlite
        health_check_interval: 10,
        agent_timeout: 30,
        logging: LoggingConfig {
            level: "info".into(),
        },
    };

    let orchestrator = Orchestrator::new(orch_config)
        .await
        .expect("Failed to create orchestrator");

    // Spawn Orchestrator
    tokio::spawn(async move {
        if let Err(e) = orchestrator.start().await {
            tracing::error!("Orchestrator failed: {}", e);
        }
    });

    // Wait for Orchestrator to be ready (naive wait)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 3. Start Proxy Agent
    let agent_args = AgentArgs {
        listen_addr: "127.0.0.1".to_string(),
        listen_port: agent_port,
        admin_port: agent_admin_port,
        orchestrator_url: format!("http://127.0.0.1:{}", orch_grpc_port),
        name: Some("e2e-test-agent".to_string()),
        ca_cert: None,
        ca_key: None,
    };

    // Spawn Agent
    tokio::spawn(async move {
        if let Err(e) = run_agent(agent_args).await {
            tracing::error!("Agent failed: {}", e);
        }
    });

    // Wait for Agent to register and start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 4. Send Request through Proxy
    // We'll target http://example.com as a simple connectivity check.
    // Ideally we'd spawn a local echo server, but example.com is stable enough for basic verification
    // if internet is available. If not, this might be flaky.
    // Let's use a local server instead to be safe.

    let target_port = get_free_port().await;
    let target_addr = format!("127.0.0.1:{}", target_port);

    // Spawn local target server (simple echo)
    tokio::spawn(async move {
        let listener = TcpListener::bind(target_addr).await.unwrap();
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0; 1024];
                let _n = socket.read(&mut buf).await.unwrap();
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello World!";
                socket.write_all(response.as_bytes()).await.unwrap();
            });
        }
    });

    let proxy_url = format!("http://127.0.0.1:{}", agent_port);
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).unwrap())
        .danger_accept_invalid_certs(true) // For our generated CA
        .build()
        .unwrap();

    let target_url = format!("http://127.0.0.1:{}", target_port);

    tracing::info!("Sending request to {} via {}", target_url, proxy_url);

    // Retry logic in case of startup delay
    let mut success = false;
    for _ in 0..5 {
        match client.get(&target_url).send().await {
            Ok(resp) => {
                assert_eq!(resp.status(), 200);
                let text = resp.text().await.unwrap();
                assert_eq!(text, "Hello World!");
                success = true;
                break;
            }
            Err(e) => {
                tracing::warn!("Request failed, retrying: {}", e);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    assert!(success, "Failed to send request through proxy");

    // 5. Verify Database
    tracing::info!("Verifying database records...");

    // Give some time for async logging to DB
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Connect to the same DB file
    // Note: sqlite::memory: would not work here unless shared, but we used a file path.
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to test DB");

    let row = sqlx::query("SELECT count(*) FROM http_transactions")
        .fetch_one(&pool)
        .await
        .expect("Failed to query DB");

    let count: i64 = row.get(0);
    tracing::info!("Found {} transactions in DB", count);

    assert!(count > 0, "Database should contain at least 1 transaction");

    // Verify details
    let row = sqlx::query(
        "SELECT req_url, res_status FROM http_transactions ORDER BY req_timestamp DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch transaction details");

    let url: String = row.get("req_url");
    // Check if URL contains our target (might have trailing slash)
    assert!(url.contains(&target_url));
}
