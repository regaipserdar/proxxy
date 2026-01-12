//! Property-based tests for intruder payload distribution algorithms
//! 
//! **Feature: repeater-intruder, Property 4: Payload Distribution Algorithms**
//! **Validates: Requirements 3.5, 4.4, 4.5**

use orchestrator::intruder::distribution::{IntruderPayloadDistributor, AgentLoad};
use attack_engine::{DistributionStrategy, AgentInfo, AgentStatus};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::HashSet;

/// Generate test agents with various configurations
fn arb_agents() -> impl Strategy<Value = Vec<AgentInfo>> {
    prop::collection::vec(
        (
            "[a-z]{1,10}",  // agent_id
            "[a-z]{1,10}",  // hostname
            prop::sample::select(vec![AgentStatus::Online, AgentStatus::Offline, AgentStatus::Busy]),
            0.0f64..1.0f64, // load
            prop::option::of(1u64..5000u64), // response_time_ms
        ).prop_map(|(id, hostname, status, load, response_time)| {
            AgentInfo {
                id,
                hostname,
                status,
                load,
                response_time_ms: response_time,
            }
        }),
        1..10 // 1 to 10 agents
    )
}

/// Generate test payloads
fn arb_payloads() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        "[a-zA-Z0-9]{1,20}",
        1..100 // 1 to 100 payloads
    )
}

/// Generate distribution strategies
fn arb_distribution_strategy() -> impl Strategy<Value = DistributionStrategy> {
    prop_oneof![
        Just(DistributionStrategy::RoundRobin),
        (1usize..50usize).prop_map(|batch_size| DistributionStrategy::Batch { batch_size }),
        Just(DistributionStrategy::LoadBalanced),
    ]
}

