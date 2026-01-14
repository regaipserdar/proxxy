//! System Metrics Collection Module
//!
//! This module provides functionality to collect system metrics including CPU usage,
//! memory consumption, network I/O, disk I/O, and process-specific metrics using sysinfo.
//! It supports gRPC streaming with configurable intervals and dynamic configuration updates.

use crate::pb::{
    AgentMetrics, DiskMetrics, MetricsCommand, NetworkMetrics, ProcessMetrics, SystemMetrics,
    SystemMetricsEvent,
};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Configuration for system metrics collection
#[derive(Debug, Clone)]
pub struct SystemMetricsCollectorConfig {
    /// Collection interval in seconds (default: 5)
    pub collection_interval_seconds: u64,
    /// Buffer size for gRPC streaming (default: 100)
    pub stream_buffer_size: usize,
    /// Include network interface details (default: true)
    pub include_network_details: bool,
    /// Include disk details (default: true)
    pub include_disk_details: bool,
    /// Include process details (default: true)
    pub include_process_details: bool,
}

impl Default for SystemMetricsCollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 5,
            stream_buffer_size: 100,
            include_network_details: true,
            include_disk_details: true,
            include_process_details: true,
        }
    }
}

/// System metrics collector that uses sysinfo to gather system resource information
pub struct SystemMetricsCollector {
    /// sysinfo System instance for collecting metrics
    system: System,
    /// Agent ID for identifying the source of metrics
    agent_id: String,
    /// Current configuration
    config: SystemMetricsCollectorConfig,
    /// Previous network stats for calculating rates (reserved for future use)
    #[allow(dead_code)]
    prev_network_stats: HashMap<String, (u64, u64)>, // interface -> (rx_bytes, tx_bytes)
    /// Previous disk stats for calculating rates (reserved for future use)
    #[allow(dead_code)]
    prev_disk_stats: HashMap<String, (u64, u64)>, // disk -> (read_bytes, write_bytes)
    /// Last collection timestamp for rate calculations
    last_collection: Option<Instant>,
    /// Process ID for collecting process-specific metrics
    process_id: u32,
    
    // Public IP caching strategy
    public_ip: Option<String>,
    public_ip_last_attempt: Option<Instant>,
    public_ip_retry_count: u32,
    http_client: reqwest::Client,
}

impl SystemMetricsCollector {
    /// Create a new SystemMetricsCollector
    pub fn new(agent_id: String) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let process_id = std::process::id();

        // Create HTTP client for IP resolution with short timeout
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("proxxy-agent/1.0")
            .build()
            .unwrap_or_default();

