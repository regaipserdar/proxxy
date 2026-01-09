use proxy_core::{CertificateAuthority, ProxyConfig, ProxyServer};
use std::time::Duration;
use tempfile::tempdir;
use tokio::net::TcpStream;

#[tokio::test]
async fn test_proxy_server_startup() {
    let dir = tempdir().unwrap();
    let ca = CertificateAuthority::new(dir.path()).unwrap();

    // Use a high port to avoid conflicts
    let port = 19090;

    let config = ProxyConfig {
        listen_address: "127.0.0.1".to_string(),
        listen_port: port,
        admin_port: 19091,
        ..Default::default()
    };

    let server = ProxyServer::new(config.clone(), ca);

    // Spawn server in background
    let _handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Proxy server failed: {}", e);
        }
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Try to connect to the proxy port
    let addr = format!("127.0.0.1:{}", port);
    match TcpStream::connect(&addr).await {
        Ok(_) => println!("Successfully connected to proxy at {}", addr),
        Err(e) => panic!("Failed to connect to proxy at {}: {}", addr, e),
    }

    // Verify Admin API (Health)
    let admin_port = config.admin_port;
    let health_url = format!("http://127.0.0.1:{}/health", admin_port);
    let resp = reqwest::get(&health_url).await.unwrap();
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("ok"));

    // Verify Admin API (Metrics)
    let metrics_url = format!("http://127.0.0.1:{}/metrics", admin_port);
    let resp = reqwest::get(&metrics_url).await.unwrap();
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("total_requests"));
}
