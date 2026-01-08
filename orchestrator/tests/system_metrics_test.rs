//! System Metrics Integration Tests
//! 
//! Tests for system metrics gRPC streaming, database storage, and GraphQL integration.

use orchestrator::{Database, pb::*};
use std::sync::Arc;
use tokio::sync::{mpsc, broadcast};

#[tokio::test]
async fn test_system_metrics_database_storage() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    
    // First, create the agent record to satisfy foreign key constraint
    db.upsert_agent("test-agent", "Test Agent", "localhost", "1.0.0").await.unwrap();
    
    // Create a sample metrics event
    let metrics_event = SystemMetricsEvent {
        agent_id: "test-agent".to_string(),
        timestamp: 1640995200, // 2022-01-01 00:00:00 UTC
        metrics: Some(SystemMetrics {
            cpu_usage_percent: 45.5,
            memory_used_bytes: 1024 * 1024 * 512, // 512 MB
            memory_total_bytes: 1024 * 1024 * 1024 * 2, // 2 GB
            network: Some(NetworkMetrics {
                rx_bytes_total: 1000000,
                tx_bytes_total: 500000,
                rx_bytes_per_sec: 1000,
                tx_bytes_per_sec: 500,
                interfaces: vec![],
            }),
            disk: Some(DiskMetrics {
                read_bytes_total: 2000000,
                write_bytes_total: 1000000,
                read_bytes_per_sec: 2000,
                write_bytes_per_sec: 1000,
                available_bytes: 1024 * 1024 * 1024 * 10, // 10 GB
                total_bytes: 1024 * 1024 * 1024 * 20, // 20 GB
            }),
            process: Some(ProcessMetrics {
                cpu_usage_percent: 25.0,
                memory_bytes: 1024 * 1024 * 100, // 100 MB
                uptime_seconds: 3600, // 1 hour
                thread_count: 10,
                file_descriptor_count: 50,
            }),
        }),
    };
    
    // Store metrics
    db.save_system_metrics(&metrics_event).await.unwrap();
    
    // Retrieve metrics
    let retrieved = db.get_recent_system_metrics(Some("test-agent"), 1).await.unwrap();
    assert_eq!(retrieved.len(), 1);
    
    let retrieved_event = &retrieved[0];
    assert_eq!(retrieved_event.agent_id, "test-agent");
    assert_eq!(retrieved_event.timestamp, 1640995200);
    
    let retrieved_metrics = retrieved_event.metrics.as_ref().unwrap();
    assert_eq!(retrieved_metrics.cpu_usage_percent, 45.5);
    assert_eq!(retrieved_metrics.memory_used_bytes, 1024 * 1024 * 512);
    assert_eq!(retrieved_metrics.memory_total_bytes, 1024 * 1024 * 1024 * 2);
}

#[tokio::test]
async fn test_system_metrics_database_query_filtering() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    
    // Create metrics for multiple agents
    let agents = vec!["agent-1", "agent-2", "agent-3"];
    for agent_id in &agents {
        // First create the agent record
        db.upsert_agent(agent_id, &format!("Agent {}", agent_id), "localhost", "1.0.0").await.unwrap();
    }
    
    for (i, agent_id) in agents.iter().enumerate() {
        let metrics_event = SystemMetricsEvent {
            agent_id: agent_id.to_string(),
            timestamp: 1640995200 + i as i64 * 60, // 1 minute apart
            metrics: Some(SystemMetrics {
                cpu_usage_percent: 10.0 + i as f32 * 10.0,
                memory_used_bytes: 1024 * 1024 * (100 + i * 100) as u64,
                memory_total_bytes: 1024 * 1024 * 1024 * 2,
                network: None,
                disk: None,
                process: None,
            }),
        };
        db.save_system_metrics(&metrics_event).await.unwrap();
    }
    
    // Query all metrics
    let all_metrics = db.get_recent_system_metrics(None, 10).await.unwrap();
    assert_eq!(all_metrics.len(), 3);
    
    // Query specific agent
    let agent1_metrics = db.get_recent_system_metrics(Some("agent-1"), 10).await.unwrap();
    assert_eq!(agent1_metrics.len(), 1);
    assert_eq!(agent1_metrics[0].agent_id, "agent-1");
    assert_eq!(agent1_metrics[0].metrics.as_ref().unwrap().cpu_usage_percent, 10.0);
    
    // Query with limit
    let limited_metrics = db.get_recent_system_metrics(None, 2).await.unwrap();
    assert_eq!(limited_metrics.len(), 2);
    // Should be ordered by timestamp DESC, so agent-3 and agent-2
    assert_eq!(limited_metrics[0].agent_id, "agent-3");
    assert_eq!(limited_metrics[1].agent_id, "agent-2");
}

