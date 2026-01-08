//! Repeater Engine Tests
//! 
//! Tests for request replay functionality including database retrieval,
//! agent command sending, and end-to-end replay flow.

use orchestrator::{Database, AgentRegistry};
use std::sync::Arc;

#[tokio::test]
async fn test_get_request_by_id() {
    // Create in-memory database
    let db = Database::new("sqlite::memory:").await.unwrap();
    
    // Register agent first (foreign key requirement)
    db.upsert_agent("test-agent-1", "Test Agent", "localhost", "0.1.0").await.unwrap();
    
    // Insert a test request
    use orchestrator::pb::{TrafficEvent, traffic_event, HttpRequestData, HttpHeaders};
    
    let mut headers_map = std::collections::HashMap::new();
    headers_map.insert("Content-Type".to_string(), "application/json".to_string());
    
    let event = TrafficEvent {
        request_id: "test-req-123".to_string(),
        event: Some(traffic_event::Event::Request(HttpRequestData {
            method: "POST".to_string(),
            url: "https://example.com/api/test".to_string(),
            headers: Some(HttpHeaders { headers: headers_map }),
            body: b"{\"test\":\"data\"}".to_vec(),
            tls: None,
        })),
    };
    
    db.save_request(&event, "test-agent-1").await.unwrap();
    
    // Retrieve the request
    let retrieved = db.get_request_by_id("test-req-123").await.unwrap();
    assert!(retrieved.is_some());
    
    let req_data = retrieved.unwrap();
    assert_eq!(req_data.method, "POST");
    assert_eq!(req_data.url, "https://example.com/api/test");
    assert_eq!(req_data.body, b"{\"test\":\"data\"}");
}

#[tokio::test]
async fn test_get_agent_id_for_request() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    
    // Register agent first
    db.upsert_agent("agent-abc-123", "Test Agent", "localhost", "0.1.0").await.unwrap();
    
    use orchestrator::pb::{TrafficEvent, traffic_event, HttpRequestData};
    
    let event = TrafficEvent {
        request_id: "test-req-456".to_string(),
        event: Some(traffic_event::Event::Request(HttpRequestData {
            method: "GET".to_string(),
            url: "https://example.com/test".to_string(),
            headers: None,
            body: vec![],
            tls: None,
        })),
    };
    
    db.save_request(&event, "agent-abc-123").await.unwrap();
    
    let agent_id = db.get_agent_id_for_request("test-req-456").await.unwrap();
    assert_eq!(agent_id, Some("agent-abc-123".to_string()));
}

#[tokio::test]
async fn test_replay_request_not_found() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    
    let result = db.get_request_by_id("non-existent-request").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_agent_registry_command_channel() {
    use tokio::sync::mpsc;
    use tonic::Status;
    use orchestrator::pb::InterceptCommand;
    
    let registry = AgentRegistry::new();
    let (tx, mut rx) = mpsc::channel::<Result<InterceptCommand, Status>>(10);
    
    registry.register_agent(
        "test-agent".to_string(),
        "Test Agent".to_string(),
        "localhost".to_string(),
        tx,
    );
    
    // Get the command channel
    let agent_tx = registry.get_agent_tx("test-agent");
    assert!(agent_tx.is_some());
    
    // Send a test command
    let cmd_tx = agent_tx.unwrap();
    use orchestrator::pb::{intercept_command, ExecuteRequest, HttpRequestData};
    
    let test_cmd = InterceptCommand {
        command: Some(intercept_command::Command::Execute(ExecuteRequest {
            request_id: "replay-test".to_string(),
            request: Some(HttpRequestData {
                method: "GET".to_string(),
                url: "http://test.com".to_string(),
                headers: None,
                body: vec![],
                tls: None,
            }),
        })),
    };
    
    cmd_tx.send(Ok(test_cmd)).await.unwrap();
    
    // Verify command was received
    let received = rx.recv().await.unwrap();
    assert!(received.is_ok());
    
    if let Some(intercept_command::Command::Execute(exec)) = received.unwrap().command {
        assert_eq!(exec.request_id, "replay-test");
    } else {
        panic!("Expected Execute command");
    }
}

#[tokio::test]
async fn test_replay_flow_integration() {
    // This test verifies the complete replay flow:
    // 1. Request is saved to database
    // 2. Request is retrieved by ID
    // 3. Agent ID is found
    // 4. Command channel is available
    
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    let registry = Arc::new(AgentRegistry::new());
    
    use tokio::sync::mpsc;
    use tonic::Status;
    use orchestrator::pb::{TrafficEvent, traffic_event, HttpRequestData, InterceptCommand};
    
    // Setup: Register agent in database first
    db.upsert_agent("agent-replay-test", "Replay Test Agent", "localhost", "0.1.0").await.unwrap();
    
    // Setup: Save a request
    let event = TrafficEvent {
        request_id: "replay-flow-test".to_string(),
        event: Some(traffic_event::Event::Request(HttpRequestData {
            method: "POST".to_string(),
            url: "https://api.example.com/data".to_string(),
            headers: None,
            body: b"test payload".to_vec(),
            tls: None,
        })),
    };
    
    db.save_request(&event, "agent-replay-test").await.unwrap();
    
    // Setup: Register agent
    let (tx, mut rx) = mpsc::channel::<Result<InterceptCommand, Status>>(10);
    registry.register_agent(
        "agent-replay-test".to_string(),
        "Replay Test Agent".to_string(),
        "localhost".to_string(),
        tx,
    );
    
    // Simulate replay mutation logic
    let request_data = db.get_request_by_id("replay-flow-test").await.unwrap().unwrap();
    let agent_id = db.get_agent_id_for_request("replay-flow-test").await.unwrap().unwrap();
    let agent_tx = registry.get_agent_tx(&agent_id).unwrap();
    
    // Send execute command
    use orchestrator::pb::{intercept_command, ExecuteRequest};
    let execute_cmd = InterceptCommand {
        command: Some(intercept_command::Command::Execute(ExecuteRequest {
            request_id: "replay-flow-test-replay".to_string(),
            request: Some(request_data),
        })),
    };
    
    agent_tx.send(Ok(execute_cmd)).await.unwrap();
    
    // Verify command was received
    let received = rx.recv().await.unwrap().unwrap();
    if let Some(intercept_command::Command::Execute(exec)) = received.command {
        assert_eq!(exec.request_id, "replay-flow-test-replay");
        assert_eq!(exec.request.unwrap().url, "https://api.example.com/data");
    } else {
        panic!("Expected Execute command");
    }
}
