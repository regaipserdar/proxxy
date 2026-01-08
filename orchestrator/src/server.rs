use crate::pb::proxy_service_server::ProxyService;
use crate::pb::{RegisterAgentRequest, RegisterAgentResponse, TrafficEvent, InterceptCommand, SystemMetricsEvent, MetricsCommand};
use crate::AgentRegistry;
use tokio::sync::{mpsc, broadcast};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{info, warn, error, debug};
use std::sync::Arc;

use crate::Database;

use proxy_core::CertificateAuthority;

pub struct ProxyServiceImpl {
    agent_registry: Arc<AgentRegistry>,
    broadcast_tx: broadcast::Sender<TrafficEvent>,
    metrics_broadcast_tx: broadcast::Sender<SystemMetricsEvent>,
    db: Arc<Database>,
    ca: Arc<CertificateAuthority>,
}

impl ProxyServiceImpl {
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        broadcast_tx: broadcast::Sender<TrafficEvent>,
        metrics_broadcast_tx: broadcast::Sender<SystemMetricsEvent>,
        db: Arc<Database>,
        ca: Arc<CertificateAuthority>,
    ) -> Self {
        Self { agent_registry, broadcast_tx, metrics_broadcast_tx, db, ca }
    }
}

#[tonic::async_trait]
impl ProxyService for ProxyServiceImpl {
    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let req = request.into_inner();
        let agent_id = req.agent_id.clone();
        
        info!("🔌 Agent registration request:");
        info!("   • ID: {}", agent_id);
        info!("   • Name: {}", req.name);
        info!("   • Hostname: {}", req.hostname);
        info!("   • Version: {}", req.version);
        
        // In the future, we might validation auth tokens here.
        
        // Upsert agent to database
        if let Err(e) = self.db.upsert_agent(&agent_id, &req.name, &req.hostname, &req.version).await {
            error!("   ✗ Failed to upsert agent to DB: {}", e);
        } else {
            info!("   ✓ Agent saved to database");
        }
        
        // Read CA cert/key to send back to agent
        let ca_cert_pem = self.ca.get_ca_cert_pem().unwrap_or_default();
        let ca_key_pem = self.ca.get_ca_key_pem().unwrap_or_default();

        info!("   ✓ Sending CA credentials (cert: {} bytes, key: {} bytes)", 
            ca_cert_pem.len(), ca_key_pem.len());

        Ok(Response::new(RegisterAgentResponse {
            success: true,
            message: "Registered successfully".into(),
            ca_cert_pem,
            ca_key_pem,
        }))
    }

    type StreamTrafficStream = ReceiverStream<Result<InterceptCommand, Status>>;

    async fn stream_traffic(
        &self,
        request: Request<Streaming<TrafficEvent>>,
    ) -> Result<Response<Self::StreamTrafficStream>, Status> {
        let agent_id = match request.metadata().get("x-agent-id") {
            Some(id) => id.to_str().unwrap_or("unknown").to_string(),
            None => {
                warn!("⚠️  Stream started without x-agent-id metadata");
                "unknown".to_string()
            }
        };

        info!("📡 Agent {} connected for traffic streaming", agent_id);

        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(100);
        
        // Register the command channel
        let agent_name = match self.db.get_agent_name(&agent_id).await {
            Ok(Some(n)) => {
                info!("   ✓ Found agent name in DB: {}", n);
                n
            },
            Ok(None) => {
                warn!("   ⚠️  Agent {} not found in DB, using 'Unknown'", agent_id);
                "Unknown".to_string()
            },
            Err(e) => {
                warn!("   ✗ Failed to fetch agent name: {}", e);
                "Unknown".to_string()
            }
        };
        
        self.agent_registry.register_agent(agent_id.clone(), agent_name, "unknown".to_string(), tx);
        info!("   ✓ Agent registered in session manager");
        
        let broadcast = self.broadcast_tx.clone();
        let db = self.db.clone();
        let agent_id_cl = agent_id.clone();

        // Spawn task to handle inbound traffic events
        tokio::spawn(async move {
            let mut event_count = 0;
            while let Ok(Some(event)) = inbound.message().await {
                event_count += 1;
                info!("📦 Traffic event #{} from {}: {:?}", event_count, agent_id_cl, event.request_id);
                
                // 1. Save to DB
                if let Err(e) = db.save_request(&event, &agent_id_cl).await {
                    error!("   ✗ Failed to save to DB: {}", e);
                } else {
                    info!("   ✓ Saved to database");
                }

                // 2. Broadcast event to UI/Subscribers
                if let Err(e) = broadcast.send(event) {
                    warn!("   ⚠️  Failed to broadcast event: {}", e);
                }
            }
            info!("🔌 Agent {} stream ended (processed {} events)", agent_id_cl, event_count);
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type StreamMetricsStream = ReceiverStream<Result<MetricsCommand, Status>>;

    async fn stream_metrics(
        &self,
        request: Request<Streaming<SystemMetricsEvent>>,
    ) -> Result<Response<Self::StreamMetricsStream>, Status> {
        let agent_id = match request.metadata().get("x-agent-id") {
            Some(id) => id.to_str().unwrap_or("unknown").to_string(),
            None => {
                warn!("⚠️  Metrics stream started without x-agent-id metadata");
                "unknown".to_string()
            }
        };

        info!("📊 Agent {} connected for metrics streaming", agent_id);

        let mut inbound = request.into_inner();
        let (_tx, rx) = mpsc::channel(10);
        
        let metrics_broadcast = self.metrics_broadcast_tx.clone();
        let db = self.db.clone();
        let agent_id_cl = agent_id.clone();

        // Spawn task to handle inbound metrics events
        tokio::spawn(async move {
            let mut metrics_count = 0;
            while let Ok(Some(metrics_event)) = inbound.message().await {
                metrics_count += 1;
                debug!("📊 Metrics event #{} from {}: CPU: {:.2}%, Memory: {} MB", 
                       metrics_count, 
                       agent_id_cl,
                       metrics_event.metrics.as_ref().map(|m| m.cpu_usage_percent).unwrap_or(0.0),
                       metrics_event.metrics.as_ref().map(|m| m.memory_used_bytes / 1024 / 1024).unwrap_or(0));
                
                // 1. Save to DB
                if let Err(e) = db.save_system_metrics(&metrics_event).await {
                    error!("   ✗ Failed to save metrics to DB: {}", e);
                } else {
                    debug!("   ✓ Metrics saved to database");
                }

                // 2. Broadcast metrics to UI/Subscribers
                if let Err(e) = metrics_broadcast.send(metrics_event) {
                    warn!("   ⚠️  Failed to broadcast metrics: {}", e);
                }
            }
            info!("📊 Agent {} metrics stream ended (processed {} metrics)", agent_id_cl, metrics_count);
        });

        // For now, return an empty command stream
        // In the future, this could send configuration updates
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