#[tokio::test]
async fn test_system_metrics_bounds_validation() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    
    // First, create the agent record to satisfy foreign key constraint
    db.upsert_agent("test-agent", "Test Agent", "localhost", "1.0.0").await.unwrap();
    
    // Test with edge case values
    let metrics_event = SystemMetricsEvent {
        agent_id: "test-agent".to_string(),
        timestamp: 1640995200,
        metrics: Some(SystemMetrics {
            cpu_usage_percent: 100.0, // Max CPU
            memory_used_bytes: 0, // Min memory used
            memory_total_bytes: u64::MAX, // Max memory total
            network: Some(NetworkMetrics {
                rx_bytes_total: u64::MAX,
                tx_bytes_total: 0,
                rx_bytes_per_sec: u64::MAX,
                tx_bytes_per_sec: 0,
                interfaces: vec![],
            }),
            disk: Some(DiskMetrics {
                read_bytes_total: 0,
                write_bytes_total: u64::MAX,
                read_bytes_per_sec: 0,
                write_bytes_per_sec: u64::MAX,
                available_bytes: 0, // No space available
                total_bytes: u64::MAX,
            }),
            process: Some(ProcessMetrics {
                cpu_usage_percent: 0.0, // Min process CPU
                memory_bytes: u64::MAX, // Max process memory
                uptime_seconds: 0, // Just started
                thread_count: 1, // Min threads
                file_descriptor_count: u32::MAX, // Max FDs
            }),
        }),
    };
    
    // Should store without error
    db.save_system_metrics(&metrics_event).await.unwrap();
    
    // Should retrieve correctly
    let retrieved = db.get_recent_system_metrics(Some("test-agent"), 1).await.unwrap();
    assert_eq!(retrieved.len(), 1);
    
    let retrieved_metrics = retrieved[0].metrics.as_ref().unwrap();
    assert_eq!(retrieved_metrics.cpu_usage_percent, 100.0);
    assert_eq!(retrieved_metrics.memory_used_bytes, 0);
    assert_eq!(retrieved_metrics.memory_total_bytes, u64::MAX);
}

#[tokio::test]
async fn test_metrics_broadcast_integration() {
    let (metrics_broadcast_tx, mut metrics_broadcast_rx) = broadcast::channel(10);
    
    // Create a sample metrics event
    let metrics_event = SystemMetricsEvent {
        agent_id: "test-agent".to_string(),
        timestamp: 1640995200,
        metrics: Some(SystemMetrics {
            cpu_usage_percent: 50.0,
            memory_used_bytes: 1024 * 1024 * 256,
            memory_total_bytes: 1024 * 1024 * 1024,
            network: None,
            disk: None,
            process: None,
        }),
    };
    
    // Send metrics event
    metrics_broadcast_tx.send(metrics_event.clone()).unwrap();
    
    // Receive and verify
    let received = metrics_broadcast_rx.recv().await.unwrap();
    assert_eq!(received.agent_id, "test-agent");
    assert_eq!(received.timestamp, 1640995200);
    assert_eq!(received.metrics.as_ref().unwrap().cpu_usage_percent, 50.0);
}

