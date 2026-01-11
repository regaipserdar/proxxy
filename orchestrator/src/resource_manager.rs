//! Global Resource Manager for Proxxy
//! 
//! Coordinates resource allocation across all modules to prevent conflicts
//! and ensure system stability under load.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;
use tracing::{info, warn, error};

/// Global resource limits for the entire Proxxy system
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum number of concurrent browser instances
    pub max_browsers: usize,
    
    /// Maximum number of concurrent HTTP requests per agent
    pub max_requests_per_agent: usize,
    
    /// Maximum number of concurrent recording sessions
    pub max_recording_sessions: usize,
    
    /// Maximum number of concurrent nuclei scans
    pub max_nuclei_scans: usize,
    
    /// Maximum number of concurrent intruder attacks
    pub max_intruder_attacks: usize,
    
    /// Maximum total memory usage (MB)
    pub max_memory_mb: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_browsers: 10,
            max_requests_per_agent: 50,
            max_recording_sessions: 3,
            max_nuclei_scans: 5,
            max_intruder_attacks: 10,
            max_memory_mb: 4096, // 4GB
        }
    }
}

/// Resource allocation request
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub id: Uuid,
    pub module: ModuleType,
    pub resource_type: ResourceType,
    pub quantity: usize,
    pub priority: Priority,
}

/// Module types that can request resources
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleType {
    LSR,
    Repeater,
    Intruder,
    Nuclei,
    Agent,
}

/// Types of resources that can be allocated
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    Browser,
    HttpRequest,
    RecordingSession,
    NucleiScan,
    IntruderAttack,
    Memory(usize), // MB
}

/// Request priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Resource allocation result
#[derive(Debug)]
pub enum AllocationResult {
    Granted(ResourceAllocation),
    Denied(String),
    Queued(Uuid), // Request ID for queued request
}

/// Active resource allocation
#[derive(Debug)]
pub struct ResourceAllocation {
    pub id: Uuid,
    pub module: ModuleType,
    pub resource_type: ResourceType,
    pub quantity: usize,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
}

/// Global resource manager
pub struct ResourceManager {
    limits: ResourceLimits,
    allocations: Arc<Mutex<HashMap<Uuid, ResourceAllocation>>>,
    browser_semaphore: Arc<Semaphore>,
    recording_semaphore: Arc<Semaphore>,
    nuclei_semaphore: Arc<Semaphore>,
    intruder_semaphore: Arc<Semaphore>,
    agent_request_semaphores: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
    pending_requests: Arc<Mutex<Vec<ResourceRequest>>>,
}

impl ResourceManager {
    /// Create a new resource manager with default limits
    pub fn new() -> Self {
        Self::with_limits(ResourceLimits::default())
    }
    
    /// Create a new resource manager with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            browser_semaphore: Arc::new(Semaphore::new(limits.max_browsers)),
            recording_semaphore: Arc::new(Semaphore::new(limits.max_recording_sessions)),
            nuclei_semaphore: Arc::new(Semaphore::new(limits.max_nuclei_scans)),
            intruder_semaphore: Arc::new(Semaphore::new(limits.max_intruder_attacks)),
            limits,
            allocations: Arc::new(Mutex::new(HashMap::new())),
            agent_request_semaphores: Arc::new(Mutex::new(HashMap::new())),
            pending_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Request resource allocation
    pub async fn request_resource(&self, request: ResourceRequest) -> AllocationResult {
        info!("Resource request: {:?}", request);
        
        match self.try_allocate(&request).await {
            Ok(allocation) => {
                let mut allocations = self.allocations.lock().await;
                allocations.insert(allocation.id, allocation);
                AllocationResult::Granted(allocations.get(&request.id).unwrap().clone())
            }
            Err(reason) => {
                if request.priority >= Priority::High {
                    // Queue high priority requests
                    let mut pending = self.pending_requests.lock().await;
                    pending.push(request.clone());
                    pending.sort_by(|a, b| b.priority.cmp(&a.priority));
                    AllocationResult::Queued(request.id)
                } else {
                    AllocationResult::Denied(reason)
                }
            }
        }
    }
    
    /// Release allocated resources
    pub async fn release_resource(&self, allocation_id: Uuid) -> Result<(), String> {
        let mut allocations = self.allocations.lock().await;
        
        if let Some(allocation) = allocations.remove(&allocation_id) {
            info!("Releasing resource: {:?}", allocation);
            
            // Release semaphore permits
            match allocation.resource_type {
                ResourceType::Browser => {
                    self.browser_semaphore.add_permits(allocation.quantity);
                }
                ResourceType::RecordingSession => {
                    self.recording_semaphore.add_permits(allocation.quantity);
                }
                ResourceType::NucleiScan => {
                    self.nuclei_semaphore.add_permits(allocation.quantity);
                }
                ResourceType::IntruderAttack => {
                    self.intruder_semaphore.add_permits(allocation.quantity);
                }
                ResourceType::HttpRequest => {
                    // Handle per-agent request limits
                    // Implementation depends on agent identification
                }
                ResourceType::Memory(_) => {
                    // Memory tracking implementation
                }
            }
            
            // Process pending requests
            self.process_pending_requests().await;
            
            Ok(())
        } else {
            Err(format!("Allocation {} not found", allocation_id))
        }
    }
    
