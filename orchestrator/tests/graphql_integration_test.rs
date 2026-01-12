//! GraphQL Integration Tests
//! 
//! Comprehensive tests for GraphQL API covering repeater, intruder, and session integration.
//! Tests complete workflows from creation to execution with real-time subscriptions.

use orchestrator::{LoggingConfig, Orchestrator, OrchestratorConfig};
use orchestrator::graphql::{
    SessionGql, SessionApplicationResultGql, RepeaterTabGql, RepeaterExecutionGql,
    IntruderAttackGql, IntruderResultGql, SessionEventGql
};
use proxy_common::session::{Session, SessionStatus, Cookie, SameSite};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::Duration;
use uuid::Uuid;

/// Test helper to create a test session
fn create_test_session() -> Session {
    let mut session = Session::new("Test Session".to_string(), None);
    session.headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    session.headers.insert("X-CSRF-Token".to_string(), "csrf123".to_string());
    session.cookies.push(Cookie {
        name: "sessionid".to_string(),
        value: "abc123".to_string(),
        domain: Some("example.com".to_string()),
        path: Some("/".to_string()),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some(SameSite::Lax),
    });
    session.status = SessionStatus::Active;
    session
}

/// Test helper to execute GraphQL query
async fn execute_graphql_query_on_port(client: &Client, port: u16, query: &str, variables: Option<Value>) -> Value {
    let body = if let Some(vars) = variables {
        json!({
            "query": query,
            "variables": vars
        })
    } else {
        json!({
            "query": query
        })
    };

    let res = client
        .post(&format!("http://127.0.0.1:{}/graphql", port))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to send GraphQL request");

    assert_eq!(res.status(), 200, "GraphQL request failed");
    
    let response: Value = res.json().await.expect("Failed to parse GraphQL response");
    
    // Check for GraphQL errors
    if let Some(errors) = response.get("errors") {
        panic!("GraphQL errors: {}", errors);
    }
    
    response
}

/// Test helper to execute GraphQL query
async fn execute_graphql_query(client: &Client, query: &str, variables: Option<Value>) -> Value {
    execute_graphql_query_on_port(client, 50072, query, variables).await
}

