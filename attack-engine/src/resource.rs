//! Resource management integration for the attack engine

use crate::{AttackError, ModuleType, Priority};
use std::sync::Arc;
use uuid::Uuid;

/// Resource manager adapter for integrating with the global resource manager
pub struct ResourceManagerAdapter {
    // This will be integrated with orchestrator::resource_manager::ResourceManager
    // For now, we define the interface that will be implemented
}

impl ResourceManagerAdapter {
    /// Create a new resource manager adapter
    pub fn new() -> Self {
        Self {}
    }
    
    /// Request resources for attack execution
    pub async fn request_attack_resources(
        &self,
        module_type: ModuleType,
        agent_count: usize,
        concurrent_requests: u32,
        priority: Priority,
    ) -> Result<ResourceAllocation, AttackError> {
        // TODO: Integrate with orchestrator::resource_manager::ResourceManager
        // This will use the ResourceRequest and ResourceManager from orchestrator
        
        // For now, return a mock allocation
        Ok(ResourceAllocation {
            id: Uuid::new_v4(),
            module_type,
            agent_count,
            concurrent_requests,
            allocated_at: chrono::Utc::now(),
        })
    }
    
    /// Release allocated resources
    pub async fn release_resources(&self, allocation_id: Uuid) -> Result<(), AttackError> {
        // TODO: Integrate with orchestrator::resource_manager::ResourceManager
        tracing::info!("Releasing resource allocation: {}", allocation_id);
        Ok(())
    }
    
    /// Check if resources are available for the given requirements
    pub async fn check_resource_availability(
        &self,
        module_type: ModuleType,
        agent_count: usize,
        concurrent_requests: u32,
    ) -> Result<bool, AttackError> {
        // TODO: Integrate with orchestrator::resource_manager::ResourceManager
        // For now, always return true
        Ok(true)
    }
}

/// Resource allocation information
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub id: Uuid,
    pub module_type: ModuleType,
    pub agent_count: usize,
    pub concurrent_requests: u32,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
}

impl ResourceAllocation {
    /// Calculate total resource usage
    pub fn total_concurrent_requests(&self) -> u32 {
        self.concurrent_requests * self.agent_count as u32
    }
    
    /// Check if allocation is expired (older than 1 hour)
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.allocated_at);
        duration.num_hours() > 1
    }
}

/// Resource usage statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsageStats {
    pub active_allocations: usize,
    pub total_concurrent_requests: u32,
    pub repeater_allocations: usize,
    pub intruder_allocations: usize,
    pub average_allocation_age_minutes: f64,
}

impl Default for ResourceUsageStats {
    fn default() -> Self {
        Self {
            active_allocations: 0,
            total_concurrent_requests: 0,
            repeater_allocations: 0,
            intruder_allocations: 0,
            average_allocation_age_minutes: 0.0,
        }
    }
}

/// Resource monitoring service
pub struct ResourceMonitor {
    allocations: Arc<tokio::sync::RwLock<Vec<ResourceAllocation>>>,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Add a new allocation to monitor
    pub async fn add_allocation(&self, allocation: ResourceAllocation) {
        let mut allocations = self.allocations.write().await;
        allocations.push(allocation);
    }
    
    /// Remove an allocation from monitoring
    pub async fn remove_allocation(&self, allocation_id: Uuid) {
        let mut allocations = self.allocations.write().await;
        allocations.retain(|a| a.id != allocation_id);
    }
    
    /// Get current resource usage statistics
    pub async fn get_usage_stats(&self) -> ResourceUsageStats {
        let allocations = self.allocations.read().await;
        let now = chrono::Utc::now();
        
        let mut stats = ResourceUsageStats::default();
        stats.active_allocations = allocations.len();
        
        let mut total_age_minutes = 0.0;
        
        for allocation in allocations.iter() {
            stats.total_concurrent_requests += allocation.total_concurrent_requests();
            
            match allocation.module_type {
                ModuleType::Repeater => stats.repeater_allocations += 1,
                ModuleType::Intruder => stats.intruder_allocations += 1,
            }
            
            let age = now.signed_duration_since(allocation.allocated_at);
            total_age_minutes += age.num_minutes() as f64;
        }
        
        if !allocations.is_empty() {
            stats.average_allocation_age_minutes = total_age_minutes / allocations.len() as f64;
        }
        
        stats
    }
    
    /// Clean up expired allocations
    pub async fn cleanup_expired(&self) -> usize {
        let mut allocations = self.allocations.write().await;
        let initial_count = allocations.len();
        allocations.retain(|a| !a.is_expired());
        initial_count - allocations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_allocation_calculations() {
        let allocation = ResourceAllocation {
            id: Uuid::new_v4(),
            module_type: ModuleType::Intruder,
            agent_count: 3,
            concurrent_requests: 10,
            allocated_at: chrono::Utc::now(),
        };
        
        assert_eq!(allocation.total_concurrent_requests(), 30);
        assert!(!allocation.is_expired());
    }
    
    #[test]
    fn test_expired_allocation() {
        let allocation = ResourceAllocation {
            id: Uuid::new_v4(),
            module_type: ModuleType::Repeater,
            agent_count: 1,
            concurrent_requests: 5,
            allocated_at: chrono::Utc::now() - chrono::Duration::hours(2),
        };
        
        assert!(allocation.is_expired());
    }
    
    #[tokio::test]
    async fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        
        let allocation = ResourceAllocation {
            id: Uuid::new_v4(),
            module_type: ModuleType::Intruder,
            agent_count: 2,
            concurrent_requests: 15,
            allocated_at: chrono::Utc::now(),
        };
        
        monitor.add_allocation(allocation.clone()).await;
        
        let stats = monitor.get_usage_stats().await;
        assert_eq!(stats.active_allocations, 1);
        assert_eq!(stats.total_concurrent_requests, 30);
        assert_eq!(stats.intruder_allocations, 1);
        assert_eq!(stats.repeater_allocations, 0);
        
        monitor.remove_allocation(allocation.id).await;
        
        let stats = monitor.get_usage_stats().await;
        assert_eq!(stats.active_allocations, 0);
    }
}