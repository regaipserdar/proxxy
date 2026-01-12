//! Core functionality validation test
//! 
//! This test validates that all core attack engine components work together
//! and that database operations and data persistence function correctly.

use orchestrator::{Database, repeater::RepeaterManager, intruder::IntruderManager};
use orchestrator::session_manager::AgentRegistry;
use attack_engine::{HttpRequestData, HttpHeaders, AttackError};
use std::sync::Arc;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a test database instance
async fn create_test_database() -> (Arc<Database>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db = Arc::new(
        Database::new(temp_dir.path().to_str().unwrap())
            .await
            .expect("Failed to create database")
    );
    
    // Create and load a test project
    db.create_project("test-project")
        .await
        .expect("Failed to create test project");
    
    db.load_project("test-project")
        .await
        .expect("Failed to load test project");
    
    (db, temp_dir)
}

/// Create a test HTTP request
fn create_test_request() -> HttpRequestData {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("User-Agent".to_string(), "Proxxy-Test/1.0".to_string());
    
    HttpRequestData {
        method: "POST".to_string(),
        url: "https://api.example.com/test".to_string(),
        headers: Some(HttpHeaders { headers }),
        body: b"{\"test\": \"data\"}".to_vec(),
        tls: None,
    }
}

/// Test that database operations work correctly
#[tokio::test]
async fn test_database_operations() {
    let (db, _temp_dir) = create_test_database().await;
    
    // Create a test agent first (required for foreign key constraint)
    db.upsert_agent("test-agent-1", "Test Agent", "localhost", "1.0.0")
        .await.expect("Failed to create test agent");
    
    // Test repeater database operations
    let request = create_test_request();
    let request_json = serde_json::to_string(&request).expect("Failed to serialize request");
    
    // Create a repeater tab
    let tab_id = db.create_repeater_tab(
        "Test Tab",
        &request_json,
        Some("test-agent-1"),
    ).await.expect("Failed to create repeater tab");
    
    // Verify tab was created
    let tabs = db.get_repeater_tabs().await.expect("Failed to get repeater tabs");
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].name, "Test Tab");
    assert_eq!(tabs[0].target_agent_id, Some("test-agent-1".to_string()));
    
    // Save an execution result
    let response_json = r#"{"status_code": 200, "headers": {"Content-Type": "application/json"}, "body": [123, 34, 115, 117, 99, 99, 101, 115, 115, 34, 58, 116, 114, 117, 101, 125], "tls": null}"#;
    let _execution_id = db.save_repeater_execution(
        &tab_id,
        &request_json,
        Some(response_json),
        "test-agent-1",
        Some(150),
        Some(200),
    ).await.expect("Failed to save execution");
    
    // Verify execution was saved
    let history = db.get_repeater_history(&tab_id, Some(10))
        .await.expect("Failed to get history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].agent_id, "test-agent-1");
    assert_eq!(history[0].status_code, Some(200));
    assert_eq!(history[0].duration_ms, Some(150));
    
    // Test intruder database operations
    let attack_id = db.create_intruder_attack(
        "Test Attack",
        "GET /api/test?param=§payload1§ HTTP/1.1\r\n\r\n",
        "sniper",
        r#"[{"id": "set1", "name": "Test Set", "payload_config": {"Custom": {"values": ["test1", "test2"]}}, "position_index": 0}]"#,
        r#"["test-agent-1", "test-agent-2"]"#,
        "round_robin",
    ).await.expect("Failed to create intruder attack");
    
    // Verify attack was created
    let attacks = db.get_intruder_attacks(Some(10))
        .await.expect("Failed to get attacks");
    assert_eq!(attacks.len(), 1);
    assert_eq!(attacks[0].name, "Test Attack");
    assert_eq!(attacks[0].attack_mode, "sniper");
    assert_eq!(attacks[0].status, "configured");
    
    // Save attack results
    let _result_id = db.save_intruder_result(
        &attack_id,
        &request_json,
        Some(response_json),
        "test-agent-1",
        r#"["test1"]"#,
        Some(120),
        Some(200),
        Some(1024),
        false,
    ).await.expect("Failed to save result");
    
    // Verify result was saved
    let results = db.get_intruder_results(&attack_id, Some(10), None)
        .await.expect("Failed to get results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].agent_id, "test-agent-1");
    assert_eq!(results[0].status_code, Some(200));
    assert_eq!(results[0].response_length, Some(1024));
    
    // Test payload sets
    let payload_config = r#"{"Custom": {"values": ["payload1", "payload2", "payload3"]}}"#;
    let _set_id = db.create_payload_set(
        "Test Payload Set",
        "custom",
        payload_config,
    ).await.expect("Failed to create payload set");
    
    // Verify payload set was created
    let sets = db.get_payload_sets().await.expect("Failed to get payload sets");
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].name, "Test Payload Set");
    assert_eq!(sets[0].payload_type, "custom");
    
    println!("✓ Database operations validation passed");
}