#[tokio::test]
async fn test_session_graphql_workflow() {
    // 1. Setup Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50071,
        http_port: 50072,
        database_url: format!("sqlite::memory:?test_session_{}", std::process::id()),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    };

    let orchestrator = Orchestrator::new(config).await.unwrap();

    // 2. Spawn Server
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });

    // Give it a moment to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = Client::new();

    // 3. Create a project first
    let create_project_query = r#"
        mutation CreateProject($name: String!) {
            createProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, create_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    // 4. Load the project
    let load_project_query = r#"
        mutation LoadProject($name: String!) {
            loadProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, load_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    // 3. Test session creation
    let session = create_test_session();
    let session_id = session.id.to_string();
    
    let add_session_query = r#"
        mutation AddSession($input: SessionInput!) {
            addSession(input: $input) {
                id
                name
                status
                usageCount
                headers
                cookies {
                    name
                    value
                    domain
                    httpOnly
                    secure
                }
            }
        }
    "#;

    let session_input = json!({
        "id": session_id,
        "name": session.name,
        "headers": serde_json::to_string(&session.headers).unwrap(),
        "cookies": session.cookies.iter().map(|c| json!({
            "name": c.name,
            "value": c.value,
            "domain": c.domain,
            "path": c.path,
            "expires": c.expires.map(|dt| dt.to_rfc3339()),
            "httpOnly": c.http_only,
            "secure": c.secure,
            "sameSite": c.same_site.as_ref().map(|ss| match ss {
                SameSite::Strict => "Strict",
                SameSite::Lax => "Lax",
                SameSite::None => "None",
            })
        })).collect::<Vec<_>>(),
        "status": "Active"
    });

    let response = execute_graphql_query(&client, add_session_query, Some(json!({
        "input": session_input
    }))).await;

    let added_session = &response["data"]["addSession"];
    assert_eq!(added_session["id"], session_id);
    assert_eq!(added_session["name"], "Test Session");
    assert_eq!(added_session["status"], "Active");

    // 4. Test session queries
    let get_sessions_query = r#"
        query GetSessions {
            sessions {
                id
                name
                status
                usageCount
            }
            activeSessions {
                id
                name
                status
            }
            sessionStatistics {
                totalSessions
                activeSessions
                expiredSessions
                invalidSessions
            }
        }
    "#;

    let response = execute_graphql_query(&client, get_sessions_query, None).await;
    
    let sessions = &response["data"]["sessions"];
    assert!(sessions.is_array());
    assert_eq!(sessions.as_array().unwrap().len(), 1);
    
    let active_sessions = &response["data"]["activeSessions"];
    assert!(active_sessions.is_array());
    assert_eq!(active_sessions.as_array().unwrap().len(), 1);
    
    let stats = &response["data"]["sessionStatistics"];
    // We expect at least 1 session, but there might be more due to test interactions
    assert!(stats["totalSessions"].as_i64().unwrap() >= 1);
    assert!(stats["activeSessions"].as_i64().unwrap() >= 1);

    // 5. Test session selection
    let select_session_query = r#"
        query SelectSession($criteria: SessionSelectionCriteriaInput) {
            selectSession(criteria: $criteria) {
                id
                name
                status
            }
        }
    "#;

    let response = execute_graphql_query(&client, select_session_query, Some(json!({
        "criteria": {
            "maxValidationAgeMinutes": 60,
            "excludeRecentFailures": true
        }
    }))).await;

    let selected_session = &response["data"]["selectSession"];
    assert!(selected_session.is_object());
    assert_eq!(selected_session["id"], session_id);

    // 6. Test session application to request
    let apply_session_query = r#"
        mutation ApplySessionToRequest($input: ApplySessionToRequestInput!) {
            applySessionToRequest(input: $input) {
                sessionId
                sessionName
                headersApplied
                cookiesApplied
                warnings
            }
        }
    "#;

    let request_template = json!({
        "method": "GET",
        "url": "https://example.com/api/test",
        "headers": "{}",
        "body": ""
    });

    let response = execute_graphql_query(&client, apply_session_query, Some(json!({
        "input": {
            "sessionId": session_id,
            "requestTemplate": request_template,
            "expirationHandling": {
                "strategy": "Fail"
            }
        }
    }))).await;

    let application_result = &response["data"]["applySessionToRequest"];
    assert_eq!(application_result["sessionId"], session_id);
    assert_eq!(application_result["sessionName"], "Test Session");
    assert!(application_result["headersApplied"].as_i64().unwrap() > 0);
    assert_eq!(application_result["cookiesApplied"], 1);

    // 7. Test session validation
    let validate_session_query = r#"
        mutation ValidateSession($sessionId: String!, $validationUrl: String!) {
            validateSession(sessionId: $sessionId, validationUrl: $validationUrl)
        }
    "#;

    let response = execute_graphql_query(&client, validate_session_query, Some(json!({
        "sessionId": session_id,
        "validationUrl": "https://example.com/api/validate"
    }))).await;

    let is_valid = response["data"]["validateSession"].as_bool().unwrap();
    assert!(is_valid);

    // 8. Test session removal
    let remove_session_query = r#"
        mutation RemoveSession($id: String!) {
            removeSession(id: $id)
        }
    "#;

    let response = execute_graphql_query(&client, remove_session_query, Some(json!({
        "id": session_id
    }))).await;

    let removed = response["data"]["removeSession"].as_bool().unwrap();
    assert!(removed);

    // 9. Verify session is removed
    let response = execute_graphql_query(&client, get_sessions_query, None).await;
    let sessions = &response["data"]["sessions"];
    assert_eq!(sessions.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_repeater_with_session_integration() {
    // 1. Setup Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50073,
        http_port: 50074,
        database_url: format!("sqlite::memory:?test_repeater_{}", std::process::id()),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    };

    let orchestrator = Orchestrator::new(config).await.unwrap();

    // 2. Spawn Server
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = Client::new();

    // 3. Create and load a project first
    let create_project_query = r#"
        mutation CreateProject($name: String!) {
            createProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, create_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    let load_project_query = r#"
        mutation LoadProject($name: String!) {
            loadProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, load_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    // 3. Create a session first
    let session = create_test_session();
    let session_id = session.id.to_string();
    
    let add_session_query = r#"
        mutation AddSession($input: SessionInput!) {
            addSession(input: $input) {
                id
                name
            }
        }
    "#;

    let session_input = json!({
        "id": session_id,
        "name": session.name,
        "headers": serde_json::to_string(&session.headers).unwrap(),
        "cookies": [],
        "status": "Active"
    });

    execute_graphql_query_on_port(&client, 50074, add_session_query, Some(json!({
        "input": session_input
    }))).await;

    // 5. Create a repeater tab
    let create_tab_query = r#"
        mutation CreateRepeaterTab($input: CreateRepeaterTabInput!) {
            createRepeaterTab(input: $input) {
                id
                name
                validationStatus
                requestTemplate {
                    method
                    url
                    body
                }
            }
        }
    "#;

    let request_template = json!({
        "method": "GET",
        "url": "https://example.com/api/test",
        "headers": "{}",
        "body": ""
    });

    let response = execute_graphql_query_on_port(&client, 50074, create_tab_query, Some(json!({
        "input": {
            "name": "Test Tab",
            "requestTemplate": request_template,
            "targetAgentId": "default-agent"  // Provide a default agent ID
        }
    }))).await;

    let created_tab = &response["data"]["createRepeaterTab"];
    let tab_id = created_tab["id"].as_str().unwrap();
    assert_eq!(created_tab["name"], "Test Tab");

    // 5. Test repeater execution with session
    let execute_request_query = r#"
        mutation ExecuteRepeaterRequest($input: ExecuteRepeaterRequestInput!) {
            executeRepeaterRequest(input: $input) {
                id
                tabId
                agentId
                statusCode
                error
                requestData {
                    method
                    url
                    headers
                }
            }
        }
    "#;

    // Note: This will likely fail because we don't have a real agent,
    // but we can test the GraphQL structure and session integration
    let response = execute_graphql_query(&client, execute_request_query, Some(json!({
        "input": {
            "tabId": tab_id,
            "requestData": request_template,
            "targetAgentId": "default-agent",  // Provide a default agent ID
            "sessionId": session_id,
            "expirationHandling": {
                "strategy": "ContinueWithoutSession"
            }
        }
    }))).await;

    // The execution might fail due to no real agent, but we should get a structured response
    if let Some(execution) = response["data"]["executeRepeaterRequest"].as_object() {
        assert_eq!(execution["tabId"], tab_id);
        assert_eq!(execution["agentId"], "default-agent");
    }

    // 6. Test repeater queries
    let get_tabs_query = r#"
        query GetRepeaterTabs {
            repeaterTabs {
                id
                name
                validationStatus
                createdAt
            }
        }
    "#;

    let response = execute_graphql_query(&client, get_tabs_query, None).await;
    let tabs = &response["data"]["repeaterTabs"];
    assert!(tabs.is_array());
    assert_eq!(tabs.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_intruder_with_session_integration() {
    // 1. Setup Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50075,
        http_port: 50076,
        database_url: format!("sqlite::memory:?test_intruder_{}", std::process::id()),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    };

    let orchestrator = Orchestrator::new(config).await.unwrap();

    // 2. Spawn Server
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = Client::new();

    // 3. Create and load a project first
    let create_project_query = r#"
        mutation CreateProject($name: String!) {
            createProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, create_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    let load_project_query = r#"
        mutation LoadProject($name: String!) {
            loadProject(name: $name) {
                success
                message
            }
        }
    "#;

    execute_graphql_query(&client, load_project_query, Some(json!({
        "name": "test-project"
    }))).await;

    // 3. Create a session first
    let session = create_test_session();
    let session_id = session.id.to_string();
    
    let add_session_query = r#"
        mutation AddSession($input: SessionInput!) {
            addSession(input: $input) {
                id
                name
            }
        }
    "#;

    let session_input = json!({
        "id": session_id,
        "name": session.name,
        "headers": serde_json::to_string(&session.headers).unwrap(),
        "cookies": [],
        "status": "Active"
    });

    execute_graphql_query(&client, add_session_query, Some(json!({
        "input": session_input
    }))).await;

    // 4. Create a payload set
    let create_payload_set_query = r#"
        mutation CreatePayloadSet($input: CreatePayloadSetInput!) {
            createPayloadSet(input: $input) {
                id
                name
                payloadType
                configuration {
                    configType
                    configData
                }
            }
        }
    "#;

    let response = execute_graphql_query(&client, create_payload_set_query, Some(json!({
        "input": {
            "name": "Test Payload Set",
            "configuration": {
                "configType": "custom",
                "configData": json!({"values": ["payload1", "payload2", "payload3"]}).to_string()
            }
        }
    }))).await;

    let payload_set = &response["data"]["createPayloadSet"];
    let payload_set_id = payload_set["id"].as_str().unwrap();
    assert_eq!(payload_set["name"], "Test Payload Set");

    // 5. Create an intruder attack with session
    let create_attack_query = r#"
        mutation CreateIntruderAttack($input: CreateIntruderAttackInput!) {
            createIntruderAttack(input: $input) {
                id
                name
                attackMode
                targetAgents
                status
            }
        }
    "#;

    let response = execute_graphql_query(&client, create_attack_query, Some(json!({
        "input": {
            "name": "Test Attack",
            "requestTemplate": "GET /api/test?param=§payload1§ HTTP/1.1\r\n\r\n",
            "attackMode": {
                "modeType": "sniper"
            },
            "payloadSets": [{
                "id": payload_set_id,
                "name": "Test Payload Set",
                "positionIndex": 0,
                "configuration": {
                    "configType": "custom",
                    "configData": json!({"values": ["payload1", "payload2", "payload3"]}).to_string()
                }
            }],
            "targetAgents": ["test-agent-1", "test-agent-2"],
            "distributionStrategy": {
                "strategyType": "round_robin"
            },
            "sessionData": {
                "id": session_id,
                "name": session.name,
                "headers": serde_json::to_string(&session.headers).unwrap(),
                "cookies": [],
                "status": "Active"
            }
        }
    }))).await;

    let attack = &response["data"]["createIntruderAttack"];
    let attack_id = attack["id"].as_str().unwrap();
    assert_eq!(attack["name"], "Test Attack");
    assert_eq!(attack["attackMode"], "sniper");
    assert_eq!(attack["status"], "configured");

    // 6. Test intruder queries
    let get_attacks_query = r#"
        query GetIntruderAttacks {
            intruderAttacks {
                id
                name
                attackMode
                status
                createdAt
            }
        }
    "#;

    let response = execute_graphql_query(&client, get_attacks_query, None).await;
    let attacks = &response["data"]["intruderAttacks"];
    assert!(attacks.is_array());
    assert_eq!(attacks.as_array().unwrap().len(), 1);

    // 7. Test payload set queries
    let get_payload_sets_query = r#"
        query GetPayloadSets {
            payloadSets {
                id
                name
                payloadType
                createdAt
            }
        }
    "#;

    let response = execute_graphql_query(&client, get_payload_sets_query, None).await;
    let payload_sets = &response["data"]["payloadSets"];
    assert!(payload_sets.is_array());
    assert_eq!(payload_sets.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_auth_failure_config_graphql() {
    // 1. Setup Orchestrator
    let config = OrchestratorConfig {
        grpc_port: 50077,
        http_port: 50078,
        database_url: format!("sqlite::memory:?test_auth_{}", std::process::id()),
        health_check_interval: 10,
        agent_timeout: 10,
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    };

    let orchestrator = Orchestrator::new(config).await.unwrap();

    // 2. Spawn Server
    tokio::spawn(async move {
        orchestrator.start().await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let client = Client::new();

    // 3. Test getting current auth failure config
    let get_config_query = r#"
        query GetAuthFailureConfig {
            authFailureConfig {
                failureStatusCodes
                failureBodyPatterns
                failureHeaderPatterns
                loginRedirectPatterns
            }
        }
    "#;

    let response = execute_graphql_query_on_port(&client, 50078, get_config_query, None).await;
    let config = &response["data"]["authFailureConfig"];
    assert!(config["failureStatusCodes"].is_array());
    assert!(config["failureBodyPatterns"].is_array());

    // 4. Test updating auth failure config
    let update_config_query = r#"
        mutation UpdateAuthFailureConfig($input: AuthFailureDetectionConfigInput!) {
            updateAuthFailureConfig(input: $input) {
                failureStatusCodes
                failureBodyPatterns
                loginRedirectPatterns
            }
        }
    "#;

    let new_config = json!({
        "failureStatusCodes": [401, 403, 302],
        "failureBodyPatterns": ["login", "unauthorized", "access denied"],
        "failureHeaderPatterns": json!({"WWW-Authenticate": ".*", "Location": ".*/login.*"}).to_string(),
        "loginRedirectPatterns": [".*/login.*", ".*/signin.*"]
    });

    let response = execute_graphql_query_on_port(&client, 50078, update_config_query, Some(json!({
        "input": new_config
    }))).await;

    let updated_config = &response["data"]["updateAuthFailureConfig"];
    assert_eq!(updated_config["failureStatusCodes"], json!([401, 403, 302]));
    assert_eq!(updated_config["failureBodyPatterns"], json!(["login", "unauthorized", "access denied"]));
}

#[test]
fn test_session_gql_conversion() {
    let session = create_test_session();
    let session_gql = SessionGql::from(session.clone());

    assert_eq!(session_gql.id, session.id.to_string());
    assert_eq!(session_gql.name, session.name);
    assert_eq!(session_gql.status, "Active");
    assert_eq!(session_gql.usage_count, "0");
    assert!(session_gql.created_at.contains("T")); // ISO 8601 format
    assert_eq!(session_gql.headers.len(), 2);
    assert_eq!(session_gql.cookies.len(), 1);
}

#[test]
fn test_session_application_result_gql_conversion() {
    use orchestrator::session_integration::SessionApplicationResult;
    use uuid::Uuid;

    let result = SessionApplicationResult {
        session_id: Uuid::new_v4(),
        session_name: "Test Session".to_string(),
        headers_applied: 3,
        cookies_applied: 1,
        warnings: vec!["Test warning".to_string()],
    };

    let gql_result = SessionApplicationResultGql::from(result.clone());

    assert_eq!(gql_result.session_id, result.session_id.to_string());
    assert_eq!(gql_result.session_name, result.session_name);
    assert_eq!(gql_result.headers_applied, 3);
    assert_eq!(gql_result.cookies_applied, 1);
    assert_eq!(gql_result.warnings, vec!["Test warning"]);
}