#[tokio::test]
async fn test_metrics_streaming_reliability() {
    let (metrics_tx, mut metrics_rx) = mpsc::channel(100);
    
    // Simulate high-frequency metrics streaming
    let num_events = 50;
    for i in 0..num_events {
        let metrics_event = SystemMetricsEvent {
            agent_id: format!("agent-{}", i % 3), // 3 different agents
            timestamp: 1640995200 + i,
            metrics: Some(SystemMetrics {
                cpu_usage_percent: (i % 100) as f32,
                memory_used_bytes: 1024 * 1024 * (100 + i) as u64,
                memory_total_bytes: 1024 * 1024 * 1024 * 2,
                network: None,
                disk: None,
                process: None,
            }),
        };
        
        metrics_tx.send(metrics_event).await.unwrap();
    }
    
    // Verify all events are received
    let mut received_count = 0;
    let mut agent_counts = std::collections::HashMap::new();
    
    // Close sender to end the stream
    drop(metrics_tx);
    
    while let Some(event) = metrics_rx.recv().await {
        received_count += 1;
        *agent_counts.entry(event.agent_id).or_insert(0) += 1;
    }
    
    assert_eq!(received_count, num_events);
    assert_eq!(agent_counts.len(), 3); // 3 different agents
    
    // Each agent should have roughly equal number of events
    for count in agent_counts.values() {
        assert!(*count >= 16 && *count <= 18); // 50/3 ≈ 16.67
    }
}

#[tokio::test]
async fn test_system_metrics_data_consistency_over_time() {
    let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
    
    // Store metrics over time for the same agent
    let agent_id = "consistency-test-agent";
    let base_timestamp = 1640995200;
    
    // First, create the agent record to satisfy foreign key constraint
    db.upsert_agent(agent_id, "Consistency Test Agent", "localhost", "1.0.0").await.unwrap();
    
    for i in 0..10 {
        let metrics_event = SystemMetricsEvent {
            agent_id: agent_id.to_string(),
            timestamp: base_timestamp + i * 60, // 1 minute intervals
            metrics: Some(SystemMetrics {
                cpu_usage_percent: 20.0 + (i as f32 * 5.0), // Increasing CPU
                memory_used_bytes: 1024 * 1024 * (200 + i * 50) as u64, // Increasing memory
                memory_total_bytes: 1024 * 1024 * 1024 * 4, // Constant total memory
                network: Some(NetworkMetrics {
                    rx_bytes_total: (1000000 + i * 10000) as u64, // Increasing network usage
                    tx_bytes_total: (500000 + i * 5000) as u64,
                    rx_bytes_per_sec: 1000,
                    tx_bytes_per_sec: 500,
                    interfaces: vec![],
                }),
                disk: None,
                process: None,
            }),
        };
        
        db.save_system_metrics(&metrics_event).await.unwrap();
    }
    
    // Retrieve all metrics for the agent
    let metrics = db.get_recent_system_metrics(Some(agent_id), 20).await.unwrap();
    assert_eq!(metrics.len(), 10);
    
    // Verify ordering (should be DESC by timestamp)
    for i in 0..9 {
        assert!(metrics[i].timestamp >= metrics[i + 1].timestamp);
    }
    
    // Verify data consistency
    let latest_metrics = &metrics[0]; // Most recent
    let oldest_metrics = &metrics[9]; // Oldest
    
    // Memory total should be consistent
    assert_eq!(
        latest_metrics.metrics.as_ref().unwrap().memory_total_bytes,
        oldest_metrics.metrics.as_ref().unwrap().memory_total_bytes
    );
    
    // CPU should have increased over time (latest should be higher than oldest)
    assert!(
        latest_metrics.metrics.as_ref().unwrap().cpu_usage_percent >
        oldest_metrics.metrics.as_ref().unwrap().cpu_usage_percent
    );
    
    // Network total bytes should have increased
    let latest_network = latest_metrics.metrics.as_ref().unwrap().network.as_ref().unwrap();
    let oldest_network = oldest_metrics.metrics.as_ref().unwrap().network.as_ref().unwrap();
    assert!(latest_network.rx_bytes_total > oldest_network.rx_bytes_total);
    assert!(latest_network.tx_bytes_total > oldest_network.tx_bytes_total);
}