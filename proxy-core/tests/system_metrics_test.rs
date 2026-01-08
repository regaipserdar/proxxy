//! System Metrics Collection Tests
//! 
//! Tests for system metrics collection functionality including accuracy bounds checking,
//! gRPC streaming integration, and metrics data consistency.

use proxy_core::{SystemMetricsCollector, SystemMetricsCollectorConfig};
use tokio::sync::mpsc;
use std::time::Duration;

#[tokio::test]
async fn test_system_metrics_collector_creation() {
    let collector = SystemMetricsCollector::new("test-agent".to_string());
    assert_eq!(collector.agent_id, "test-agent");
    assert_eq!(collector.config.collection_interval_seconds, 5);
}

#[tokio::test]
async fn test_collect_metrics() {
    let mut collector = SystemMetricsCollector::new("test-agent".to_string());
    
    let metrics_event = collector.collect_metrics().await.unwrap();
    
    assert_eq!(metrics_event.agent_id, "test-agent");
    assert!(metrics_event.timestamp > 0);
    assert!(metrics_event.metrics.is_some());
    
    let metrics = metrics_event.metrics.unwrap();
    assert!(metrics.cpu_usage_percent >= 0.0);
    assert!(metrics.memory_used_bytes > 0);
    assert!(metrics.memory_total_bytes > 0);
}

#[tokio::test]
async fn test_config_update() {
    let mut collector = SystemMetricsCollector::new("test-agent".to_string());
    
    let new_config = SystemMetricsCollectorConfig {
        collection_interval_seconds: 10,
        include_network_details: false,
        ..Default::default()
    };
    
    collector.update_config(new_config.clone());
    
    assert_eq!(collector.config.collection_interval_seconds, 10);
    assert!(!collector.config.include_network_details);
}

#[test]
fn test_metrics_bounds_checking() {
    let collector = SystemMetricsCollector::new("test-agent".to_string());
    
    // Test that CPU usage is within bounds (0-100%)
    let cpu_usage = collector.system.global_cpu_info().cpu_usage();
    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0, "CPU usage should be between 0-100%");
    
    // Test that memory values are non-negative
    let memory_used = collector.system.used_memory();
    let memory_total = collector.system.total_memory();
    assert!(memory_used <= memory_total, "Used memory should not exceed total memory");
    assert!(memory_total > 0, "Total memory should be positive");
}

#[tokio::test]
async fn test_metrics_streaming_integration() {
    let mut collector = SystemMetricsCollector::with_config(
        "test-agent".to_string(),
        SystemMetricsCollectorConfig {
            collection_interval_seconds: 1, // Fast for testing
            ..Default::default()
        }
    );
    
    let (metrics_tx, mut metrics_rx) = mpsc::channel(10);
    let (command_tx, command_rx) = mpsc::channel(10);
    
    // Start streaming in background
    let streaming_handle = tokio::spawn(async move {
        collector.start_streaming(metrics_tx, command_rx).await
    });
    
    // Wait for at least one metrics event
    let timeout = tokio::time::timeout(Duration::from_secs(3), metrics_rx.recv()).await;
    assert!(timeout.is_ok(), "Should receive metrics within timeout");
    
    let metrics_event = timeout.unwrap().unwrap();
    assert_eq!(metrics_event.agent_id, "test-agent");
    assert!(metrics_event.metrics.is_some());
    
    // Stop streaming
    drop(command_tx);
    streaming_handle.abort();
}

