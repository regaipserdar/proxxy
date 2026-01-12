//! Core execution engine for attack requests

use crate::{
    AttackRequest, AttackResultData, AttackError, AttackExecutor, AgentManager, 
    PayloadDistributor, ResultProcessor, DistributionStrategy,
    AttackStatistics, ModuleType, AttackContext
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, debug};
use uuid::Uuid;

/// Core attack execution engine
pub struct AttackEngine {
    agent_manager: Arc<dyn AgentManager + Send + Sync>,
    result_processor: Arc<dyn ResultProcessor + Send + Sync>,
    payload_distributor: Arc<dyn PayloadDistributor + Send + Sync>,
    active_attacks: Arc<RwLock<HashMap<Uuid, AttackContext>>>,
    resource_manager: Option<Arc<dyn ResourceManager + Send + Sync>>,
}

/// Resource manager trait for integration with global resource management
#[async_trait]
pub trait ResourceManager {
    /// Request resources for attack execution
    async fn request_attack_resources(
        &self,
        module_type: ModuleType,
        agent_count: usize,
        concurrent_requests: u32,
    ) -> Result<ResourceAllocation, AttackError>;
    
    /// Release allocated resources
    async fn release_resources(&self, allocation_id: Uuid) -> Result<(), AttackError>;
}

/// Resource allocation handle
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub id: Uuid,
    pub module_type: ModuleType,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
}

impl AttackEngine {
    /// Create a new attack engine
    pub fn new(
        agent_manager: Arc<dyn AgentManager + Send + Sync>,
        result_processor: Arc<dyn ResultProcessor + Send + Sync>,
        payload_distributor: Arc<dyn PayloadDistributor + Send + Sync>,
    ) -> Self {
        Self {
            agent_manager,
            result_processor,
            payload_distributor,
            active_attacks: Arc::new(RwLock::new(HashMap::new())),
            resource_manager: None,
        }
    }
    
    /// Set resource manager for global resource coordination
    pub fn with_resource_manager(
        mut self,
        resource_manager: Arc<dyn ResourceManager + Send + Sync>,
    ) -> Self {
        self.resource_manager = Some(resource_manager);
        self
    }
    
    /// Start a new attack with the given context
    pub async fn start_attack(
        &self,
        context: AttackContext,
        requests: Vec<AttackRequest>,
    ) -> Result<(), AttackError> {
        info!("Starting attack {} with {} requests", context.attack_id, requests.len());
        
        // Register the attack
        {
            let mut active_attacks = self.active_attacks.write().await;
            active_attacks.insert(context.attack_id, context.clone());
        }
        
        // Request resources if resource manager is available
        let _resource_allocation = if let Some(ref rm) = self.resource_manager {
            let agent_count = requests.first()
                .map(|r| r.target_agents.len())
                .unwrap_or(1);
            let concurrent_requests = requests.first()
                .map(|r| r.execution_config.concurrent_requests_per_agent)
                .unwrap_or(10);
                
            Some(rm.request_attack_resources(
                context.module_type.clone(),
                agent_count,
                concurrent_requests,
            ).await?)
        } else {
            None
        };
        
        // Execute requests
        let results = self.execute_attack_requests(requests).await;
        
        // Process results
        for result in results {
            match result {
                Ok(attack_result) => {
                    if let Err(e) = self.result_processor.process_result(attack_result).await {
                        error!("Failed to process attack result: {}", e);
                    }
                }
                Err(e) => {
                    error!("Attack request failed: {}", e);
                }
            }
        }
        
        // Clean up resources
        if let Some(allocation) = _resource_allocation {
            if let Some(ref rm) = self.resource_manager {
                if let Err(e) = rm.release_resources(allocation.id).await {
                    error!("Failed to release resources: {}", e);
                }
            }
        }
        
        // Remove from active attacks
        {
            let mut active_attacks = self.active_attacks.write().await;
            active_attacks.remove(&context.attack_id);
        }
        
        info!("Attack {} completed", context.attack_id);
        Ok(())
    }
    
    /// Execute multiple attack requests
    async fn execute_attack_requests(
        &self,
        requests: Vec<AttackRequest>,
    ) -> Vec<Result<AttackResultData, AttackError>> {
        let mut results = Vec::new();
        
        for request in requests {
            // Select best agent for this request
            let agent_id = match self.agent_manager.select_agent(&request.target_agents).await {
                Ok(id) => id,
                Err(e) => {
                    results.push(Err(e));
                    continue;
                }
            };
            
            // Execute the request
            let result = self.execute_single_request(request, &agent_id).await;
            results.push(result);
        }
        
        results
    }
    
