//! Payload distribution algorithms for intruder attacks
//! 
//! This module implements various strategies for distributing payloads across
//! multiple agents, including load balancing and failure recovery mechanisms.

use attack_engine::{
    AttackError, AttackResult, DistributionStrategy, AgentInfo, AgentStatus
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Payload distribution assignment for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadAssignment {
    pub agent_id: String,
    pub payloads: Vec<String>,
    pub start_index: usize,
    pub end_index: usize,
    pub priority: u8, // 1-10, higher is more important
}

/// Distribution statistics and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub total_payloads: usize,
    pub total_agents: usize,
    pub assignments: Vec<PayloadAssignment>,
    pub load_balance_factor: f64, // 0.0-1.0, higher is more balanced
    pub estimated_completion_time: Option<std::time::Duration>,
}

/// Agent load information for load balancing
#[derive(Debug, Clone)]
pub struct AgentLoad {
    pub agent_id: String,
    pub current_load: f64, // 0.0-1.0
    pub response_time_ms: u64,
    pub active_requests: u32,
    pub max_concurrent: u32,
    pub reliability_score: f64, // 0.0-1.0 based on historical performance
}

/// Advanced payload distributor with load balancing and failure recovery
pub struct IntruderPayloadDistributor {
    agent_loads: Arc<RwLock<HashMap<String, AgentLoad>>>,
    failure_history: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>>>,
}

impl IntruderPayloadDistributor {
    /// Create a new payload distributor
    pub fn new() -> Self {
        Self {
            agent_loads: Arc::new(RwLock::new(HashMap::new())),
            failure_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update agent load information
    pub async fn update_agent_load(&self, agent_id: &str, load: AgentLoad) {
        let mut loads = self.agent_loads.write().await;
        loads.insert(agent_id.to_string(), load);
    }

    /// Record agent failure for reliability tracking
    pub async fn record_agent_failure(&self, agent_id: &str) {
        let mut failures = self.failure_history.write().await;
        let now = chrono::Utc::now();
        
        failures.entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(now);
        
        // Keep only failures from the last hour
        let cutoff = now - chrono::Duration::hours(1);
        if let Some(agent_failures) = failures.get_mut(agent_id) {
            agent_failures.retain(|&failure_time| failure_time > cutoff);
        }
    }

    /// Calculate agent reliability score based on recent failures
    async fn calculate_reliability_score(&self, agent_id: &str) -> f64 {
        let failures = self.failure_history.read().await;
        let agent_failures = failures.get(agent_id).map(|f| f.len()).unwrap_or(0);
        
        // Score decreases with more failures (max 10 failures = 0.0 score)
        (10.0 - agent_failures.min(10) as f64) / 10.0
    }

    /// Distribute payloads using the specified strategy
    pub async fn distribute_payloads(
        &self,
        payloads: Vec<String>,
        available_agents: &[AgentInfo],
        strategy: &DistributionStrategy,
    ) -> AttackResult<DistributionStats> {
        if payloads.is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "No payloads to distribute".to_string(),
            });
        }

        if available_agents.is_empty() {
            return Err(AttackError::AgentUnavailable {
                agent_id: "No agents available".to_string(),
            });
        }

        // Filter to only online agents
        let online_agents: Vec<&AgentInfo> = available_agents
            .iter()
            .filter(|agent| agent.status == AgentStatus::Online)
            .collect();

        if online_agents.is_empty() {
            return Err(AttackError::AgentUnavailable {
                agent_id: "No online agents available".to_string(),
            });
        }

        let payload_count = payloads.len();
        let assignments = match strategy {
            DistributionStrategy::RoundRobin => {
                self.distribute_round_robin(payloads, &online_agents).await?
            }
            DistributionStrategy::Batch { batch_size } => {
                self.distribute_batch(payloads, &online_agents, *batch_size).await?
            }
            DistributionStrategy::LoadBalanced => {
                self.distribute_load_balanced(payloads, &online_agents).await?
            }
        };

        let load_balance_factor = self.calculate_load_balance_factor(&assignments);
        let estimated_completion_time = self.estimate_completion_time(&assignments, &online_agents).await;

