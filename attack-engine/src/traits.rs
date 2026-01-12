//! Core traits for the attack engine

use crate::{AttackRequest, AttackResultData, AttackError, AgentInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Trait for executing attack requests on agents
#[async_trait]
pub trait AttackExecutor {
    /// Execute a single attack request on a specific agent
    async fn execute_request(
        &self,
        request: AttackRequest,
        agent_id: &str,
    ) -> Result<AttackResultData, AttackError>;
    
    /// Execute multiple requests across multiple agents
    async fn execute_batch(
        &self,
        requests: Vec<AttackRequest>,
    ) -> Vec<Result<AttackResultData, AttackError>>;
    
    /// Get available agents for attack execution
    async fn get_available_agents(&self) -> Result<Vec<AgentInfo>, AttackError>;
}

/// Trait for managing agent selection and load balancing
#[async_trait]
pub trait AgentManager {
    /// Select the best agent for a request based on load and availability
    async fn select_agent(&self, agents: &[String]) -> Result<String, AttackError>;
    
    /// Get current load information for all agents
    async fn get_agent_loads(&self) -> Result<HashMap<String, f64>, AttackError>;
    
    /// Update agent status
    async fn update_agent_status(&self, agent_id: &str, status: crate::AgentStatus);
    
    /// Get agent information
    async fn get_agent_info(&self, agent_id: &str) -> Result<AgentInfo, AttackError>;
    
    /// Check if a specific agent is available
    async fn is_agent_available(&self, agent_id: &str) -> Result<bool, AttackError>;
}

/// Trait for distributing payloads across agents
pub trait PayloadDistributor {
    /// Distribute payloads across agents using the specified strategy
    fn distribute_payloads(
        &self,
        payloads: Vec<String>,
        agents: &[String],
        strategy: &crate::DistributionStrategy,
    ) -> Result<HashMap<String, Vec<String>>, AttackError>;
    
    /// Calculate optimal batch size for the given agents and payloads
    fn calculate_batch_size(&self, payload_count: usize, agent_count: usize) -> usize;
}

/// Trait for result processing and aggregation
#[async_trait]
pub trait ResultProcessor {
    /// Process a single attack result
    async fn process_result(&self, result: AttackResultData) -> Result<(), AttackError>;
    
    /// Process multiple results in batch
    async fn process_batch(&self, results: Vec<AttackResultData>) -> Result<(), AttackError>;
    
    /// Get results for a specific attack
    async fn get_attack_results(&self, attack_id: Uuid) -> Result<Vec<AttackResultData>, AttackError>;
    
    /// Get result statistics for an attack
    async fn get_attack_statistics(&self, attack_id: Uuid) -> Result<AttackStatistics, AttackError>;
}

/// Statistics for attack results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttackStatistics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub status_code_distribution: HashMap<i32, u64>,
    pub error_distribution: HashMap<String, u64>,
}

impl Default for AttackStatistics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            min_response_time_ms: 0,
            max_response_time_ms: 0,
            status_code_distribution: HashMap::new(),
            error_distribution: HashMap::new(),
        }
    }
}

impl AttackStatistics {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
    
    /// Get the most common status code
    pub fn most_common_status_code(&self) -> Option<i32> {
        self.status_code_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(code, _)| *code)
    }
    
    /// Get the most common error
    pub fn most_common_error(&self) -> Option<&String> {
        self.error_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error, _)| error)
    }
}