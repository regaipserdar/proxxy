//! System Metrics Collection Module
//! 
//! This module provides functionality to collect system metrics including CPU usage,
//! memory consumption, network I/O, disk I/O, and process-specific metrics using sysinfo.
//! It supports gRPC streaming with configurable intervals and dynamic configuration updates.

use crate::pb::{SystemMetricsEvent, SystemMetrics, NetworkMetrics, DiskMetrics, ProcessMetrics, NetworkInterface, MetricsCommand, MetricsConfig};
use sysinfo::{System, SystemExt, CpuExt, NetworkExt, DiskExt, ProcessExt, PidExt};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use tokio::time::interval;
use tracing::{debug, warn, error, info};

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
    /// Previous network stats for calculating rates
    prev_network_stats: HashMap<String, (u64, u64)>, // interface -> (rx_bytes, tx_bytes)
    /// Previous disk stats for calculating rates
    prev_disk_stats: HashMap<String, (u64, u64)>, // disk -> (read_bytes, write_bytes)
    /// Last collection timestamp for rate calculations
    last_collection: Option<Instant>,
    /// Process ID for collecting process-specific metrics
    process_id: u32,
}

impl SystemMetricsCollector {
    /// Create a new SystemMetricsCollector
    pub fn new(agent_id: String) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let process_id = std::process::id();
        
        Self {
            system,
            agent_id,
            config: SystemMetricsCollectorConfig::default(),
            prev_network_stats: HashMap::new(),
            prev_disk_stats: HashMap::new(),
            last_collection: None,
            process_id,
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
    
    /// Start streaming system metrics via gRPC
    pub async fn start_streaming<S>(
        &mut self,
        mut metrics_sender: tokio::sync::mpsc::Sender<SystemMetricsEvent>,
        mut command_receiver: tokio::sync::mpsc::Receiver<MetricsCommand>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: futures::Stream<Item = Result<MetricsCommand, tonic::Status>> + Send + 'static,
    {
        info!("Starting system metrics streaming for agent: {}", self.agent_id);
        
        let mut interval = interval(Duration::from_secs(self.config.collection_interval_seconds));
        let mut enabled = true;
        
        loop {
            tokio::select! {
                _ = interval.tick(), if enabled => {
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
        
        info!("System metrics streaming stopped for agent: {}", self.agent_id);
        Ok(())
    }
    
    /// Collect current system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetricsEvent, Box<dyn std::error::Error + Send + Sync>> {
        // Refresh system information
        self.system.refresh_all();
        
        let now = Instant::now();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        
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
    
    /// Collect network metrics with rate calculations
    fn collect_network_metrics(&mut self, now: Instant) -> NetworkMetrics {
        let networks = self.system.networks();
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        let mut total_rx_rate = 0u64;
        let mut total_tx_rate = 0u64;
        let mut interfaces = Vec::new();
        
        let time_delta = self.last_collection
            .map(|last| now.duration_since(last).as_secs_f64())
            .unwrap_or(1.0);
        
        for (interface_name, network) in networks {
            let rx_bytes = network.total_received();
            let tx_bytes = network.total_transmitted();
            
            total_rx += rx_bytes;
            total_tx += tx_bytes;
            
            // Calculate rates if we have previous data
            let (rx_rate, tx_rate) = if let Some((prev_rx, prev_tx)) = self.prev_network_stats.get(interface_name) {
                let rx_rate = ((rx_bytes.saturating_sub(*prev_rx)) as f64 / time_delta) as u64;
                let tx_rate = ((tx_bytes.saturating_sub(*prev_tx)) as f64 / time_delta) as u64;
                (rx_rate, tx_rate)
            } else {
                (0, 0)
            };
            
            total_rx_rate += rx_rate;
            total_tx_rate += tx_rate;
            
            interfaces.push(NetworkInterface {
                name: interface_name.clone(),
                rx_bytes,
                tx_bytes,
            });
            
            // Update previous stats
            self.prev_network_stats.insert(interface_name.clone(), (rx_bytes, tx_bytes));
        }
        
        NetworkMetrics {
            rx_bytes_total: total_rx,
            tx_bytes_total: total_tx,
            rx_bytes_per_sec: total_rx_rate,
            tx_bytes_per_sec: total_tx_rate,
            interfaces,
        }
    }
    
    /// Collect disk metrics with rate calculations
    fn collect_disk_metrics(&mut self, now: Instant) -> DiskMetrics {
        let disks = self.system.disks();
        let mut total_read = 0u64;
        let mut total_write = 0u64;
        let mut total_read_rate = 0u64;
        let mut total_write_rate = 0u64;
        let mut total_available = 0u64;
        let mut total_space = 0u64;
        
        let time_delta = self.last_collection
            .map(|last| now.duration_since(last).as_secs_f64())
            .unwrap_or(1.0);
        
        for disk in disks {
            let disk_name = disk.name().to_string_lossy().to_string();
            
            // Note: sysinfo doesn't provide read/write bytes directly
            // In a real implementation, you might need to read from /proc/diskstats on Linux
            // For now, we'll use placeholder values
            let read_bytes = 0u64; // Would need platform-specific implementation
            let write_bytes = 0u64; // Would need platform-specific implementation
            
            total_available += disk.available_space();
            total_space += disk.total_space();
            
            // Calculate rates if we have previous data
            let (read_rate, write_rate) = if let Some((prev_read, prev_write)) = self.prev_disk_stats.get(&disk_name) {
                let read_rate = ((read_bytes.saturating_sub(*prev_read)) as f64 / time_delta) as u64;
                let write_rate = ((write_bytes.saturating_sub(*prev_write)) as f64 / time_delta) as u64;
                (read_rate, write_rate)
            } else {
                (0, 0)
            };
            
            total_read += read_bytes;
            total_write += write_bytes;
            total_read_rate += read_rate;
            total_write_rate += write_rate;
            
            // Update previous stats
            self.prev_disk_stats.insert(disk_name, (read_bytes, write_bytes));
        }
        
        DiskMetrics {
            read_bytes_total: total_read,
            write_bytes_total: total_write,
            read_bytes_per_sec: total_read_rate,
            write_bytes_per_sec: total_write_rate,
            available_bytes: total_available,
            total_bytes: total_space,
        }
    }
    
    /// Collect process-specific metrics
    fn collect_process_metrics(&self) -> ProcessMetrics {
        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(self.process_id)) {
            ProcessMetrics {
                cpu_usage_percent: process.cpu_usage(),
                memory_bytes: process.memory(),
                uptime_seconds: process.run_time(),
                thread_count: process.tasks().len() as u32,
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
                if new_config.collection_interval_seconds != self.config.collection_interval_seconds {
                    *interval = interval(Duration::from_secs(new_config.collection_interval_seconds));
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
        assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0, "CPU usage should be between 0-100%");
        
        // Test that memory values are non-negative
        let memory_used = collector.system.used_memory();
        let memory_total = collector.system.total_memory();
        assert!(memory_used <= memory_total, "Used memory should not exceed total memory");
        assert!(memory_total > 0, "Total memory should be positive");
    }
}