        Ok(DistributionStats {
            total_payloads: payload_count,
            total_agents: online_agents.len(),
            assignments,
            load_balance_factor,
            estimated_completion_time,
        })
    }

    /// Round-robin distribution strategy
    async fn distribute_round_robin(
        &self,
        payloads: Vec<String>,
        agents: &[&AgentInfo],
    ) -> AttackResult<Vec<PayloadAssignment>> {
        let mut assignments: HashMap<String, Vec<String>> = HashMap::new();
        let mut agent_indices: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, payload) in payloads.into_iter().enumerate() {
            let agent = &agents[i % agents.len()];
            assignments.entry(agent.id.clone())
                .or_insert_with(Vec::new)
                .push(payload);
            agent_indices.entry(agent.id.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }

        let mut result = Vec::new();
        for (agent_id, agent_payloads) in assignments {
            let indices = agent_indices.get(&agent_id).unwrap();
            result.push(PayloadAssignment {
                agent_id,
                payloads: agent_payloads,
                start_index: *indices.first().unwrap(),
                end_index: *indices.last().unwrap(),
                priority: 5, // Default priority
            });
        }

        Ok(result)
    }

    /// Batch distribution strategy
    async fn distribute_batch(
        &self,
        payloads: Vec<String>,
        agents: &[&AgentInfo],
        batch_size: usize,
    ) -> AttackResult<Vec<PayloadAssignment>> {
        let batch_size = batch_size.max(1);
        let mut assignments = Vec::new();
        let mut agent_index = 0;
        let mut payload_index = 0;

        for chunk in payloads.chunks(batch_size) {
            let agent = &agents[agent_index % agents.len()];
            
            assignments.push(PayloadAssignment {
                agent_id: agent.id.clone(),
                payloads: chunk.to_vec(),
                start_index: payload_index,
                end_index: payload_index + chunk.len() - 1,
                priority: 5, // Default priority
            });

            agent_index += 1;
            payload_index += chunk.len();
        }

        Ok(assignments)
    }

    /// Load-balanced distribution strategy
    async fn distribute_load_balanced(
        &self,
        payloads: Vec<String>,
        agents: &[&AgentInfo],
    ) -> AttackResult<Vec<PayloadAssignment>> {
        // Calculate agent weights based on load, response time, and reliability
        let mut agent_weights = Vec::new();
        let loads = self.agent_loads.read().await;

        for agent in agents {
            let load_info = loads.get(&agent.id);
            let reliability = self.calculate_reliability_score(&agent.id).await;
            
            let weight = match load_info {
                Some(load) => {
                    // Weight based on available capacity, response time, and reliability
                    let capacity_factor = 1.0 - load.current_load;
                    let response_factor = 1.0 / (1.0 + load.response_time_ms as f64 / 1000.0);
                    capacity_factor * response_factor * reliability
                }
                None => {
                    // Default weight for agents without load info
                    reliability * 0.5
                }
            };

            agent_weights.push((agent.id.clone(), weight.max(0.1))); // Minimum weight
        }

        // Normalize weights
        let total_weight: f64 = agent_weights.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return Err(AttackError::AgentUnavailable {
                agent_id: "All agents have zero weight".to_string(),
            });
        }

        // Distribute payloads proportionally to weights
        let mut assignments = Vec::new();
        let mut payload_index = 0;

        for (agent_id, weight) in agent_weights {
            let proportion = weight / total_weight;
            let payload_count = (payloads.len() as f64 * proportion).round() as usize;
            
            if payload_count > 0 && payload_index < payloads.len() {
                let end_index = (payload_index + payload_count).min(payloads.len());
                let agent_payloads = payloads[payload_index..end_index].to_vec();
                
                if !agent_payloads.is_empty() {
                    assignments.push(PayloadAssignment {
                        agent_id,
                        payloads: agent_payloads,
                        start_index: payload_index,
                        end_index: end_index - 1,
                        priority: (weight * 10.0) as u8, // Convert weight to priority
                    });
                    
                    payload_index = end_index;
                }
            }
        }

        // Assign any remaining payloads to the highest-weight agent
        if payload_index < payloads.len() {
            let remaining_payloads = payloads[payload_index..].to_vec();
            if let Some(assignment) = assignments.first_mut() {
                assignment.payloads.extend(remaining_payloads);
                assignment.end_index = payloads.len() - 1;
            }
        }

        Ok(assignments)
    }

    /// Redistribute payloads when an agent fails
    pub async fn redistribute_on_failure(
        &self,
        failed_agent_id: &str,
        original_distribution: &DistributionStats,
        available_agents: &[AgentInfo],
    ) -> AttackResult<DistributionStats> {
        info!("Redistributing payloads due to agent failure: {}", failed_agent_id);
        
        // Record the failure
        self.record_agent_failure(failed_agent_id).await;

        // Find the failed assignment
        let failed_assignment = original_distribution.assignments
            .iter()
            .find(|a| a.agent_id == failed_agent_id);

        let failed_payloads = match failed_assignment {
            Some(assignment) => assignment.payloads.clone(),
            None => {
                warn!("No assignment found for failed agent: {}", failed_agent_id);
                return Ok(original_distribution.clone());
            }
        };

        // Get remaining online agents (excluding the failed one)
        let remaining_agents: Vec<&AgentInfo> = available_agents
            .iter()
            .filter(|agent| agent.status == AgentStatus::Online && agent.id != failed_agent_id)
            .collect();

        if remaining_agents.is_empty() {
            return Err(AttackError::AgentUnavailable {
                agent_id: "No remaining agents available for redistribution".to_string(),
            });
        }

        // Redistribute failed payloads using load-balanced strategy
        let redistribution = self.distribute_load_balanced(
            failed_payloads,
            &remaining_agents,
        ).await?;

        // Merge with existing assignments (excluding the failed one)
        let mut new_assignments: Vec<PayloadAssignment> = original_distribution.assignments
            .iter()
            .filter(|a| a.agent_id != failed_agent_id)
            .cloned()
            .collect();

        // Add redistributed assignments
        for new_assignment in redistribution {
            // Check if we already have an assignment for this agent
            if let Some(existing) = new_assignments.iter_mut().find(|a| a.agent_id == new_assignment.agent_id) {
                // Merge payloads
                existing.payloads.extend(new_assignment.payloads);
                existing.end_index = existing.start_index + existing.payloads.len() - 1;
            } else {
                new_assignments.push(new_assignment);
            }
        }

        let load_balance_factor = self.calculate_load_balance_factor(&new_assignments);

        Ok(DistributionStats {
            total_payloads: original_distribution.total_payloads,
            total_agents: remaining_agents.len(),
            assignments: new_assignments,
            load_balance_factor,
            estimated_completion_time: None, // Recalculate if needed
        })
    }

    /// Calculate load balance factor (0.0 = unbalanced, 1.0 = perfectly balanced)
    fn calculate_load_balance_factor(&self, assignments: &[PayloadAssignment]) -> f64 {
        if assignments.is_empty() {
            return 1.0;
        }

        let payload_counts: Vec<usize> = assignments.iter().map(|a| a.payloads.len()).collect();
        let mean = payload_counts.iter().sum::<usize>() as f64 / payload_counts.len() as f64;
        
        if mean == 0.0 {
            return 1.0;
        }

        let variance = payload_counts.iter()
            .map(|&count| (count as f64 - mean).powi(2))
            .sum::<f64>() / payload_counts.len() as f64;
        
        let coefficient_of_variation = variance.sqrt() / mean;
        
        // Convert to balance factor (lower CV = higher balance)
        (1.0 / (1.0 + coefficient_of_variation)).max(0.0).min(1.0)
    }

    /// Estimate completion time based on agent performance
    async fn estimate_completion_time(
        &self,
        assignments: &[PayloadAssignment],
        agents: &[&AgentInfo],
    ) -> Option<std::time::Duration> {
        let loads = self.agent_loads.read().await;
        let mut max_time_ms = 0u64;

        for assignment in assignments {
            // Find agent info
            let agent = agents.iter().find(|a| a.id == assignment.agent_id)?;
            
            // Get load info or use defaults
            let (response_time, concurrent_requests) = match loads.get(&assignment.agent_id) {
                Some(load) => (load.response_time_ms, load.max_concurrent),
                None => (agent.response_time_ms.unwrap_or(1000), 10),
            };

            // Estimate time for this agent
            let payloads_per_batch = concurrent_requests as usize;
            let batches = (assignment.payloads.len() + payloads_per_batch - 1) / payloads_per_batch;
            let agent_time_ms = batches as u64 * response_time;
            
            max_time_ms = max_time_ms.max(agent_time_ms);
        }

        if max_time_ms > 0 {
            Some(std::time::Duration::from_millis(max_time_ms))
        } else {
            None
        }
    }

    /// Get current agent load statistics
    pub async fn get_agent_loads(&self) -> HashMap<String, AgentLoad> {
        self.agent_loads.read().await.clone()
    }

    /// Get agent failure history
    pub async fn get_failure_history(&self) -> HashMap<String, Vec<chrono::DateTime<chrono::Utc>>> {
        self.failure_history.read().await.clone()
    }
}

