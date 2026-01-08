use orchestrator::AgentRegistry;
use tokio::sync::mpsc;
use tonic::Status;

#[test]
fn test_register_and_get() {
    let registry = AgentRegistry::new();
    let agent_id = "agent-1".to_string();
    let (tx, _rx) = mpsc::channel(1);

    registry.register_agent(agent_id.clone(), "localhost".to_string(), tx);

    let retrieved = registry.get_agent_tx(&agent_id);
    assert!(retrieved.is_some(), "Should retrieve registered agent");

    let missing = registry.get_agent_tx("agent-2");
    assert!(missing.is_none(), "Should not retrieve missing agent");
}
