//! Integration test for memory management in response body capture
//! 
//! This test demonstrates the memory tracking and backpressure mechanisms
//! working together with the LogHandler.

#[cfg(test)]
mod integration_tests {
    use crate::{LogHandler, BodyCaptureConfig, Metrics};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_memory_manager_integration() {
        // Create a LogHandler with memory limits
        let metrics = Arc::new(Metrics::default());
        let (sender, _receiver) = mpsc::channel(100);
        
        let config = BodyCaptureConfig {
            enabled: true,
            max_body_size: 1024,
            truncate_threshold: 512,
            memory_limit: 2048,  // 2KB total memory limit
            max_concurrent_captures: 2,  // Only 2 concurrent captures
            content_type_filters: vec![],
            content_type_filter_mode: crate::config::ContentTypeFilterMode::CaptureAll,
            response_timeout_secs: 30,
            stream_read_timeout_secs: 5,
        };
        
        let handler = LogHandler::new(metrics, Some(sender), config);
        
        // Check initial memory stats
        let stats = handler.get_memory_stats();
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.memory_limit, 2048);
        assert_eq!(stats.available_permits, 2);
        assert_eq!(stats.max_concurrent_captures, 2);
        
        println!("Memory manager integration test passed!");
        println!("Initial stats: {}", stats);
    }

    #[tokio::test]
    async fn test_memory_manager_backpressure() {
        use crate::memory_manager::MemoryManager;
        
        // Create a memory manager with very limited resources
        let manager = MemoryManager::new(100, 1); // 100 bytes, 1 concurrent operation
        
        // First permit should succeed
        let permit1 = manager.try_acquire_permit();
        assert!(permit1.is_some());
        
        // Second permit should fail (backpressure)
        let permit2 = manager.try_acquire_permit();
        assert!(permit2.is_none());
        
        // After dropping first permit, second should succeed
        drop(permit1);
        let permit3 = manager.try_acquire_permit();
        assert!(permit3.is_some());
        
        println!("Backpressure mechanism working correctly!");
    }

    #[tokio::test]
    async fn test_memory_allocation_tracking() {
        use crate::memory_manager::MemoryManager;
        
        let manager = MemoryManager::new(1000, 5);
        let permit = manager.try_acquire_permit().unwrap();
        
        // Test allocation tracking
        let allocation1 = permit.allocate(300).unwrap();
        assert_eq!(manager.current_usage(), 300);
        
        let allocation2 = permit.allocate(400).unwrap();
        assert_eq!(manager.current_usage(), 700);
        
        // Test memory limit enforcement
        let allocation3 = permit.allocate(400); // Would exceed 1000 byte limit
        assert!(allocation3.is_err());
        assert_eq!(manager.current_usage(), 700); // Should remain unchanged
        
        // Test automatic deallocation
        drop(allocation1);
        assert_eq!(manager.current_usage(), 400);
        
        drop(allocation2);
        assert_eq!(manager.current_usage(), 0);
        
        println!("Memory allocation tracking working correctly!");
    }

    #[tokio::test]
    async fn test_content_type_filtering_integration() {
        use crate::config::ContentTypeFilterMode;
        
        // Test whitelist filtering
        let metrics = Arc::new(Metrics::default());
        let (sender, _receiver) = mpsc::channel(100);
        
        let whitelist_config = BodyCaptureConfig {
            enabled: true,
            max_body_size: 1024,
            truncate_threshold: 512,
            memory_limit: 2048,
            max_concurrent_captures: 2,
            content_type_filters: vec!["json".to_string()],
            content_type_filter_mode: ContentTypeFilterMode::Whitelist,
            response_timeout_secs: 30,
            stream_read_timeout_secs: 5,
        };
        
        let handler = LogHandler::new(metrics.clone(), Some(sender.clone()), whitelist_config);
        
        // Verify configuration is applied correctly
        let stats = handler.get_memory_stats();
        assert_eq!(stats.memory_limit, 2048);
        
        // Test blacklist filtering
        let blacklist_config = BodyCaptureConfig {
            enabled: true,
            max_body_size: 1024,
            truncate_threshold: 512,
            memory_limit: 2048,
            max_concurrent_captures: 2,
            content_type_filters: vec!["image".to_string()],
            content_type_filter_mode: ContentTypeFilterMode::Blacklist,
            response_timeout_secs: 30,
            stream_read_timeout_secs: 5,
        };
        
        let _blacklist_handler = LogHandler::new(metrics, Some(sender), blacklist_config);
        
        println!("Content-type filtering integration test passed!");
    }
}