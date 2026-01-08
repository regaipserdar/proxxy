use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AgentInfo, AgentStatus, OrchestratorError};

/// Health status of the orchestrator system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: SystemStatus,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub total_agents: usize,
    pub online_agents: usize,
    pub offline_agents: usize,
    pub degraded_agents: usize,
    pub database_status: ComponentStatus,
    pub grpc_server_status: ComponentStatus,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
}

/// Overall system status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Status of individual components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    Up,
    Down,
    Degraded,
    Unknown,
}

/// Health checker for monitoring system and agent health
pub struct HealthChecker {
    check_interval: Duration,
    agent_timeout: Duration,
    start_time: Instant,
    is_running: Arc<RwLock<bool>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(check_interval_seconds: u64, agent_timeout_seconds: u64) -> Self {
        Self {
            check_interval: Duration::from_secs(check_interval_seconds),
            agent_timeout: Duration::from_secs(agent_timeout_seconds),
            start_time: Instant::now(),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the health checking process
    pub async fn start(
        &self,
        agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    ) -> Result<(), OrchestratorError> {
        tracing::info!("Starting health checker with interval: {:?}", self.check_interval);
        
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(OrchestratorError::HealthCheck(
                "Health checker is already running".to_string(),
            ));
        }
        *is_running = true;
        drop(is_running);
        
        let agents_clone = agents.clone();
        let check_interval = self.check_interval;
        let agent_timeout = self.agent_timeout;
        let is_running_clone = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(check_interval);
            
            loop {
                interval_timer.tick().await;
                
                // Check if we should stop
                {
                    let running = is_running_clone.read().await;
                    if !*running {
                        tracing::info!("Health checker stopping");
                        break;
                    }
                }
                
                // Perform health checks
                if let Err(e) = Self::check_agent_health(&agents_clone, agent_timeout).await {
                    tracing::error!("Health check failed: {}", e);
                }
            }
        });
        
