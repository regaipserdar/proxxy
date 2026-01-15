use crate::pb::InterceptCommand;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::Status;

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
    // Metrics
    pub cpu_usage: f32,
    pub memory_usage_mb: f64,
    pub uptime_seconds: u64,
    pub public_ip: String,
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
            port: 9095,                       // Placeholder, normally sent in registration
            status: "Online".to_string(),
            last_heartbeat: chrono::Utc::now().to_rfc3339(),
            version: "0.1.0".to_string(),
            capabilities: vec!["HTTP".to_string(), "HTTPS".to_string()],
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: 0,
            public_ip: String::new(),
            command_tx,
        };
        self.agents.insert(id, agent);
    }

    pub fn get_agent_tx(&self, id: &str) -> Option<mpsc::Sender<Result<InterceptCommand, Status>>> {
        self.agents.get(id).map(|a| a.command_tx.clone())
    }

    pub fn get_agent(&self, id: &str) -> Option<AgentData> {
        self.agents.get(id).map(|a| a.value().clone())
    }

    pub fn list_agents(&self) -> Vec<AgentData> {
        self.agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn remove_agent(&self, id: &str) {
        self.agents.remove(id);
    }

    pub fn update_heartbeat(&self, id: &str, cpu: f32, mem: f64, uptime: u64, ip: String) {
        if let Some(mut agent) = self.agents.get_mut(id) {
            agent.last_heartbeat = chrono::Utc::now().to_rfc3339();
            agent.cpu_usage = cpu;
            agent.memory_usage_mb = mem;
            agent.uptime_seconds = uptime;
            if !ip.is_empty() {
                agent.public_ip = ip;
            }
        }
    }
}
