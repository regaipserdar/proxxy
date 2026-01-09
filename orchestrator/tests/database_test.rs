use orchestrator::Database;
use orchestrator::pb::{TrafficEvent, traffic_event, HttpRequestData};
use sqlx::Row;

#[tokio::test]
async fn test_database_persistence() {
    let db = Database::new("sqlite::memory:").await.expect("Failed to create DB");
    
    // Create Agent for FK constraint
    sqlx::query("INSERT INTO agents (id, name, hostname, version, status, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("agent-test")
        .bind("Test Agent")
        .bind("localhost")
        .bind("0.0.0")
        .bind("Online")
        .bind(0)
        .execute(db.pool())
        .await
        .expect("Failed to create test agent");
    
    let event = TrafficEvent {
        request_id: "req-123".to_string(),
        event: Some(traffic_event::Event::Request(HttpRequestData {
            method: "GET".to_string(),
            url: "http://example.com".to_string(),
            headers: None,
            body: vec![],
            tls: None,
        })),
    };

    db.save_request(&event, "agent-test").await.expect("Failed to save request");

    // Verify count
    let count: i64 = sqlx::query("SELECT count(*) FROM http_transactions")
        .fetch_one(db.pool())
        .await
        .expect("Failed to query")
        .get(0);
        
    assert_eq!(count, 1, "Should have 1 request saved");
}