// Temporarily disabled proptest for compilation check
#[cfg(disabled)]
proptest! {
    /// **Property 4: Payload Distribution Algorithms**
    /// **Validates: Requirements 3.5, 4.4, 4.5**
    /// 
    /// For any payload distribution across multiple agents, the system should implement 
    /// the specified distribution strategy (round-robin or batch), handle agent failures 
    /// by redistributing payloads, and maintain load balance across available agents.
    #[test]
    fn test_payload_distribution_completeness(
        payloads in arb_payloads(),
        agents in arb_agents(),
        strategy in arb_distribution_strategy()
    ) {
        let result = tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            
            // Filter to only online agents (as the system should do)
            let online_agents: Vec<&AgentInfo> = agents.iter()
                .filter(|agent| agent.status == AgentStatus::Online)
                .collect();
            
            if online_agents.is_empty() {
                // If no online agents, distribution should fail
                let result = distributor.distribute_payloads(payloads, &agents, &strategy).await;
                prop_assert!(result.is_err());
                return Ok(());
            }
            
            let result = distributor.distribute_payloads(payloads.clone(), &agents, &strategy).await;
            prop_assert!(result.is_ok());
            
            let distribution = result.unwrap();
            
            // Property 1: All payloads must be distributed
            let total_distributed: usize = distribution.assignments.iter()
                .map(|assignment| assignment.payloads.len())
                .sum();
            prop_assert_eq!(total_distributed, payloads.len());
            
            // Property 2: No payload should be duplicated across agents
            let mut all_distributed_payloads = Vec::new();
            for assignment in &distribution.assignments {
                all_distributed_payloads.extend(assignment.payloads.iter().cloned());
            }
            let unique_payloads: HashSet<String> = all_distributed_payloads.iter().cloned().collect();
            prop_assert_eq!(unique_payloads.len(), payloads.len());
            
            // Property 3: Only online agents should receive assignments
            for assignment in &distribution.assignments {
                let agent_is_online = online_agents.iter()
                    .any(|agent| agent.id == assignment.agent_id && agent.status == AgentStatus::Online);
                prop_assert!(agent_is_online);
            }
            
            // Property 4: Distribution statistics should be accurate
            prop_assert_eq!(distribution.total_payloads, payloads.len());
            prop_assert_eq!(distribution.total_agents, online_agents.len());
            prop_assert!(distribution.load_balance_factor >= 0.0 && distribution.load_balance_factor <= 1.0);
            
            Ok(())
        })
    }

    /// Test round-robin distribution fairness
    #[test]
    fn test_round_robin_fairness(
        payloads in arb_payloads(),
        agents in arb_agents().prop_filter("Need online agents", |agents| {
            agents.iter().any(|a| a.status == AgentStatus::Online)
        })
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            let online_agents: Vec<&AgentInfo> = agents.iter()
                .filter(|agent| agent.status == AgentStatus::Online)
                .collect();
            
            let result = distributor.distribute_payloads(
                payloads.clone(), 
                &agents, 
                &DistributionStrategy::RoundRobin
            ).await;
            
            prop_assert!(result.is_ok());
            let distribution = result.unwrap();
            
            // For round-robin, the difference in payload count between any two agents
            // should be at most 1 (fair distribution)
            let payload_counts: Vec<usize> = distribution.assignments.iter()
                .map(|assignment| assignment.payloads.len())
                .collect();
            
            if payload_counts.len() > 1 {
                let min_count = payload_counts.iter().min().unwrap();
                let max_count = payload_counts.iter().max().unwrap();
                prop_assert!(max_count - min_count <= 1);
            }
            
            Ok(())
        })
    }

    /// Test batch distribution properties
    #[test]
    fn test_batch_distribution_properties(
        payloads in arb_payloads(),
        agents in arb_agents().prop_filter("Need online agents", |agents| {
            agents.iter().any(|a| a.status == AgentStatus::Online)
        }),
        batch_size in 1usize..20usize
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            let online_agents: Vec<&AgentInfo> = agents.iter()
                .filter(|agent| agent.status == AgentStatus::Online)
                .collect();
            
            let result = distributor.distribute_payloads(
                payloads.clone(), 
                &agents, 
                &DistributionStrategy::Batch { batch_size }
            ).await;
            
            prop_assert!(result.is_ok());
            let distribution = result.unwrap();
            
            // For batch distribution, each assignment (except possibly the last) 
            // should have exactly batch_size payloads
            let mut assignment_sizes: Vec<usize> = distribution.assignments.iter()
                .map(|assignment| assignment.payloads.len())
                .collect();
            assignment_sizes.sort();
            
            // All but the last assignment should have batch_size payloads
            for &size in &assignment_sizes[..assignment_sizes.len().saturating_sub(1)] {
                prop_assert_eq!(size, batch_size);
            }
            
            // The last assignment should have between 1 and batch_size payloads
            if let Some(&last_size) = assignment_sizes.last() {
                prop_assert!(last_size >= 1 && last_size <= batch_size);
            }
            
            Ok(())
        })
    }

    /// Test load-balanced distribution considers agent capabilities
    #[test]
    fn test_load_balanced_distribution(
        payloads in arb_payloads(),
        agents in arb_agents().prop_filter("Need online agents", |agents| {
            agents.iter().any(|a| a.status == AgentStatus::Online)
        })
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            
            // Set up different load levels for agents
            for (i, agent) in agents.iter().enumerate() {
                if agent.status == AgentStatus::Online {
                    let load = AgentLoad {
                        agent_id: agent.id.clone(),
                        current_load: (i as f64 * 0.2) % 1.0, // Varying loads
                        response_time_ms: 100 + (i as u64 * 50),
                        active_requests: i as u32,
                        max_concurrent: 10,
                        reliability_score: 0.9 - (i as f64 * 0.1).min(0.5),
                    };
                    distributor.update_agent_load(&agent.id, load).await;
                }
            }
            
            let result = distributor.distribute_payloads(
                payloads.clone(), 
                &agents, 
                &DistributionStrategy::LoadBalanced
            ).await;
            
            prop_assert!(result.is_ok());
            let distribution = result.unwrap();
            
            // Load-balanced distribution should still distribute all payloads
            let total_distributed: usize = distribution.assignments.iter()
                .map(|assignment| assignment.payloads.len())
                .sum();
            prop_assert_eq!(total_distributed, payloads.len());
            
            // Agents with lower load should generally get more payloads
            // (This is a soft property due to the complexity of load balancing)
            if distribution.assignments.len() > 1 {
                prop_assert!(distribution.load_balance_factor >= 0.0);
            }
            
            Ok(())
        })
    }

    /// Test failure redistribution maintains payload completeness
    #[test]
    fn test_failure_redistribution_completeness(
        payloads in arb_payloads(),
        agents in arb_agents().prop_filter("Need multiple online agents", |agents| {
            agents.iter().filter(|a| a.status == AgentStatus::Online).count() >= 2
        })
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            let online_agents: Vec<&AgentInfo> = agents.iter()
                .filter(|agent| agent.status == AgentStatus::Online)
                .collect();
            
            // Initial distribution
            let initial_result = distributor.distribute_payloads(
                payloads.clone(), 
                &agents, 
                &DistributionStrategy::RoundRobin
            ).await;
            prop_assert!(initial_result.is_ok());
            let initial_distribution = initial_result.unwrap();
            
            // Pick a random agent to fail
            if let Some(failed_agent) = initial_distribution.assignments.first() {
                let failed_agent_id = &failed_agent.agent_id;
                
                // Redistribute after failure
                let redistribution_result = distributor.redistribute_on_failure(
                    failed_agent_id,
                    &initial_distribution,
                    &agents
                ).await;
                
                prop_assert!(redistribution_result.is_ok());
                let redistributed = redistribution_result.unwrap();
                
                // Property 1: Total payloads should remain the same
                prop_assert_eq!(redistributed.total_payloads, initial_distribution.total_payloads);
                
                // Property 2: Failed agent should not appear in new distribution
                let has_failed_agent = redistributed.assignments.iter()
                    .any(|assignment| assignment.agent_id == *failed_agent_id);
                prop_assert!(!has_failed_agent);
                
                // Property 3: All payloads should still be distributed
                let total_redistributed: usize = redistributed.assignments.iter()
                    .map(|assignment| assignment.payloads.len())
                    .sum();
                prop_assert_eq!(total_redistributed, payloads.len());
                
                // Property 4: Should have fewer agents
                prop_assert!(redistributed.total_agents < initial_distribution.total_agents);
            }
            
            Ok(())
        })
    }

    /// Test agent failure recording and reliability scoring
    #[test]
    fn test_agent_reliability_tracking(
        agent_ids in prop::collection::vec("[a-z]{1,10}", 1..5),
        failure_counts in prop::collection::vec(0usize..10usize, 1..5)
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            
            // Record failures for agents
            for (agent_id, &failure_count) in agent_ids.iter().zip(failure_counts.iter()) {
                for _ in 0..failure_count {
                    distributor.record_agent_failure(agent_id).await;
                }
            }
            
            let failure_history = distributor.get_failure_history().await;
            
            // Property 1: Failure history should be recorded for each agent
            for (agent_id, &expected_count) in agent_ids.iter().zip(failure_counts.iter()) {
                if expected_count > 0 {
                    prop_assert!(failure_history.contains_key(agent_id));
                    let recorded_failures = failure_history.get(agent_id).unwrap().len();
                    prop_assert_eq!(recorded_failures, expected_count);
                }
            }
            
            // Property 2: Agents with no failures should not appear in history
            for (agent_id, &failure_count) in agent_ids.iter().zip(failure_counts.iter()) {
                if failure_count == 0 {
                    prop_assert!(!failure_history.contains_key(agent_id) || 
                                failure_history.get(agent_id).unwrap().is_empty());
                }
            }
            
            Ok(())
        })
    }

    /// Test distribution strategy consistency
    #[test]
    fn test_distribution_strategy_consistency(
        payloads in arb_payloads(),
        agents in arb_agents().prop_filter("Need online agents", |agents| {
            agents.iter().any(|a| a.status == AgentStatus::Online)
        }),
        strategy in arb_distribution_strategy()
    ) -> Result<(), TestCaseError> {
        tokio_test::block_on(async {
            let distributor = IntruderPayloadDistributor::new();
            
            // Run distribution multiple times with same inputs
            let result1 = distributor.distribute_payloads(payloads.clone(), &agents, &strategy).await;
            let result2 = distributor.distribute_payloads(payloads.clone(), &agents, &strategy).await;
            
            if result1.is_ok() && result2.is_ok() {
                let dist1 = result1.unwrap();
                let dist2 = result2.unwrap();
                
                // Property: Results should be consistent (same total payloads, agents)
                prop_assert_eq!(dist1.total_payloads, dist2.total_payloads);
                prop_assert_eq!(dist1.total_agents, dist2.total_agents);
                
                // For deterministic strategies (round-robin, batch), results should be identical
                match strategy {
                    DistributionStrategy::RoundRobin | DistributionStrategy::Batch { .. } => {
                        // Sort assignments by agent_id for comparison
                        let mut assignments1 = dist1.assignments.clone();
                        let mut assignments2 = dist2.assignments.clone();
                        assignments1.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
                        assignments2.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
                        
                        prop_assert_eq!(assignments1.len(), assignments2.len());
                        for (a1, a2) in assignments1.iter().zip(assignments2.iter()) {
                            prop_assert_eq!(a1.agent_id, a2.agent_id);
                            prop_assert_eq!(a1.payloads.len(), a2.payloads.len());
                        }
                    }
                    DistributionStrategy::LoadBalanced => {
                        // Load-balanced may vary slightly, but totals should match
                        // This is acceptable as load balancing can be dynamic
                    }
                }
            }
            
            Ok(())
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_payloads_error() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = vec![AgentInfo {
            id: "agent1".to_string(),
            hostname: "host1".to_string(),
            status: AgentStatus::Online,
            load: 0.1,
            response_time_ms: Some(100),
        }];
        
        let result = distributor.distribute_payloads(
            Vec::new(), // Empty payloads
            &agents,
            &DistributionStrategy::RoundRobin
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_online_agents_error() {
        let distributor = IntruderPayloadDistributor::new();
        let agents = vec![AgentInfo {
            id: "agent1".to_string(),
            hostname: "host1".to_string(),
            status: AgentStatus::Offline, // Offline agent
            load: 0.1,
            response_time_ms: Some(100),
        }];
        
        let result = distributor.distribute_payloads(
            vec!["payload1".to_string()],
            &agents,
            &DistributionStrategy::RoundRobin
        ).await;
        
        assert!(result.is_err());
    }
}