/// Test that RepeaterManager works correctly
#[tokio::test]
async fn test_repeater_manager_integration() {
    let (db, _temp_dir) = create_test_database().await;
    let agent_registry = Arc::new(AgentRegistry::new());
    let repeater_manager = RepeaterManager::new(db.clone(), agent_registry.clone());
    
    // Initialize the manager
    repeater_manager.initialize().await.expect("Failed to initialize repeater manager");
    
    // Create a test tab
    let request = create_test_request();
    let create_request = orchestrator::repeater::CreateRepeaterTabRequest {
        name: "Integration Test Tab".to_string(),
        request_template: request.clone(),
        target_agent_id: None, // No agent validation for this test
    };
    
    let tab_id = repeater_manager.create_tab(create_request)
        .await.expect("Failed to create tab");
    
    // Verify tab was created
    let tabs = repeater_manager.get_tabs().await;
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].name, "Integration Test Tab");
    assert_eq!(tabs[0].request_template.method, "POST");
    assert_eq!(tabs[0].request_template.url, "https://api.example.com/test");
    
    // Update the tab
    repeater_manager.update_tab(
        &tab_id,
        Some("Updated Tab Name".to_string()),
        None,
        None,
    ).await.expect("Failed to update tab");
    
    // Verify update
    let updated_tab = repeater_manager.get_tab(&tab_id).await.expect("Tab not found");
    assert_eq!(updated_tab.name, "Updated Tab Name");
    
    // Get execution statistics (should be empty)
    let stats = repeater_manager.get_execution_statistics(&tab_id)
        .await.expect("Failed to get statistics");
    assert_eq!(stats.total_executions, 0);
    assert_eq!(stats.successful_executions, 0);
    
    println!("✓ RepeaterManager integration validation passed");
}

/// Test that IntruderManager works correctly
#[tokio::test]
async fn test_intruder_manager_integration() {
    let (db, _temp_dir) = create_test_database().await;
    let intruder_manager = IntruderManager::new(db.clone())
        .await.expect("Failed to create intruder manager");
    
    // Create a test attack configuration
    let config = orchestrator::intruder::IntruderAttackConfig {
        name: "Integration Test Attack".to_string(),
        request_template: "GET /api/test?param=§payload1§ HTTP/1.1\r\nHost: example.com\r\n\r\n".to_string(),
        attack_mode: attack_engine::AttackMode::Sniper,
        payload_sets: vec![
            orchestrator::intruder::PayloadSetConfig {
                id: "test-set-1".to_string(),
                name: "Test Payloads".to_string(),
                payload_config: attack_engine::PayloadConfig::Custom {
                    values: vec!["test1".to_string(), "test2".to_string(), "test3".to_string()],
                },
                position_index: 0,
            }
        ],
        target_agents: vec!["test-agent-1".to_string()],
        distribution_strategy: attack_engine::DistributionStrategy::RoundRobin,
        session_data: None,
        execution_config: None,
    };
    
    // Validate the configuration
    let validation = intruder_manager.validate_attack_config(&config)
        .await.expect("Failed to validate config");
    
    assert!(validation.is_valid, "Config validation failed: {:?}", validation.errors);
    assert_eq!(validation.payload_positions.len(), 1);
    assert_eq!(validation.estimated_requests, Some(3)); // 3 payloads in sniper mode
    
    // Create the attack
    let attack_id = intruder_manager.create_attack(config)
        .await.expect("Failed to create attack");
    
    // Verify attack was created
    let attack = intruder_manager.get_attack(&attack_id)
        .await.expect("Failed to get attack")
        .expect("Attack not found");
    
    assert_eq!(attack.name, "Integration Test Attack");
    assert_eq!(attack.attack_mode, "sniper");
    assert_eq!(attack.status, "configured");
    
    // Update attack status
    intruder_manager.update_attack_status(&attack_id, "running")
        .await.expect("Failed to update status");
    
    // Verify status update
    let updated_attack = intruder_manager.get_attack(&attack_id)
        .await.expect("Failed to get attack")
        .expect("Attack not found");
    assert_eq!(updated_attack.status, "running");
    
    // Get attack statistics (should be empty)
    let stats = intruder_manager.get_attack_statistics(&attack_id)
        .await.expect("Failed to get statistics");
    
    let stats_obj: serde_json::Value = stats;
    assert_eq!(stats_obj["total_requests"], 0);
    assert_eq!(stats_obj["completed_requests"], 0);
    
    println!("✓ IntruderManager integration validation passed");
}

