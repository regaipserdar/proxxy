use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::Status;
use crate::pb::InterceptCommand;
use serde::Serialize;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentData {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub address: String,
    pub port: u16,
    pub status: String,
    pub last_heartbeat: String,
    pub version: String,
    pub capabilities: Vec<String>,
    #[serde(skip)]
    pub command_tx: mpsc::Sender<Result<InterceptCommand, Status>>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    /// Active agents: AgentID -> AgentData
    agents: Arc<DashMap<String, AgentData>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
        }
    }

    pub fn register_agent(
        &self,
        id: String,
        name: String,
        hostname: String,
        command_tx: mpsc::Sender<Result<InterceptCommand, Status>>,
    ) {
        let agent = AgentData {
            id: id.clone(),
            name,
            hostname,
            address: "127.0.0.1".to_string(), // Placeholder, ideally get from request remote addr
            port: 9095, // Placeholder, normally sent in registration
            status: "Online".to_string(),
            last_heartbeat: chrono::Utc::now().to_rfc3339(),
            version: "0.1.0".to_string(),
            capabilities: vec!["HTTP".to_string(), "HTTPS".to_string()],
            command_tx,
        };
        self.agents.insert(id, agent);
    }

    pub fn get_agent_tx(&self, id: &str) -> Option<mpsc::Sender<Result<InterceptCommand, Status>>> {
        self.agents.get(id).map(|a| a.command_tx.clone())
    }

    pub fn list_agents(&self) -> Vec<AgentData> {
        self.agents.iter().map(|entry| entry.value().clone()).collect()
    }
}
