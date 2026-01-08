use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{AgentMetrics, TrafficData, DatabaseManager, OrchestratorError};

/// Aggregated metrics for the orchestrator system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub active_agents: u32,
    pub total_data_processed_mb: f64,
    pub agent_metrics: HashMap<String, AgentSummary>,
}

/// Summary metrics for a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub agent_id: String,
    pub status: String,
    pub requests_handled: u64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub data_processed_mb: f64,
    pub uptime_hours: f64,
    pub last_seen: DateTime<Utc>,
}

/// Time-series metrics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metric_type: MetricType,
    pub agent_id: Option<String>,
}

/// Types of metrics we track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    RequestsPerSecond,
    ResponseTime,
    ErrorRate,
    ActiveConnections,
    MemoryUsage,
    CpuUsage,
    DataThroughput,
}

/// Metrics collector and aggregator
pub struct MetricsCollector {
    database: Arc<DatabaseManager>,
    metrics_cache: Arc<RwLock<HashMap<String, Vec<MetricsDataPoint>>>>,
    cache_duration: Duration,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(database: Arc<DatabaseManager>) -> Self {
        Self {
            database,
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_duration: Duration::hours(1), // Cache metrics for 1 hour
        }
    }
    
    /// Collect and aggregate system metrics
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics, OrchestratorError> {
        tracing::debug!("Collecting system metrics");
        
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        
        // Get recent traffic data for all agents
        let recent_traffic = self.get_recent_traffic_data(one_hour_ago, now).await?;
        
        // Get recent agent metrics
        let recent_agent_metrics = self.get_recent_agent_metrics(one_hour_ago, now).await?;
        
        // Calculate aggregated metrics
        let total_requests = recent_traffic.len() as u64;
        let requests_per_second = total_requests as f64 / 3600.0; // Per hour / 3600 seconds
        
        let (average_response_time_ms, error_rate, total_data_processed_mb) = 
            self.calculate_traffic_metrics(&recent_traffic);
        
        let active_agents = recent_agent_metrics.len() as u32;
        
        // Build agent summaries
        let agent_metrics = self.build_agent_summaries(&recent_traffic, &recent_agent_metrics).await?;
        
        let system_metrics = SystemMetrics {
            timestamp: now,
            total_requests,
            requests_per_second,
            average_response_time_ms,
            error_rate,
            active_agents,
            total_data_processed_mb,
            agent_metrics,
        };
        
        tracing::debug!("System metrics collected: {} requests, {} agents active", 
                       total_requests, active_agents);
        
        Ok(system_metrics)
    }
    
    /// Get metrics for a specific time range
    pub async fn get_metrics_time_series(
        &self,
        metric_type: MetricType,
        agent_id: Option<String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        tracing::debug!("Getting time series for metric: {:?}, agent: {:?}", 
                       metric_type, agent_id);
        
        // Check cache first
        let cache_key = format!("{:?}_{:?}_{}_{}",
                               metric_type, agent_id, start_time.timestamp(), end_time.timestamp());
        
        {
            let cache = self.metrics_cache.read().await;
            if let Some(cached_data) = cache.get(&cache_key) {
                if let Some(first_point) = cached_data.first() {
                    if Utc::now() - first_point.timestamp < self.cache_duration {
                        tracing::debug!("Returning cached metrics data");
                        return Ok(cached_data.clone());
                    }
                }
            }
        }
        
        // Fetch from database
        let data_points = match metric_type {
            MetricType::RequestsPerSecond => {
                self.get_requests_per_second_series(agent_id, start_time, end_time).await?
            }
            MetricType::ResponseTime => {
                self.get_response_time_series(agent_id, start_time, end_time).await?
            }
            MetricType::ErrorRate => {
                self.get_error_rate_series(agent_id, start_time, end_time).await?
            }
            MetricType::ActiveConnections => {
                self.get_active_connections_series(agent_id, start_time, end_time).await?
            }
            MetricType::MemoryUsage => {
                self.get_memory_usage_series(agent_id, start_time, end_time).await?
            }
            MetricType::CpuUsage => {
                self.get_cpu_usage_series(agent_id, start_time, end_time).await?
            }
            MetricType::DataThroughput => {
                self.get_data_throughput_series(agent_id, start_time, end_time).await?
            }
        };
        
        // Cache the results
        {
            let mut cache = self.metrics_cache.write().await;
            cache.insert(cache_key, data_points.clone());
        }
        
        Ok(data_points)
    }
    