/// Test payload generation and parsing
#[tokio::test]
async fn test_payload_generation_and_parsing() {
    use attack_engine::{PayloadGeneratorFactory, PayloadConfig, PayloadPositionParser};
    
    // Test custom payload generation
    let custom_config = PayloadConfig::Custom {
        values: vec!["test1".to_string(), "test2".to_string(), "test3".to_string()],
    };
    
    let generator = PayloadGeneratorFactory::create(&custom_config)
        .expect("Failed to create generator");
    
    let payloads = generator.generate().await.expect("Failed to generate payloads");
    assert_eq!(payloads.len(), 3);
    assert_eq!(payloads, vec!["test1", "test2", "test3"]);
    
    let count = generator.count().await.expect("Failed to count payloads");
    assert_eq!(count, 3);
    
    // Test number range generation
    let number_config = PayloadConfig::NumberRange {
        start: 1,
        end: 5,
        step: 1,
        format: "{}".to_string(),
    };
    
    let number_generator = PayloadGeneratorFactory::create(&number_config)
        .expect("Failed to create number generator");
    
    let number_payloads = number_generator.generate().await.expect("Failed to generate numbers");
    assert_eq!(number_payloads.len(), 5);
    assert_eq!(number_payloads, vec!["1", "2", "3", "4", "5"]);
    
    // Test payload position parsing
    let template = "GET /api/test?param1=§payload1§&param2=§payload2§ HTTP/1.1\r\n\r\n";
    let parsed = PayloadPositionParser::parse(template).expect("Failed to parse template");
    
    assert_eq!(parsed.positions.len(), 2);
    assert_eq!(parsed.positions[0].start, template.find("§payload1§").unwrap());
    assert_eq!(parsed.positions[1].start, template.find("§payload2§").unwrap());
    
    println!("✓ Payload generation and parsing validation passed");
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() {
    let (db, _temp_dir) = create_test_database().await;
    let agent_registry = Arc::new(AgentRegistry::new());
    let repeater_manager = RepeaterManager::new(db.clone(), agent_registry.clone());
    
    // Test invalid request validation
    let invalid_request = HttpRequestData {
        method: "INVALID_METHOD".to_string(),
        url: "not-a-url".to_string(),
        headers: None,
        body: Vec::new(),
        tls: None,
    };
    
    let create_request = orchestrator::repeater::CreateRepeaterTabRequest {
        name: "Invalid Tab".to_string(),
        request_template: invalid_request,
        target_agent_id: None,
    };
    
    let result = repeater_manager.create_tab(create_request).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AttackError::InvalidPayloadConfig { reason } => {
            assert!(reason.contains("Invalid HTTP method") || reason.contains("must start with http"));
        }
        _ => panic!("Expected InvalidPayloadConfig error"),
    }
    
    // Test agent unavailable error
    let valid_request = create_test_request();
    let create_request = orchestrator::repeater::CreateRepeaterTabRequest {
        name: "Valid Tab".to_string(),
        request_template: valid_request.clone(),
        target_agent_id: Some("non-existent-agent".to_string()),
    };
    
    let result = repeater_manager.create_tab(create_request).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AttackError::AgentUnavailable { agent_id } => {
            assert_eq!(agent_id, "non-existent-agent");
        }
        _ => panic!("Expected AgentUnavailable error"),
    }
    
    println!("✓ Error handling validation passed");
}

