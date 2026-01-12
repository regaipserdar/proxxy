use orchestrator::graphql::{IntruderAttackGql, IntruderResultGql, PayloadSetGql};
use orchestrator::database::intruder::{IntruderAttack, IntruderResult, PayloadSet};

#[test]
fn test_intruder_attack_gql_conversion() {
    let attack = IntruderAttack {
        id: "test-attack-1".to_string(),
        name: "Test Attack".to_string(),
        request_template: "GET /api/test?param=§payload1§ HTTP/1.1\r\n\r\n".to_string(),
        attack_mode: "sniper".to_string(),
        payload_sets: r#"[{"id":"set1","name":"Test Set","payload_config":{"Custom":{"values":["test1","test2"]}},"position_index":0}]"#.to_string(),
        target_agents: r#"["agent1","agent2"]"#.to_string(),
        distribution_strategy: "round_robin".to_string(),
        created_at: 1640995200, // 2022-01-01 00:00:00 UTC
        updated_at: 1640995200,
        status: "configured".to_string(),
    };

    let gql_attack = IntruderAttackGql::from(attack);

    assert_eq!(gql_attack.id, "test-attack-1");
    assert_eq!(gql_attack.name, "Test Attack");
    assert_eq!(gql_attack.attack_mode, "sniper");
    assert_eq!(gql_attack.target_agents, vec!["agent1", "agent2"]);
    assert_eq!(gql_attack.distribution_strategy, "round_robin");
    assert_eq!(gql_attack.status, "configured");
    assert!(gql_attack.created_at.contains("2022-01-01"));
}

#[test]
fn test_intruder_result_gql_conversion() {
    let result = IntruderResult {
        id: "result-1".to_string(),
        attack_id: "attack-1".to_string(),
        request_data: r#"{"method":"GET","url":"https://example.com/api/test?param=test1","headers":null,"body":[],"tls":null}"#.to_string(),
        response_data: Some(r#"{"status_code":200,"headers":null,"body":[123,34,115,117,99,99,101,115,115,34,58,116,114,117,101,125],"body_length":16}"#.to_string()),
        agent_id: "agent1".to_string(),
        payload_values: r#"["test1"]"#.to_string(),
        executed_at: 1640995200,
        duration_ms: Some(150),
        status_code: Some(200),
        response_length: Some(16),
        is_highlighted: false,
    };

    let gql_result = IntruderResultGql::from(result);

    assert_eq!(gql_result.id, "result-1");
    assert_eq!(gql_result.attack_id, "attack-1");
    assert_eq!(gql_result.agent_id, "agent1");
    assert_eq!(gql_result.duration_ms, Some(150));
    assert_eq!(gql_result.status_code, Some(200));
    assert_eq!(gql_result.response_length, Some(16));
    assert!(!gql_result.is_highlighted);
    assert!(gql_result.executed_at.contains("2022-01-01"));
}

#[test]
fn test_payload_set_gql_conversion() {
    let payload_set = PayloadSet {
        id: "set-1".to_string(),
        name: "Test Payload Set".to_string(),
        payload_type: "custom".to_string(),
        configuration: r#"{"Custom":{"values":["payload1","payload2","payload3"]}}"#.to_string(),
        created_at: 1640995200,
    };

    let gql_set = PayloadSetGql::from(payload_set);

    assert_eq!(gql_set.id, "set-1");
    assert_eq!(gql_set.name, "Test Payload Set");
    assert_eq!(gql_set.payload_type, "custom");
    assert!(gql_set.created_at.contains("2022-01-01"));
}