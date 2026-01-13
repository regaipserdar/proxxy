//! Memory management for response body capture operations
//! 
//! This module provides memory tracking and enforcement for concurrent body captures
//! to prevent excessive memory usage and implement backpressure mechanisms.

use crate::error::BodyCaptureError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, warn, info};

/// Global memory manager for tracking and limiting memory usage across all body captures
#[derive(Debug, Clone)]
pub struct MemoryManager {
    /// Current memory usage in bytes (atomic for thread-safe access)
    current_usage: Arc<AtomicUsize>,
    /// Maximum allowed memory usage in bytes
    memory_limit: usize,
    /// Semaphore for controlling concurrent operations (backpressure)
    semaphore: Arc<Semaphore>,
    /// Maximum number of concurrent body captures allowed
    max_concurrent_captures: usize,
}

impl MemoryManager {
    /// Create a new MemoryManager with the specified limits
    pub fn new(memory_limit: usize, max_concurrent_captures: usize) -> Self {
        info!(
            "Creating MemoryManager with memory_limit={} bytes, max_concurrent={}",
            memory_limit, max_concurrent_captures
        );
        
        Self {
            current_usage: Arc::new(AtomicUsize::new(0)),
            memory_limit,
            semaphore: Arc::new(Semaphore::new(max_concurrent_captures)),
            max_concurrent_captures,
        }
    }

    /// Get current memory usage in bytes
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get memory limit in bytes
    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }

    /// Get available memory in bytes
    pub fn available_memory(&self) -> usize {
        self.memory_limit.saturating_sub(self.current_usage())
    }

    /// Get number of available concurrent capture slots
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Check if we can allocate the requested amount of memory
    pub fn can_allocate(&self, size: usize) -> bool {
        let current = self.current_usage();
        let would_exceed = current.saturating_add(size) > self.memory_limit;
        
        debug!(
            "Memory allocation check: current={}, requested={}, limit={}, would_exceed={}",
            current, size, self.memory_limit, would_exceed
        );
        
        !would_exceed
    }

    /// Attempt to acquire a permit for body capture (implements backpressure)
    /// Returns None if no permits are available (non-blocking)
    pub fn try_acquire_permit(&self) -> Option<MemoryPermit> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                debug!("Acquired memory permit, {} permits remaining", self.semaphore.available_permits());
                Some(MemoryPermit {
                    _permit: permit,
                    memory_manager: self.clone(),
                })
            }
            Err(_) => {
                debug!("No memory permits available for body capture");
                None
            }
        }
    }

    /// Acquire a permit for body capture (blocking, with backpressure)
    /// This will wait until a permit becomes available
    pub async fn acquire_permit(&self) -> Result<MemoryPermit, BodyCaptureError> {
        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                debug!("Acquired memory permit after waiting, {} permits remaining", self.semaphore.available_permits());
                Ok(MemoryPermit {
                    _permit: permit,
                    memory_manager: self.clone(),
                })
            }
            Err(_) => {
                warn!("Failed to acquire memory permit - semaphore closed");
                Err(BodyCaptureError::MemoryAllocationError)
            }
        }
    }

    /// Allocate memory for body capture
    /// Returns an allocation tracker that automatically frees memory when dropped
    pub fn allocate(&self, size: usize) -> Result<MemoryAllocation, BodyCaptureError> {
        if !self.can_allocate(size) {
            let current = self.current_usage();
            warn!(
                "Memory allocation failed: requested={}, current={}, limit={}, available={}",
                size, current, self.memory_limit, self.available_memory()
            );
            return Err(BodyCaptureError::MemoryAllocationError);
        }

        // Atomically add to current usage
        let new_usage = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;
        
        debug!(
            "Allocated {} bytes, total usage now {} bytes ({:.1}% of limit)",
            size, new_usage, (new_usage as f64 / self.memory_limit as f64) * 100.0
        );

        Ok(MemoryAllocation {
            size,
            memory_manager: self.clone(),
        })
    }

    /// Internal method to deallocate memory (called by MemoryAllocation::drop)
    fn deallocate(&self, size: usize) {
        let new_usage = self.current_usage.fetch_sub(size, Ordering::Relaxed).saturating_sub(size);
        
        debug!(
            "Deallocated {} bytes, total usage now {} bytes ({:.1}% of limit)",
            size, new_usage, (new_usage as f64 / self.memory_limit as f64) * 100.0
        );
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        let current = self.current_usage();
        let available = self.available_memory();
        let usage_percent = (current as f64 / self.memory_limit as f64) * 100.0;
        
        MemoryStats {
            current_usage: current,
            memory_limit: self.memory_limit,
            available_memory: available,
            usage_percent,
            available_permits: self.available_permits(),
            max_concurrent_captures: self.max_concurrent_captures,
        }
    }
}