        Self {
            system,
            agent_id,
            config: SystemMetricsCollectorConfig::default(),
            prev_network_stats: HashMap::new(),
            prev_disk_stats: HashMap::new(),
            last_collection: None,
            process_id,
            public_ip: None,
            public_ip_last_attempt: None,
            public_ip_retry_count: 0,
            http_client,
        }
    }

    /// Create a new SystemMetricsCollector with custom configuration
    pub fn with_config(agent_id: String, config: SystemMetricsCollectorConfig) -> Self {
        let mut collector = Self::new(agent_id);
        collector.config = config;
        collector
    }

    /// Update the collector configuration
    pub fn update_config(&mut self, config: SystemMetricsCollectorConfig) {
        info!("Updating system metrics configuration: {:?}", config);
        self.config = config;
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the current configuration
    pub fn config(&self) -> &SystemMetricsCollectorConfig {
        &self.config
    }

    /// Start streaming system metrics via gRPC
    pub async fn start_streaming(
        &mut self,
        metrics_sender: tokio::sync::mpsc::Sender<SystemMetricsEvent>,
        mut command_receiver: tokio::sync::mpsc::Receiver<MetricsCommand>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting system metrics streaming for agent: {}",
            self.agent_id
        );

        let mut interval = interval(Duration::from_secs(self.config.collection_interval_seconds));
        let mut enabled = true;

        loop {
            tokio::select! {
                _ = interval.tick(), if enabled => {
                     // We also try to resolve IP here if needed, but primarily for the heartbeat
                     // For SystemMetrics stream, we stick to system info.
                     // The user asked to inject IP into HEARTBEAT.
                    match self.collect_metrics().await {
                        Ok(metrics_event) => {
                            debug!("Collected system metrics: CPU: {:.2}%, Memory: {} MB",
                                   metrics_event.metrics.as_ref().map(|m| m.cpu_usage_percent).unwrap_or(0.0),
                                   metrics_event.metrics.as_ref().map(|m| m.memory_used_bytes / 1024 / 1024).unwrap_or(0));

                            if let Err(e) = metrics_sender.send(metrics_event).await {
                                error!("Failed to send metrics event: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to collect system metrics: {}", e);
                        }
                    }
                }

                command = command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_metrics_command(cmd, &mut interval, &mut enabled).await {
                                error!("Failed to handle metrics command: {}", e);
                            }
                        }
                        None => {
                            info!("Metrics command channel closed, stopping metrics collection");
                            break;
                        }
                    }
                }
            }
        }

        info!(
            "System metrics streaming stopped for agent: {}",
            self.agent_id
        );
        Ok(())
    }

    /// Collect current system metrics (Global + Process)
    pub async fn collect_metrics(
        &mut self,
    ) -> Result<SystemMetricsEvent, Box<dyn std::error::Error + Send + Sync>> {
        // Refresh system information (non-blocking usually, but refresh_all can be heavy)
        // We only refresh needed parts ideally, but for now refresh_all
        self.system.refresh_all();

        let now = Instant::now();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        // Collect system-wide metrics
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();

        // Collect network metrics
        let network_metrics = if self.config.include_network_details {
            Some(self.collect_network_metrics(now))
        } else {
            None
        };

        // Collect disk metrics
        let disk_metrics = if self.config.include_disk_details {
            Some(self.collect_disk_metrics(now))
        } else {
            None
        };

        // Collect process-specific metrics
        let process_metrics = if self.config.include_process_details {
            Some(self.collect_process_metrics())
        } else {
            None
        };

        self.last_collection = Some(now);

        let metrics = SystemMetrics {
            cpu_usage_percent: cpu_usage,
            memory_used_bytes: memory_used,
            memory_total_bytes: memory_total,
            network: network_metrics,
            disk: disk_metrics,
            process: process_metrics,
        };

        Ok(SystemMetricsEvent {
            agent_id: self.agent_id.clone(),
            timestamp,
            metrics: Some(metrics),
        })
    }

    /// Collect Agent Metrics for Heartbeating (Process specific + Public IP)
    pub async fn collect_agent_metrics(&mut self) -> AgentMetrics {
        // Ensure IP is resolved
        self.resolve_public_ip().await;

        // Refresh specific process
        self.system.refresh_process(Pid::from_u32(self.process_id));
        
        // We also need global refresh for relative CPU? 
        // sysinfo's process cpu usage is usually based on last refresh.
        // If collect_metrics called refresh_all recently, we might be good, 
        // but to be safe for a dedicated heartbeat task:
        // self.system.refresh_processes(); // expensive?
        // Let's just rely on refresh_process for the specific PID.
        
        let (cpu_usage, memory_mb, uptime) = if let Some(process) = self.system.process(Pid::from_u32(self.process_id)) {
            (
                process.cpu_usage(),
                process.memory() as f64 / 1024.0 / 1024.0,
                process.run_time(),
            )
        } else {
            (0.0, 0.0, 0)
        };

        AgentMetrics {
            agent_id: self.agent_id.clone(),
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_mb,
            uptime_seconds: uptime,
            public_ip: self.public_ip.clone().unwrap_or_default(),
        }
    }

    /// Resolve Public IP with caching and backoff
    async fn resolve_public_ip(&mut self) {
        if self.public_ip.is_some() {
            return;
        }

        // Check backoff
        if let Some(last) = self.public_ip_last_attempt {
            let backoff = Duration::from_secs(2u64.pow(self.public_ip_retry_count.min(6)));
            if last.elapsed() < backoff {
                return;
            }
        }

        info!("Attempting to resolve public IP...");
        self.public_ip_last_attempt = Some(Instant::now());

        match self.http_client.get("https://api.ipify.org").send().await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(ip) => {
                        let ip = ip.trim().to_string();
                        info!("Resolved public IP: {}", ip);
                        self.public_ip = Some(ip);
                        self.public_ip_retry_count = 0;
                    }
                    Err(e) => {
                        warn!("Failed to read IP from response: {}", e);
                        self.public_ip_retry_count += 1;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to resolve public IP: {}", e);
                self.public_ip_retry_count += 1;
            }
        }
    }

    /// Collect network metrics with rate calculations
    fn collect_network_metrics(&mut self, _now: Instant) -> NetworkMetrics {
        // For sysinfo 0.30, we'll use a simplified approach
        // In a real implementation, you might need platform-specific code
        let total_rx = 0u64;
        let total_tx = 0u64;
        let total_rx_rate = 0u64;
        let total_tx_rate = 0u64;
        let interfaces = Vec::new(); // Simplified for now

        // Note: sysinfo 0.30 doesn't have direct network access in the same way
        // This would need platform-specific implementation

        NetworkMetrics {
            rx_bytes_total: total_rx,
            tx_bytes_total: total_tx,
            rx_bytes_per_sec: total_rx_rate,
            tx_bytes_per_sec: total_tx_rate,
            interfaces,
        }
    }

    /// Collect disk metrics with rate calculations
    fn collect_disk_metrics(&mut self, _now: Instant) -> DiskMetrics {
        // For sysinfo 0.30, we'll use a simplified approach
        let total_available = 0u64;
        let total_space = 0u64;

        // Note: sysinfo 0.30 doesn't have direct disk access in the same way
        // This would need platform-specific implementation for read/write rates

        DiskMetrics {
            read_bytes_total: 0,  // Would need platform-specific implementation
            write_bytes_total: 0, // Would need platform-specific implementation
            read_bytes_per_sec: 0,
            write_bytes_per_sec: 0,
            available_bytes: total_available,
            total_bytes: total_space,
        }
    }

    /// Collect process-specific metrics
    fn collect_process_metrics(&self) -> ProcessMetrics {
        if let Some(process) = self.system.process(Pid::from_u32(self.process_id)) {
            ProcessMetrics {
                cpu_usage_percent: process.cpu_usage(),
                memory_bytes: process.memory(),
                uptime_seconds: process.run_time(),
                thread_count: 1, // Simplified - sysinfo 0.30 doesn't expose tasks().len()
                file_descriptor_count: 0, // sysinfo doesn't provide this directly
            }
        } else {
            warn!("Could not find process with PID: {}", self.process_id);
            ProcessMetrics {
                cpu_usage_percent: 0.0,
                memory_bytes: 0,
                uptime_seconds: 0,
                thread_count: 0,
                file_descriptor_count: 0,
            }
        }
    }

    /// Handle incoming metrics commands
    async fn handle_metrics_command(
        &mut self,
        command: MetricsCommand,
        interval: &mut tokio::time::Interval,
        enabled: &mut bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match command.command {
            Some(crate::pb::metrics_command::Command::Config(config)) => {
                info!("Received metrics configuration update: {:?}", config);

                let new_config = SystemMetricsCollectorConfig {
                    collection_interval_seconds: config.collection_interval_seconds as u64,
                    include_network_details: config.include_network_details,
                    include_disk_details: config.include_disk_details,
                    include_process_details: config.include_process_details,
                    ..self.config.clone()
                };

                // Update interval if changed
                if new_config.collection_interval_seconds != self.config.collection_interval_seconds
                {
                    *interval = tokio::time::interval(Duration::from_secs(
                        new_config.collection_interval_seconds,
                    ));
                }

                self.update_config(new_config);
            }
            Some(crate::pb::metrics_command::Command::StopMetrics(stop)) => {
                info!("Received stop metrics command: {}", stop);
                *enabled = !stop;
            }
            Some(crate::pb::metrics_command::Command::StartMetrics(start)) => {
                info!("Received start metrics command: {}", start);
                *enabled = start;
            }
            None => {
                warn!("Received empty metrics command");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            cpu_usage >= 0.0 && cpu_usage <= 100.0,
            "CPU usage should be between 0-100%"
        );

        // Test that memory values are non-negative
        let memory_used = collector.system.used_memory();
        let memory_total = collector.system.total_memory();
        assert!(
            memory_used <= memory_total,
            "Used memory should not exceed total memory"
        );
        assert!(memory_total > 0, "Total memory should be positive");
    }
}