    /// Get recent traffic data from database
    async fn get_recent_traffic_data(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TrafficData>, OrchestratorError> {
        // This would be implemented with actual database queries
        // For now, return empty vector as placeholder
        tracing::debug!("Fetching traffic data from {} to {}", start_time, end_time);
        Ok(Vec::new())
    }
    
    /// Get recent agent metrics from database
    async fn get_recent_agent_metrics(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AgentMetrics>, OrchestratorError> {
        // This would be implemented with actual database queries
        // For now, return empty vector as placeholder
        tracing::debug!("Fetching agent metrics from {} to {}", start_time, end_time);
        Ok(Vec::new())
    }
    
    /// Calculate traffic-based metrics
    fn calculate_traffic_metrics(&self, traffic_data: &[TrafficData]) -> (f64, f64, f64) {
        if traffic_data.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let total_response_time: u64 = traffic_data.iter()
            .map(|t| t.processing_time_ms)
            .sum();
        let average_response_time = total_response_time as f64 / traffic_data.len() as f64;
        
        let error_count = traffic_data.iter()
            .filter(|t| t.status_code.map_or(true, |code| code >= 400))
            .count();
        let error_rate = error_count as f64 / traffic_data.len() as f64;
        
        let total_data_bytes: u64 = traffic_data.iter()
            .map(|t| t.request_size + t.response_size.unwrap_or(0))
            .sum();
        let total_data_mb = total_data_bytes as f64 / (1024.0 * 1024.0);
        
        (average_response_time, error_rate, total_data_mb)
    }
    
    /// Build agent summary metrics
    async fn build_agent_summaries(
        &self,
        traffic_data: &[TrafficData],
        agent_metrics: &[AgentMetrics],
    ) -> Result<HashMap<String, AgentSummary>, OrchestratorError> {
        let mut summaries = HashMap::new();
        
        // Group traffic data by agent
        let mut agent_traffic: HashMap<String, Vec<&TrafficData>> = HashMap::new();
        for traffic in traffic_data {
            agent_traffic.entry(traffic.agent_id.clone())
                .or_insert_with(Vec::new)
                .push(traffic);
        }
        
        // Group metrics by agent
        let mut agent_metrics_map: HashMap<String, Vec<&AgentMetrics>> = HashMap::new();
        for metrics in agent_metrics {
            agent_metrics_map.entry(metrics.agent_id.clone())
                .or_insert_with(Vec::new)
                .push(metrics);
        }
        
        // Build summaries for each agent
        for (agent_id, traffic_list) in agent_traffic {
            let requests_handled = traffic_list.len() as u64;
            let traffic_data_vec: Vec<TrafficData> = traffic_list.into_iter().cloned().collect();
            let (avg_response_time, error_rate, data_processed) = 
                self.calculate_traffic_metrics(&traffic_data_vec);
            
            let last_seen = traffic_data_vec.iter()
                .map(|t| t.timestamp)
                .max()
                .unwrap_or_else(Utc::now);
            
            // Calculate uptime (simplified)
            let uptime_hours = 24.0; // Placeholder
            
            summaries.insert(agent_id.clone(), AgentSummary {
                agent_id: agent_id.clone(),
                status: "online".to_string(), // Simplified
                requests_handled,
                average_response_time_ms: avg_response_time,
                error_rate,
                data_processed_mb: data_processed,
                uptime_hours,
                last_seen,
            });
        }
        
        Ok(summaries)
    }
    
    // Placeholder implementations for time series methods
    async fn get_requests_per_second_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_response_time_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_error_rate_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_active_connections_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_memory_usage_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_cpu_usage_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    async fn get_data_throughput_series(
        &self,
        _agent_id: Option<String>,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricsDataPoint>, OrchestratorError> {
        Ok(Vec::new())
    }
    
    /// Clear old cached metrics
    pub async fn clear_old_cache(&self) {
        let mut cache = self.metrics_cache.write().await;
        let cutoff_time = Utc::now() - self.cache_duration;
        
        cache.retain(|_, data_points| {
            data_points.first()
                .map_or(false, |point| point.timestamp > cutoff_time)
        });
        
        tracing::debug!("Cleared old metrics cache entries");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseManager;
    
    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let database = Arc::new(DatabaseManager::new("sqlite::memory:").await.unwrap());
        let _collector = MetricsCollector::new(database);
        
        // Collector should be created successfully
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_traffic_metrics_calculation() {
        let database = Arc::new(DatabaseManager::new("sqlite::memory:").await.unwrap());
        let collector = MetricsCollector::new(database);
        
        let traffic_data = vec![
            TrafficData {
                id: "1".to_string(),
                agent_id: "agent1".to_string(),
                timestamp: Utc::now(),
                method: "GET".to_string(),
                url: "http://example.com".to_string(),
                status_code: Some(200),
                request_size: 1024,
                response_size: Some(2048),
                processing_time_ms: 100,
            },
            TrafficData {
                id: "2".to_string(),
                agent_id: "agent1".to_string(),
                timestamp: Utc::now(),
                method: "POST".to_string(),
                url: "http://example.com/api".to_string(),
                status_code: Some(500),
                request_size: 512,
                response_size: Some(256),
                processing_time_ms: 200,
            },
        ];
        
        let (avg_response_time, error_rate, data_mb) = collector.calculate_traffic_metrics(&traffic_data);
        
        assert_eq!(avg_response_time, 150.0); // (100 + 200) / 2
        assert_eq!(error_rate, 0.5); // 1 error out of 2 requests
        assert!(data_mb > 0.0); // Should have some data processed
    }
    
    #[test]
    fn test_metric_type_serialization() {
        let metric_type = MetricType::RequestsPerSecond;
        let serialized = serde_json::to_string(&metric_type).unwrap();
        let deserialized: MetricType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(metric_type, deserialized);
    }
}