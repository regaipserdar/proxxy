use orchestrator::{Database, repeater::*};
use proptest::prelude::*;
use attack_engine::{HttpRequestData, HttpHeaders};
use std::collections::HashMap;

// Test data generators
prop_compose! {
    fn arb_http_request_data()(
        method in prop::sample::select(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]),
        url in "https?://[a-zA-Z0-9.-]+\\.[a-z]{2,}/[a-zA-Z0-9/_-]*",
        body in prop::option::of("[a-zA-Z0-9 {}\":,\\[\\]]{0,200}"),
    ) -> HttpRequestData {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Test Agent".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        HttpRequestData {
            method: method.to_string(),
            url,
            headers: Some(HttpHeaders { headers }),
            body: body.map(|s| s.into_bytes()).unwrap_or_else(Vec::new),
            tls: None,
        }
    }
}

prop_compose! {
    fn arb_repeater_tab_data()(
        name in "[a-zA-Z0-9 ]{1,50}",
        method in prop::sample::select(vec!["GET", "POST", "PUT", "DELETE", "PATCH"]),
        url in "https?://[a-zA-Z0-9.-]+\\.[a-z]{2,}/[a-zA-Z0-9/_-]*",
        // Use valid agent IDs that exist in test database (agent-0 through agent-4) or None
        agent_id_option in prop::option::of(prop::sample::select(vec![
            "agent-0", "agent-1", "agent-2", "agent-3", "agent-4"
        ])),
    ) -> (String, String, String, Option<String>) {
        let request_template = serde_json::json!({
            "method": method,
            "url": url,
            "headers": {"User-Agent": "Test Agent"},
            "body": ""
        }).to_string();
        
        // Convert &str to String for agent_id
        let agent_id = agent_id_option.map(|s| s.to_string());
        
        (name, request_template, url, agent_id)
    }
}

prop_compose! {
    fn arb_agent_selection_scenario()(
        primary_agent in prop::sample::select(vec!["agent-0", "agent-1", "agent-2", "agent-3", "agent-4"]),
        agent_status in prop::sample::select(vec!["Online", "Offline", "Connecting", "Error"]),
        request_data in arb_http_request_data(),
    ) -> (String, String, HttpRequestData) {
        (primary_agent.to_string(), agent_status.to_string(), request_data)
    }
}

async fn setup_test_db() -> Database {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    
    let db = Database::new(temp_dir.path().to_str().unwrap())
        .await
        .expect("Failed to create database");
    
    // Create a test project and load it
    db.create_project("test_project")
        .await
        .expect("Failed to create test project");
    
    db.load_project("test_project")
        .await
        .expect("Failed to load test project");
    
    // Create test agents for foreign key constraints
    for i in 0..5 {
        let agent_id = format!("agent-{}", i);
        db.upsert_agent(&agent_id, &format!("Test Agent {}", i), "localhost", "1.0.0")
            .await
            .expect("Failed to create test agent");
    }
    
    db
}