/// Main integration test that validates all components work together
#[tokio::test]
async fn test_core_functionality_integration() {
    println!("🔄 Running core functionality validation...");
    
    // Create test database
    let (db, _temp_dir) = create_test_database().await;
    
    // Create a test agent first (required for foreign key constraint)
    db.upsert_agent("test-agent-1", "Test Agent", "localhost", "1.0.0")
        .await.expect("Failed to create test agent");
    
    // Test 1: Database operations
    println!("   Testing database operations...");
    let request = create_test_request();
    let request_json = serde_json::to_string(&request).expect("Failed to serialize request");
    
    // Create a repeater tab
    let tab_id = db.create_repeater_tab(
        "Test Tab",
        &request_json,
        Some("test-agent-1"),
    ).await.expect("Failed to create repeater tab");
    
    // Verify tab was created
    let tabs = db.get_repeater_tabs().await.expect("Failed to get repeater tabs");
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].name, "Test Tab");
    
    // Test 2: RepeaterManager integration
    println!("   Testing RepeaterManager integration...");
    let agent_registry = Arc::new(AgentRegistry::new());
    let repeater_manager = RepeaterManager::new(db.clone(), agent_registry.clone());
    
    repeater_manager.initialize().await.expect("Failed to initialize repeater manager");
    
    let create_request = orchestrator::repeater::CreateRepeaterTabRequest {
        name: "Integration Test Tab".to_string(),
        request_template: request.clone(),
        target_agent_id: None,
    };
    
    let tab_id2 = repeater_manager.create_tab(create_request)
        .await.expect("Failed to create tab");
    
    let tabs = repeater_manager.get_tabs().await;
    assert!(tabs.len() >= 1);
    
    // Test 3: IntruderManager integration
    println!("   Testing IntruderManager integration...");
    let intruder_manager = IntruderManager::new(db.clone())
        .await.expect("Failed to create intruder manager");
    
    let config = orchestrator::intruder::IntruderAttackConfig {
        name: "Integration Test Attack".to_string(),
        request_template: "GET /api/test?param=§payload1§ HTTP/1.1\r\nHost: example.com\r\n\r\n".to_string(),
        attack_mode: attack_engine::AttackMode::Sniper,
        payload_sets: vec![
            orchestrator::intruder::PayloadSetConfig {
                id: "test-set-1".to_string(),
                name: "Test Payloads".to_string(),
                payload_config: attack_engine::PayloadConfig::Custom {
                    values: vec!["test1".to_string(), "test2".to_string(), "test3".to_string()],
                },
                position_index: 0,
            }
        ],
        target_agents: vec!["test-agent-1".to_string()],
        distribution_strategy: attack_engine::DistributionStrategy::RoundRobin,
        session_data: None,
        execution_config: None,
    };
    
    let validation = intruder_manager.validate_attack_config(&config)
        .await.expect("Failed to validate config");
    
    assert!(validation.is_valid, "Config validation failed: {:?}", validation.errors);
    assert_eq!(validation.payload_positions.len(), 1);
    assert_eq!(validation.estimated_requests, Some(3));
    
    // Test 4: Payload generation
    println!("   Testing payload generation...");
    use attack_engine::{PayloadGeneratorFactory, PayloadConfig, PayloadPositionParser};
    
    let custom_config = PayloadConfig::Custom {
        values: vec!["test1".to_string(), "test2".to_string(), "test3".to_string()],
    };
    
    let generator = PayloadGeneratorFactory::create(&custom_config)
        .expect("Failed to create generator");
    
    let payloads = generator.generate().await.expect("Failed to generate payloads");
    assert_eq!(payloads.len(), 3);
    assert_eq!(payloads, vec!["test1", "test2", "test3"]);
    
    // Test 5: Error handling
    println!("   Testing error handling...");
    let invalid_request = HttpRequestData {
        method: "INVALID_METHOD".to_string(),
        url: "not-a-url".to_string(),
        headers: None,
        body: Vec::new(),
        tls: None,
    };
    
    let create_request = orchestrator::repeater::CreateRepeaterTabRequest {
        name: "Invalid Tab".to_string(),
        request_template: invalid_request,
        target_agent_id: None,
    };
    
    let result = repeater_manager.create_tab(create_request).await;
    assert!(result.is_err());
    
    println!("✅ All core functionality validation tests passed!");
    println!("   ✓ Database operations working correctly");
    println!("   ✓ RepeaterManager integration functional");
    println!("   ✓ IntruderManager integration functional");
    println!("   ✓ Payload generation and parsing working");
    println!("   ✓ Error handling working as expected");
    println!("   ✓ All components integrate successfully");
}