/// RAII guard for memory allocation that automatically frees memory when dropped
#[derive(Debug)]
pub struct MemoryAllocation {
    size: usize,
    memory_manager: MemoryManager,
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        self.memory_manager.deallocate(self.size);
    }
}

impl MemoryAllocation {
    /// Get the size of this allocation
    pub fn size(&self) -> usize {
        self.size
    }
}

/// RAII guard for concurrent operation permits that automatically releases permit when dropped
#[derive(Debug)]
pub struct MemoryPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    memory_manager: MemoryManager,
}

impl MemoryPermit {
    /// Allocate memory within this permit
    pub fn allocate(&self, size: usize) -> Result<MemoryAllocation, BodyCaptureError> {
        self.memory_manager.allocate(size)
    }

    /// Get the memory manager associated with this permit
    pub fn memory_manager(&self) -> &MemoryManager {
        &self.memory_manager
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage: usize,
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// Available memory in bytes
    pub available_memory: usize,
    /// Memory usage as percentage
    pub usage_percent: f64,
    /// Number of available concurrent capture permits
    pub available_permits: usize,
    /// Maximum number of concurrent captures allowed
    pub max_concurrent_captures: usize,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory: {}/{} bytes ({:.1}%), Permits: {}/{}",
            self.current_usage,
            self.memory_limit,
            self.usage_percent,
            self.available_permits,
            self.max_concurrent_captures
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_allocation_and_deallocation() {
        let manager = MemoryManager::new(1000, 5);
        
        // Test successful allocation
        let allocation = manager.allocate(500).unwrap();
        assert_eq!(manager.current_usage(), 500);
        assert_eq!(allocation.size(), 500);
        
        // Test allocation within limits
        let allocation2 = manager.allocate(400).unwrap();
        assert_eq!(manager.current_usage(), 900);
        
        // Test allocation that would exceed limit
        let result = manager.allocate(200);
        assert!(result.is_err());
        assert_eq!(manager.current_usage(), 900);
        
        // Test deallocation when dropped
        drop(allocation);
        assert_eq!(manager.current_usage(), 400);
        
        drop(allocation2);
        assert_eq!(manager.current_usage(), 0);
    }

    #[tokio::test]
    async fn test_permit_system() {
        let manager = MemoryManager::new(1000, 2);
        
        // Should be able to acquire permits up to limit
        let permit1 = manager.try_acquire_permit().unwrap();
        let permit2 = manager.try_acquire_permit().unwrap();
        
        // Should not be able to acquire more permits
        let permit3 = manager.try_acquire_permit();
        assert!(permit3.is_none());
        
        // Should be able to acquire after dropping
        drop(permit1);
        let permit4 = manager.try_acquire_permit().unwrap();
        
        drop(permit2);
        drop(permit4);
    }

    #[tokio::test]
    async fn test_memory_stats() {
        let manager = MemoryManager::new(1000, 3);
        let _allocation = manager.allocate(300).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.current_usage, 300);
        assert_eq!(stats.memory_limit, 1000);
        assert_eq!(stats.available_memory, 700);
        assert_eq!(stats.usage_percent, 30.0);
        assert_eq!(stats.available_permits, 3);
        assert_eq!(stats.max_concurrent_captures, 3);
    }

    #[tokio::test]
    async fn test_concurrent_allocation_with_permits() {
        let manager = MemoryManager::new(1000, 2);
        
        let permit = manager.try_acquire_permit().unwrap();
        let allocation = permit.allocate(500).unwrap();
        
        assert_eq!(manager.current_usage(), 500);
        assert_eq!(allocation.size(), 500);
        
        drop(allocation);
        assert_eq!(manager.current_usage(), 0);
    }
}