proptest! {
    /// **Feature: repeater-intruder, Property 2: Agent Selection and Routing**
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**
    /// 
    /// For any agent selection operation, the system should validate agent availability,
    /// route requests through selected agents, include agent identification in responses,
    /// and handle agent failures gracefully.
    #[test]
    fn prop_agent_selection_and_routing(
        (primary_agent, agent_status, request_data) in arb_agent_selection_scenario()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let db = setup_test_db().await;
            let agent_registry = std::sync::Arc::new(orchestrator::AgentRegistry::new());
            let repeater_manager = RepeaterManager::new(
                std::sync::Arc::new(db),
                agent_registry.clone(),
            );
            
            // Initialize the manager
            repeater_manager.initialize().await.expect("Failed to initialize manager");
            
            // Clone values for later use
            let primary_agent_clone = primary_agent.clone();
            let primary_agent_clone2 = primary_agent.clone();
            let agent_status_clone = agent_status.clone();
            
            // Test 1: Agent availability validation
            let availability_result = repeater_manager.validate_agent_availability(&primary_agent).await;
            
            if agent_status == "Online" {
                prop_assert!(availability_result.is_ok(), "Online agent should be available");
            } else {
                prop_assert!(availability_result.is_err(), "Non-online agent should be unavailable");
            }
            
            // Test 2: Create repeater tab with agent selection
            let create_request = CreateRepeaterTabRequest {
                name: "Test Tab".to_string(),
                request_template: request_data.clone(),
                target_agent_id: Some(primary_agent.clone()),
            };
            
            let tab_creation_result = repeater_manager.create_tab(create_request).await;
            
            if agent_status == "Online" {
                prop_assert!(tab_creation_result.is_ok(), "Tab creation should succeed with online agent");
                
                let tab_id = tab_creation_result.unwrap();
                
                // Test 3: Request execution with agent routing
                let execution_request = RepeaterExecutionRequest {
                    tab_id: tab_id.clone(),
                    request_data: request_data.clone(),
                    target_agent_id: primary_agent.clone(),
                    session_id: None,
                };
                
                let execution_result = repeater_manager.execute_request(execution_request).await;
                
                // For online agents, execution should succeed (with mock implementation)
                prop_assert!(execution_result.is_ok(), "Request execution should succeed with online agent");
                
                if let Ok(response) = execution_result {
                    // Test 4: Agent identification in response
                    prop_assert_eq!(response.agent_id, primary_agent_clone, "Response should include correct agent ID");
                    prop_assert!(response.duration_ms.is_some(), "Response should include timing information");
                    prop_assert!(response.executed_at <= chrono::Utc::now(), "Execution time should be valid");
                }
                
                // Test 5: Get available agents
                let available_agents = repeater_manager.get_available_agents().await;
                prop_assert!(!available_agents.is_empty(), "Should return list of agents");
                
                // Find our test agent in the list
                if let Some(agent_info) = available_agents.iter().find(|a| a.id == primary_agent_clone2) {
                    prop_assert_eq!(&agent_info.status, &agent_status_clone, "Agent status should match");
                    prop_assert_eq!(agent_info.is_available, agent_status_clone == "Online", "Availability should match status");
                }
            } else {
                // For non-online agents, tab creation should fail
                prop_assert!(tab_creation_result.is_err(), "Tab creation should fail with unavailable agent");
            }
            
            Ok(())
        });
        result?;
    }

    /// **Feature: repeater-intruder, Property 8: Data Persistence Consistency**
    /// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**
    /// 
    /// For any repeater tab configuration, creating and retrieving the tab should
    /// preserve all data fields and maintain referential integrity.
    #[test]
    fn prop_repeater_tab_persistence(
        (name, request_template, _url, target_agent_id) in arb_repeater_tab_data()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let db = setup_test_db().await;
            
            // Create repeater tab
            let tab_id = db.create_repeater_tab(&name, &request_template, target_agent_id.as_deref())
                .await
                .expect("Failed to create repeater tab");
            
            // Retrieve the tab
            let retrieved_tab = db.get_repeater_tab(&tab_id)
                .await
                .expect("Failed to get repeater tab")
                .expect("Tab should exist");
            
            // Verify all fields are preserved
            prop_assert_eq!(retrieved_tab.name, name);
            prop_assert_eq!(retrieved_tab.request_template, request_template);
            prop_assert_eq!(retrieved_tab.target_agent_id, target_agent_id);
            prop_assert_eq!(retrieved_tab.is_active, true);
            prop_assert!(retrieved_tab.created_at > 0);
            prop_assert!(retrieved_tab.updated_at > 0);
            
            // Verify tab appears in list
            let all_tabs = db.get_repeater_tabs()
                .await
                .expect("Failed to get all tabs");
            
            prop_assert!(all_tabs.iter().any(|t| t.id == tab_id));
            
            Ok(())
        });
        result?;
    }
    
    /// **Feature: repeater-intruder, Property 8: Data Persistence Consistency**
    /// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**
    /// 
    /// For any intruder attack configuration, creating and retrieving the attack should
    /// preserve all data fields and support status transitions.
    #[test]
    fn prop_intruder_attack_persistence(
        name in "[a-zA-Z0-9 ]{1,50}",
        attack_mode in prop::sample::select(vec!["sniper", "battering_ram", "pitchfork", "cluster_bomb"]),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let db = setup_test_db().await;
            
            let request_template = "POST /api/test HTTP/1.1\r\nHost: example.com\r\n\r\n{\"param\": \"§payload§\"}";
            let payload_sets = r#"[{"id": "test", "name": "test_payloads", "type": "custom", "values": ["test1", "test2"]}]"#;
            // Use valid agent IDs that exist in test database
            let target_agents = r#"["agent-0", "agent-1"]"#;
            let distribution_strategy = "round_robin";
            
            // Create intruder attack
            let attack_id = db.create_intruder_attack(
                &name,
                request_template,
                &attack_mode,
                payload_sets,
                target_agents,
                distribution_strategy,
            )
            .await
            .expect("Failed to create intruder attack");
            
            // Retrieve the attack
            let retrieved_attack = db.get_intruder_attack(&attack_id)
                .await
                .expect("Failed to get intruder attack")
                .expect("Attack should exist");
            
            // Verify all fields are preserved
            prop_assert_eq!(retrieved_attack.name, name);
            prop_assert_eq!(retrieved_attack.request_template, request_template);
            prop_assert_eq!(retrieved_attack.attack_mode, attack_mode);
            prop_assert_eq!(retrieved_attack.status, "configured");
            prop_assert!(retrieved_attack.created_at > 0);
            prop_assert!(retrieved_attack.updated_at > 0);
            
            // Test status transitions
            db.update_intruder_attack_status(&attack_id, "running")
                .await
                .expect("Failed to update status");
            
            let updated_attack = db.get_intruder_attack(&attack_id)
                .await
                .expect("Failed to get updated attack")
                .expect("Attack should exist");
            
            prop_assert_eq!(updated_attack.status, "running");
            prop_assert!(updated_attack.updated_at >= retrieved_attack.updated_at);
            
            Ok(())
        });
        result?;
    }
}