#[tokio::test]
async fn test_metrics_data_consistency() {
    let mut collector = SystemMetricsCollector::new("test-agent".to_string());
    
    // Collect metrics twice
    let metrics1 = collector.collect_metrics().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let metrics2 = collector.collect_metrics().await.unwrap();
    
    // Verify consistency
    assert_eq!(metrics1.agent_id, metrics2.agent_id);
    assert!(metrics2.timestamp >= metrics1.timestamp);
    
    // Both should have metrics
    assert!(metrics1.metrics.is_some());
    assert!(metrics2.metrics.is_some());
    
    let m1 = metrics1.metrics.unwrap();
    let m2 = metrics2.metrics.unwrap();
    
    // Memory total should be consistent
    assert_eq!(m1.memory_total_bytes, m2.memory_total_bytes);
    
    // CPU and memory usage should be reasonable
    assert!(m1.cpu_usage_percent >= 0.0 && m1.cpu_usage_percent <= 100.0);
    assert!(m2.cpu_usage_percent >= 0.0 && m2.cpu_usage_percent <= 100.0);
    assert!(m1.memory_used_bytes <= m1.memory_total_bytes);
    assert!(m2.memory_used_bytes <= m2.memory_total_bytes);
}

#[tokio::test]
async fn test_network_metrics_collection() {
    let mut collector = SystemMetricsCollector::with_config(
        "test-agent".to_string(),
        SystemMetricsCollectorConfig {
            include_network_details: true,
            ..Default::default()
        }
    );
    
    let metrics_event = collector.collect_metrics().await.unwrap();
    let metrics = metrics_event.metrics.unwrap();
    
    assert!(metrics.network.is_some());
    let network = metrics.network.unwrap();
    
    // Network bytes should be non-negative
    assert!(network.rx_bytes_total >= 0);
    assert!(network.tx_bytes_total >= 0);
    assert!(network.rx_bytes_per_sec >= 0);
    assert!(network.tx_bytes_per_sec >= 0);
}

#[tokio::test]
async fn test_disk_metrics_collection() {
    let mut collector = SystemMetricsCollector::with_config(
        "test-agent".to_string(),
        SystemMetricsCollectorConfig {
            include_disk_details: true,
            ..Default::default()
        }
    );
    
    let metrics_event = collector.collect_metrics().await.unwrap();
    let metrics = metrics_event.metrics.unwrap();
    
    assert!(metrics.disk.is_some());
    let disk = metrics.disk.unwrap();
    
    // Disk metrics should be reasonable
    assert!(disk.available_bytes <= disk.total_bytes);
    assert!(disk.total_bytes > 0);
    assert!(disk.read_bytes_total >= 0);
    assert!(disk.write_bytes_total >= 0);
}

#[tokio::test]
async fn test_process_metrics_collection() {
    let mut collector = SystemMetricsCollector::with_config(
        "test-agent".to_string(),
        SystemMetricsCollectorConfig {
            include_process_details: true,
            ..Default::default()
        }
    );
    
    let metrics_event = collector.collect_metrics().await.unwrap();
    let metrics = metrics_event.metrics.unwrap();
    
    assert!(metrics.process.is_some());
    let process = metrics.process.unwrap();
    
    // Process metrics should be reasonable
    assert!(process.cpu_usage_percent >= 0.0);
    assert!(process.memory_bytes > 0);
    assert!(process.uptime_seconds >= 0);
    assert!(process.thread_count > 0);
}

#[tokio::test]
async fn test_metrics_command_handling() {
    use proxy_core::pb::{MetricsCommand, MetricsConfig, metrics_command};
    
    let mut collector = SystemMetricsCollector::new("test-agent".to_string());
    
    let (metrics_tx, _metrics_rx) = mpsc::channel(10);
    let (command_tx, command_rx) = mpsc::channel(10);
    
    // Send configuration update command
    let config_command = MetricsCommand {
        command: Some(metrics_command::Command::Config(MetricsConfig {
            collection_interval_seconds: 10,
            include_network_details: false,
            include_disk_details: false,
            include_process_details: true,
        }))
    };
    
    command_tx.send(config_command).await.unwrap();
    
    // Start streaming briefly to process the command
    let streaming_handle = tokio::spawn(async move {
        let _ = collector.start_streaming(metrics_tx, command_rx).await;
    });
    
    // Give it time to process the command
    tokio::time::sleep(Duration::from_millis(100)).await;
    streaming_handle.abort();
    
    // The configuration should have been updated (we can't easily verify this without exposing internal state)
    // This test mainly ensures the command handling doesn't panic
}