    /// Execute a single request on a specific agent
    async fn execute_single_request(
        &self,
        mut request: AttackRequest,
        agent_id: &str,
    ) -> Result<AttackResultData, AttackError> {
        debug!("Executing request {} on agent {}", request.id, agent_id);
        
        // Apply session data if present
        if let Some(ref session) = request.session_data {
            request.request_template.apply_session(session);
        }
        
        // Check agent availability
        if !self.agent_manager.is_agent_available(agent_id).await? {
            return Err(AttackError::AgentUnavailable {
                agent_id: agent_id.to_string(),
            });
        }
        
        // Create attack result
        let start_time = std::time::Instant::now();
        let mut result = AttackResultData::new(
            request.id,
            agent_id.to_string(),
            request.request_template.clone(),
        );
        
        // TODO: Implement actual agent communication
        // For now, return a placeholder result
        let duration = start_time.elapsed().as_millis() as u64;
        
        // Simulate successful execution
        let response = crate::HttpResponseData {
            status_code: 200,
            headers: Some(crate::HttpHeaders {
                headers: HashMap::new(),
            }),
            body: b"Mock response".to_vec(),
            tls: None,
        };
        
        result = result.with_response(response, duration);
        
        debug!("Request {} completed in {}ms", request.id, duration);
        Ok(result)
    }
    
    /// Get statistics for an active attack
    pub async fn get_attack_statistics(
        &self,
        attack_id: Uuid,
    ) -> Result<AttackStatistics, AttackError> {
        self.result_processor.get_attack_statistics(attack_id).await
    }
    
    /// Stop an active attack
    pub async fn stop_attack(&self, attack_id: Uuid) -> Result<(), AttackError> {
        info!("Stopping attack {}", attack_id);
        
        // Remove from active attacks
        {
            let mut active_attacks = self.active_attacks.write().await;
            active_attacks.remove(&attack_id);
        }
        
        // TODO: Implement graceful shutdown of running requests
        
        Ok(())
    }
    
    /// Get list of active attacks
    pub async fn get_active_attacks(&self) -> Vec<AttackContext> {
        let active_attacks = self.active_attacks.read().await;
        active_attacks.values().cloned().collect()
    }
}

/// Default payload distributor implementation
pub struct DefaultPayloadDistributor;

impl PayloadDistributor for DefaultPayloadDistributor {
    fn distribute_payloads(
        &self,
        payloads: Vec<String>,
        agents: &[String],
        strategy: &DistributionStrategy,
    ) -> Result<HashMap<String, Vec<String>>, AttackError> {
        if agents.is_empty() {
            return Err(AttackError::InvalidAttackConfig {
                reason: "No agents available for payload distribution".to_string(),
            });
        }
        
        let mut distribution = HashMap::new();
        
        match strategy {
            DistributionStrategy::RoundRobin => {
                for (i, payload) in payloads.into_iter().enumerate() {
                    let agent = &agents[i % agents.len()];
                    distribution.entry(agent.clone())
                        .or_insert_with(Vec::new)
                        .push(payload);
                }
            }
            DistributionStrategy::Batch { batch_size } => {
                let batch_size = (*batch_size).max(1);
                let mut agent_index = 0;
                
                for chunk in payloads.chunks(batch_size) {
                    let agent = &agents[agent_index % agents.len()];
                    distribution.entry(agent.clone())
                        .or_insert_with(Vec::new)
                        .extend_from_slice(chunk);
                    agent_index += 1;
                }
            }
            DistributionStrategy::LoadBalanced => {
                // For now, use round-robin as a simple load balancing strategy
                // TODO: Implement actual load-based distribution
                return self.distribute_payloads(
                    payloads,
                    agents,
                    &DistributionStrategy::RoundRobin,
                );
            }
        }
        
        Ok(distribution)
    }
    
    fn calculate_batch_size(&self, payload_count: usize, agent_count: usize) -> usize {
        if agent_count == 0 {
            return payload_count;
        }
        
        (payload_count + agent_count - 1) / agent_count // Ceiling division
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_payload_distribution_round_robin() {
        let distributor = DefaultPayloadDistributor;
        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let agents = vec!["agent1".to_string(), "agent2".to_string()];
        
        let result = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::RoundRobin,
        ).unwrap();
        
        assert_eq!(result.get("agent1").unwrap(), &vec!["p1", "p3"]);
        assert_eq!(result.get("agent2").unwrap(), &vec!["p2"]);
    }
    
    #[test]
    fn test_payload_distribution_batch() {
        let distributor = DefaultPayloadDistributor;
        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string()];
        let agents = vec!["agent1".to_string(), "agent2".to_string()];
        
        let result = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::Batch { batch_size: 2 },
        ).unwrap();
        
        assert_eq!(result.get("agent1").unwrap(), &vec!["p1", "p2"]);
        assert_eq!(result.get("agent2").unwrap(), &vec!["p3", "p4"]);
    }
    
    #[test]
    fn test_batch_size_calculation() {
        let distributor = DefaultPayloadDistributor;
        
        assert_eq!(distributor.calculate_batch_size(10, 3), 4);
        assert_eq!(distributor.calculate_batch_size(9, 3), 3);
        assert_eq!(distributor.calculate_batch_size(5, 2), 3);
        assert_eq!(distributor.calculate_batch_size(0, 5), 0);
    }
}