// Additional unit tests for edge cases
#[tokio::test]
async fn test_repeater_tab_soft_delete() {
    let db = setup_test_db().await;
    
    let tab_id = db.create_repeater_tab("Test Tab", r#"{"method": "GET"}"#, None)
        .await
        .expect("Failed to create tab");
    
    // Verify tab exists and is active
    let tab = db.get_repeater_tab(&tab_id)
        .await
        .expect("Failed to get tab")
        .expect("Tab should exist");
    assert!(tab.is_active);
    
    // Delete tab (soft delete)
    db.delete_repeater_tab(&tab_id)
        .await
        .expect("Failed to delete tab");
    
    // Verify tab still exists but is inactive
    let deleted_tab = db.get_repeater_tab(&tab_id)
        .await
        .expect("Failed to get deleted tab")
        .expect("Tab should still exist");
    assert!(!deleted_tab.is_active);
    
    // Verify tab doesn't appear in active list
    let active_tabs = db.get_repeater_tabs()
        .await
        .expect("Failed to get active tabs");
    assert!(!active_tabs.iter().any(|t| t.id == tab_id));
}

#[tokio::test]
async fn test_intruder_attack_cascade_delete() {
    let db = setup_test_db().await;
    
    let attack_id = db.create_intruder_attack(
        "Test Attack",
        "GET /test",
        "sniper",
        r#"[{"type": "custom", "values": ["test"]}]"#,
        r#"["agent-0"]"#,
        "round_robin",
    )
    .await
    .expect("Failed to create attack");
    
    // Add some results
    db.save_intruder_result(
        &attack_id,
        r#"{"request": "test"}"#,
        Some(r#"{"response": "ok"}"#),
        "agent-0",
        r#"["test"]"#,
        Some(100),
        Some(200),
        Some(50),
        false,
    )
    .await
    .expect("Failed to save result");
    
    // Verify result exists
    let results = db.get_intruder_results(&attack_id, None, None)
        .await
        .expect("Failed to get results");
    assert_eq!(results.len(), 1);
    
    // Delete attack (should cascade delete results)
    db.delete_intruder_attack(&attack_id)
        .await
        .expect("Failed to delete attack");
    
    // Verify attack is gone
    let deleted_attack = db.get_intruder_attack(&attack_id)
        .await
        .expect("Failed to check deleted attack");
    assert!(deleted_attack.is_none());
    
    // Verify results are gone too
    let remaining_results = db.get_intruder_results(&attack_id, None, None)
        .await
        .expect("Failed to get remaining results");
    assert_eq!(remaining_results.len(), 0);
}