impl Default for IntruderPayloadDistributor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agents() -> Vec<AgentInfo> {
        vec![
            AgentInfo {
                id: "agent1".to_string(),
                hostname: "host1".to_string(),
                status: AgentStatus::Online,
                load: 0.3,
                response_time_ms: Some(100),
            },
            AgentInfo {
                id: "agent2".to_string(),
                hostname: "host2".to_string(),
                status: AgentStatus::Online,
                load: 0.7,
                response_time_ms: Some(200),
            },
            AgentInfo {
                id: "agent3".to_string(),
                hostname: "host3".to_string(),
                status: AgentStatus::Online,
                load: 0.1,
                response_time_ms: Some(50),
            },
        ]
    }

    #[tokio::test]
    async fn test_round_robin_distribution() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = create_test_agents();
        let agent_refs: Vec<&AgentInfo> = agents.iter().collect();
        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string()];

        let result = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::RoundRobin,
        ).await.unwrap();

        assert_eq!(result.total_payloads, 4);
        assert_eq!(result.total_agents, 3);
        assert_eq!(result.assignments.len(), 3);

        // Check that all agents got payloads
        let agent_ids: Vec<String> = result.assignments.iter().map(|a| a.agent_id.clone()).collect();
        assert!(agent_ids.contains(&"agent1".to_string()));
        assert!(agent_ids.contains(&"agent2".to_string()));
        assert!(agent_ids.contains(&"agent3".to_string()));
    }

    #[tokio::test]
    async fn test_batch_distribution() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = create_test_agents();
        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string()];

        let result = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::Batch { batch_size: 2 },
        ).await.unwrap();

        assert_eq!(result.total_payloads, 5);
        
        // Should have 3 batches: [p1,p2], [p3,p4], [p5]
        let total_assigned: usize = result.assignments.iter().map(|a| a.payloads.len()).sum();
        assert_eq!(total_assigned, 5);
    }

    #[tokio::test]
    async fn test_load_balanced_distribution() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = create_test_agents();
        
        // Set up load information
        distributor.update_agent_load("agent1", AgentLoad {
            agent_id: "agent1".to_string(),
            current_load: 0.3,
            response_time_ms: 100,
            active_requests: 3,
            max_concurrent: 10,
            reliability_score: 0.9,
        }).await;

        distributor.update_agent_load("agent2", AgentLoad {
            agent_id: "agent2".to_string(),
            current_load: 0.8,
            response_time_ms: 300,
            active_requests: 8,
            max_concurrent: 10,
            reliability_score: 0.7,
        }).await;

        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string()];

        let result = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::LoadBalanced,
        ).await.unwrap();

        assert_eq!(result.total_payloads, 4);
        
        // Agent1 should get more payloads than agent2 due to lower load
        let agent1_payloads = result.assignments.iter()
            .find(|a| a.agent_id == "agent1")
            .map(|a| a.payloads.len())
            .unwrap_or(0);
        
        let agent2_payloads = result.assignments.iter()
            .find(|a| a.agent_id == "agent2")
            .map(|a| a.payloads.len())
            .unwrap_or(0);

        // Agent1 should have at least as many payloads as agent2
        assert!(agent1_payloads >= agent2_payloads);
    }

    #[tokio::test]
    async fn test_failure_redistribution() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = create_test_agents();
        let payloads = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string()];

        // Initial distribution
        let original = distributor.distribute_payloads(
            payloads,
            &agents,
            &DistributionStrategy::RoundRobin,
        ).await.unwrap();

        // Simulate agent1 failure
        let redistributed = distributor.redistribute_on_failure(
            "agent1",
            &original,
            &agents,
        ).await.unwrap();

        // Should have fewer agents now
        assert_eq!(redistributed.total_agents, 2);
        
        // Agent1 should not be in the new distribution
        let has_agent1 = redistributed.assignments.iter().any(|a| a.agent_id == "agent1");
        assert!(!has_agent1);
        
        // Total payloads should remain the same
        let total_redistributed: usize = redistributed.assignments.iter().map(|a| a.payloads.len()).sum();
        assert_eq!(total_redistributed, 4);
    }

    #[tokio::test]
    async fn test_load_balance_factor_calculation() {
        let distributor = IntruderPayloadDistributor::new();
        
        // Perfectly balanced
        let balanced_assignments = vec![
            PayloadAssignment {
                agent_id: "agent1".to_string(),
                payloads: vec!["p1".to_string(), "p2".to_string()],
                start_index: 0,
                end_index: 1,
                priority: 5,
            },
            PayloadAssignment {
                agent_id: "agent2".to_string(),
                payloads: vec!["p3".to_string(), "p4".to_string()],
                start_index: 2,
                end_index: 3,
                priority: 5,
            },
        ];
        
        let balance_factor = distributor.calculate_load_balance_factor(&balanced_assignments);
        assert!(balance_factor > 0.9); // Should be close to 1.0
        
        // Unbalanced
        let unbalanced_assignments = vec![
            PayloadAssignment {
                agent_id: "agent1".to_string(),
                payloads: vec!["p1".to_string()],
                start_index: 0,
                end_index: 0,
                priority: 5,
            },
            PayloadAssignment {
                agent_id: "agent2".to_string(),
                payloads: vec!["p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string()],
                start_index: 1,
                end_index: 4,
                priority: 5,
            },
        ];
        
        let balance_factor = distributor.calculate_load_balance_factor(&unbalanced_assignments);
        assert!(balance_factor < 0.8); // Should be lower
    }
}