    /// Get current resource usage statistics
    pub async fn get_usage_stats(&self) -> ResourceUsageStats {
        let allocations = self.allocations.lock().await;
        
        let mut stats = ResourceUsageStats::default();
        
        for allocation in allocations.values() {
            match allocation.resource_type {
                ResourceType::Browser => stats.browsers_used += allocation.quantity,
                ResourceType::RecordingSession => stats.recording_sessions_used += allocation.quantity,
                ResourceType::NucleiScan => stats.nuclei_scans_used += allocation.quantity,
                ResourceType::IntruderAttack => stats.intruder_attacks_used += allocation.quantity,
                ResourceType::HttpRequest => stats.http_requests_used += allocation.quantity,
                ResourceType::Memory(mb) => stats.memory_used_mb += mb,
            }
        }
        
        stats.limits = self.limits.clone();
        stats
    }
    
    /// Try to allocate resources immediately
    async fn try_allocate(&self, request: &ResourceRequest) -> Result<ResourceAllocation, String> {
        let semaphore = match request.resource_type {
            ResourceType::Browser => &self.browser_semaphore,
            ResourceType::RecordingSession => &self.recording_semaphore,
            ResourceType::NucleiScan => &self.nuclei_semaphore,
            ResourceType::IntruderAttack => &self.intruder_semaphore,
            ResourceType::HttpRequest => {
                // Handle per-agent limits
                return Err("Per-agent HTTP request limits not implemented".to_string());
            }
            ResourceType::Memory(_) => {
                // Handle memory limits
                return Err("Memory limits not implemented".to_string());
            }
        };
        
        // Try to acquire permits
        match semaphore.try_acquire_many(request.quantity as u32) {
            Ok(_permit) => {
                // Don't drop the permit - it will be released when resource is freed
                std::mem::forget(_permit);
                
                Ok(ResourceAllocation {
                    id: request.id,
                    module: request.module.clone(),
                    resource_type: request.resource_type.clone(),
                    quantity: request.quantity,
                    allocated_at: chrono::Utc::now(),
                })
            }
            Err(_) => {
                Err(format!("Insufficient {} resources available", 
                    match request.resource_type {
                        ResourceType::Browser => "browser",
                        ResourceType::RecordingSession => "recording session",
                        ResourceType::NucleiScan => "nuclei scan",
                        ResourceType::IntruderAttack => "intruder attack",
                        ResourceType::HttpRequest => "HTTP request",
                        ResourceType::Memory(_) => "memory",
                    }
                ))
            }
        }
    }
    
    /// Process pending resource requests
    async fn process_pending_requests(&self) {
        let mut pending = self.pending_requests.lock().await;
        let mut processed = Vec::new();
        
        for (index, request) in pending.iter().enumerate() {
            if let Ok(allocation) = self.try_allocate(request).await {
                let mut allocations = self.allocations.lock().await;
                allocations.insert(allocation.id, allocation);
                processed.push(index);
                info!("Processed pending request: {:?}", request);
            }
        }
        
        // Remove processed requests (in reverse order to maintain indices)
        for &index in processed.iter().rev() {
            pending.remove(index);
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Default)]
pub struct ResourceUsageStats {
    pub browsers_used: usize,
    pub recording_sessions_used: usize,
    pub nuclei_scans_used: usize,
    pub intruder_attacks_used: usize,
    pub http_requests_used: usize,
    pub memory_used_mb: usize,
    pub limits: ResourceLimits,
}

impl ResourceUsageStats {
    /// Get usage percentage for browsers
    pub fn browser_usage_percent(&self) -> f64 {
        (self.browsers_used as f64 / self.limits.max_browsers as f64) * 100.0
    }
    
    /// Check if system is under high load
    pub fn is_high_load(&self) -> bool {
        self.browser_usage_percent() > 80.0 ||
        (self.recording_sessions_used as f64 / self.limits.max_recording_sessions as f64) > 80.0 ||
        (self.nuclei_scans_used as f64 / self.limits.max_nuclei_scans as f64) > 80.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_allocation() {
        let manager = ResourceManager::new();
        
        let request = ResourceRequest {
            id: Uuid::new_v4(),
            module: ModuleType::LSR,
            resource_type: ResourceType::Browser,
            quantity: 1,
            priority: Priority::Normal,
        };
        
        match manager.request_resource(request.clone()).await {
            AllocationResult::Granted(allocation) => {
                assert_eq!(allocation.module, ModuleType::LSR);
                assert_eq!(allocation.quantity, 1);
                
                // Release the resource
                manager.release_resource(allocation.id).await.unwrap();
            }
            _ => panic!("Resource allocation should have been granted"),
        }
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let limits = ResourceLimits {
            max_browsers: 1,
            ..Default::default()
        };
        let manager = ResourceManager::with_limits(limits);
        
        let request1 = ResourceRequest {
            id: Uuid::new_v4(),
            module: ModuleType::LSR,
            resource_type: ResourceType::Browser,
            quantity: 1,
            priority: Priority::Normal,
        };
        
        let request2 = ResourceRequest {
            id: Uuid::new_v4(),
            module: ModuleType::Nuclei,
            resource_type: ResourceType::Browser,
            quantity: 1,
            priority: Priority::Normal,
        };
        
        // First request should succeed
        match manager.request_resource(request1).await {
            AllocationResult::Granted(_) => {}
            _ => panic!("First request should have been granted"),
        }
        
        // Second request should be denied
        match manager.request_resource(request2).await {
            AllocationResult::Denied(_) => {}
            _ => panic!("Second request should have been denied"),
        }
    }
}