        tracing::info!("Health checker started successfully");
        Ok(())
    }
    
    /// Stop the health checking process
    pub async fn stop(&self) -> Result<(), OrchestratorError> {
        tracing::info!("Stopping health checker");
        
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        tracing::info!("Health checker stopped");
        Ok(())
    }
    
    /// Check the health of all agents
    async fn check_agent_health(
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
        timeout: Duration,
    ) -> Result<(), OrchestratorError> {
        let mut agents_guard = agents.write().await;
        let now = Utc::now();
        let timeout_threshold = now - chrono::Duration::from_std(timeout)
            .map_err(|e| OrchestratorError::HealthCheck(format!("Duration conversion error: {}", e)))?;
        
        let mut updated_agents = Vec::new();
        
        for (agent_id, agent) in agents_guard.iter() {
            let new_status = if agent.last_heartbeat < timeout_threshold {
                match agent.status {
                    AgentStatus::Online => {
                        tracing::warn!("Agent {} is now offline (last heartbeat: {})", 
                                     agent_id, agent.last_heartbeat);
                        AgentStatus::Offline
                    }
                    AgentStatus::Degraded => {
                        tracing::warn!("Agent {} is now offline (was degraded, last heartbeat: {})", 
                                     agent_id, agent.last_heartbeat);
                        AgentStatus::Offline
                    }
                    status => status, // Keep existing offline/unknown status
                }
            } else {
                // Agent is responsive, keep current status or mark as online if it was offline
                match agent.status {
                    AgentStatus::Offline => {
                        tracing::info!("Agent {} is back online", agent_id);
                        AgentStatus::Online
                    }
                    status => status,
                }
            };
            
            if new_status != agent.status {
                let mut updated_agent = agent.clone();
                updated_agent.status = new_status;
                updated_agents.push((agent_id.clone(), updated_agent));
            }
        }
        
        // Apply status updates
        for (agent_id, updated_agent) in updated_agents {
            agents_guard.insert(agent_id, updated_agent);
        }
        
        tracing::debug!("Health check completed for {} agents", agents_guard.len());
        Ok(())
    }
    
    /// Get current system health status
    pub async fn get_health_status(
        &self,
        agents: &Arc<RwLock<HashMap<String, AgentInfo>>>,
    ) -> HealthStatus {
        let agents_guard = agents.read().await;
        
        let total_agents = agents_guard.len();
        let mut online_agents = 0;
        let mut offline_agents = 0;
        let mut degraded_agents = 0;
        
        for agent in agents_guard.values() {
            match agent.status {
                AgentStatus::Online => online_agents += 1,
                AgentStatus::Offline => offline_agents += 1,
                AgentStatus::Degraded => degraded_agents += 1,
                AgentStatus::Unknown => {} // Don't count unknown agents
            }
        }
        
        // Determine overall system status
        let overall_status = if total_agents == 0 {
            SystemStatus::Unknown
        } else if offline_agents == total_agents {
            SystemStatus::Unhealthy
        } else if offline_agents > 0 || degraded_agents > 0 {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };
        
        // Get system metrics (simplified for this implementation)
        let uptime_seconds = self.start_time.elapsed().as_secs();
        let memory_usage_mb = Self::get_memory_usage();
        let cpu_usage_percent = Self::get_cpu_usage();
        
        HealthStatus {
            overall_status,
            timestamp: Utc::now(),
            uptime_seconds,
            total_agents,
            online_agents,
            offline_agents,
            degraded_agents,
            database_status: ComponentStatus::Up, // Simplified
            grpc_server_status: ComponentStatus::Up, // Simplified
            memory_usage_mb,
            cpu_usage_percent,
        }
    }
    
    /// Get current memory usage (simplified implementation)
    fn get_memory_usage() -> u64 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder value
        100 // MB
    }
    
    /// Get current CPU usage (simplified implementation)
    fn get_cpu_usage() -> f32 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder value
        15.5 // Percent
    }
    
    /// Check if the health checker is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            overall_status: SystemStatus::Unknown,
            timestamp: Utc::now(),
            uptime_seconds: 0,
            total_agents: 0,
            online_agents: 0,
            offline_agents: 0,
            degraded_agents: 0,
            database_status: ComponentStatus::Unknown,
            grpc_server_status: ComponentStatus::Unknown,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseManager;
    
    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new(30, 300);
        assert!(!checker.is_running().await);
    }
    
    #[tokio::test]
    async fn test_health_status_calculation() {
        let checker = HealthChecker::new(30, 300);
        let agents = Arc::new(RwLock::new(HashMap::new()));
        
        // Add some test agents
        {
            let mut agents_guard = agents.write().await;
            
            agents_guard.insert("agent1".to_string(), AgentInfo {
                id: "agent1".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                status: AgentStatus::Online,
                last_heartbeat: Utc::now(),
                version: "1.0.0".to_string(),
                capabilities: vec!["http".to_string()],
            });
            
            agents_guard.insert("agent2".to_string(), AgentInfo {
                id: "agent2".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8081,
                status: AgentStatus::Offline,
                last_heartbeat: Utc::now() - chrono::Duration::hours(1),
                version: "1.0.0".to_string(),
                capabilities: vec!["http".to_string()],
            });
        }
        
        let health = checker.get_health_status(&agents).await;
        
        assert_eq!(health.total_agents, 2);
        assert_eq!(health.online_agents, 1);
        assert_eq!(health.offline_agents, 1);
        assert_eq!(health.overall_status, SystemStatus::Degraded);
    }
    
    #[tokio::test]
    async fn test_agent_timeout_detection() {
        let agents = Arc::new(RwLock::new(HashMap::new()));
        let timeout = Duration::from_secs(60);
        
        // Add an agent with old heartbeat
        {
            let mut agents_guard = agents.write().await;
            agents_guard.insert("old-agent".to_string(), AgentInfo {
                id: "old-agent".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                status: AgentStatus::Online,
                last_heartbeat: Utc::now() - chrono::Duration::hours(2),
                version: "1.0.0".to_string(),
                capabilities: vec!["http".to_string()],
            });
        }
        
        // Run health check
        HealthChecker::check_agent_health(&agents, timeout).await.unwrap();
        
        // Verify agent status was updated to offline
        let agents_guard = agents.read().await;
        let agent = agents_guard.get("old-agent").unwrap();
        assert_eq!(agent.status, AgentStatus::Offline);
    }
}