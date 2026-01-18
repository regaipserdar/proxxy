use crate::pb::proxy_service_server::ProxyService;
use crate::pb::{
    InterceptCommand, MetricsCommand, RegisterAgentRequest, RegisterAgentResponse,
    SystemMetricsEvent, TrafficEvent, traffic_event, HeartbeatRequest, HeartbeatResponse,
};
use crate::AgentRegistry;
use crate::models::settings::InterceptionConfig;
use crate::recording_service::RecordingService;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use std::net::SocketAddr;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info, warn};

use crate::Database;

use proxy_core::CertificateAuthority;

pub struct ProxyServiceImpl {
    agent_registry: Arc<AgentRegistry>,
    broadcast_tx: broadcast::Sender<(String, TrafficEvent)>,
    metrics_broadcast_tx: broadcast::Sender<SystemMetricsEvent>,
    db: Arc<Database>,
    ca: Arc<CertificateAuthority>,
    #[allow(dead_code)] // Reserved for future interception implementation
    interception: Arc<RwLock<InterceptionConfig>>,
    /// Recording service for traffic-based navigation detection
    recording_service: Arc<RecordingService>,
}

impl ProxyServiceImpl {
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        broadcast_tx: broadcast::Sender<(String, TrafficEvent)>,
        metrics_broadcast_tx: broadcast::Sender<SystemMetricsEvent>,
        db: Arc<Database>,
        ca: Arc<CertificateAuthority>,
        interception: Arc<RwLock<InterceptionConfig>>,
        recording_service: Arc<RecordingService>,
    ) -> Self {
        Self {
            agent_registry,
            broadcast_tx,
            metrics_broadcast_tx,
            db,
            ca,
            interception,
            recording_service,
        }
    }

    async fn discover_agent_info(
        &self,
        agent_id: &str,
        remote_addr: SocketAddr,
    ) -> Option<crate::database::AgentInfo> {
        let ip = remote_addr.ip();
        // Try common admin ports (9091 is default)
        let ports = vec![9091];

        for port in ports {
            let url = format!("http://{}:{}/info", ip, port);
            debug!(
                "   🔍 Attempting identity discovery for {} at {}",
                agent_id, url
            );

            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
            {
                Ok(c) => c,
                Err(_) => continue,
            };

            match client.get(&url).send().await {
                Ok(resp) => {
                    if let Ok(info) = resp.json::<serde_json::Value>().await {
                        let name = info["name"].as_str().unwrap_or("Unknown").to_string();
                        let version = info["version"].as_str().unwrap_or("0.1.0").to_string();
                        let hostname = info["hostname"].as_str().unwrap_or("unknown").to_string();

                        info!("   ✅ Discovered agent identity: {} (v{})", name, version);
                        return Some(crate::database::AgentInfo {
                            name,
                            hostname,
                            version,
                        });
                    }
                }
                Err(e) => {
                    debug!("   ✗ Discovery failed at {}: {}", url, e);
                }
            }
        }
        None
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
        if let Err(e) = self
            .db
            .upsert_agent(&agent_id, &req.name, &req.hostname, &req.version)
            .await
        {
            error!("   ✗ Failed to upsert agent to DB: {}", e);
        } else {
            info!("   ✓ Agent saved to database");
        }

        // Read CA cert/key to send back to agent
        let ca_cert_pem = self.ca.get_ca_cert_pem().unwrap_or_default();
        let ca_key_pem = self.ca.get_ca_key_pem().unwrap_or_default();

        info!(
            "   ✓ Sending CA credentials (cert: {} bytes, key: {} bytes)",
            ca_cert_pem.len(),
            ca_key_pem.len()
        );

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
        let remote_addr = request.remote_addr();

        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(100);

        // Register the command channel
        let agent_info = match self.db.get_agent_info(&agent_id).await {
            Ok(Some(info)) => {
                info!("   ✓ Found agent info in DB: {} (v{})", info.name, info.version);
                info
            }
            Ok(None) => {
                warn!("   ⚠️  Agent {} not found in DB, attempting discovery...", agent_id);
                
                let discovered = if let Some(addr) = remote_addr {
                    self.discover_agent_info(&agent_id, addr).await
                } else {
                    None
                };

                match discovered {
                    Some(info) => {
                        // Save discovered info to DB for next time
                        if let Err(e) = self.db.upsert_agent(&agent_id, &info.name, &info.hostname, &info.version).await {
                             warn!("   ✗ Failed to save discovered agent info to DB: {}", e);
                        }
                        info
                    },
                    None => {
                        warn!("   ⚠️  Discovery failed for agent {}, using defaults", agent_id);
                        crate::database::AgentInfo {
                            name: "Unknown".to_string(),
                            hostname: "unknown".to_string(),
                            version: "0.1.0".to_string(), // Default fallback
                        }
                    }
                }
            }
            Err(e) => {
                warn!("   ✗ Failed to fetch agent info: {}", e);
                crate::database::AgentInfo {
                    name: "Unknown".to_string(),
                    hostname: "unknown".to_string(),
                    version: "0.1.0".to_string(), // Default fallback
                }
            }
        };

        self.agent_registry.register_agent(
            agent_id.clone(),
            agent_info.name,
            agent_info.hostname,
            agent_info.version,
            tx,
        );
        info!("   ✓ Agent registered in session manager");

        let broadcast = self.broadcast_tx.clone();
        let db = self.db.clone();
        let agent_id_cl = agent_id.clone();
        let registry = self.agent_registry.clone();
        let recording_svc = self.recording_service.clone();
        
        // Spawn task to handle inbound traffic events
        tokio::spawn(async move {
            let mut event_count = 0;
            while let Ok(Some(event)) = inbound.message().await {
                event_count += 1;
                info!(
                    "📦 Traffic event #{} from {}: {:?}",
                    event_count, agent_id_cl, event.request_id
                );

                // PROXY-BASED NAVIGATION DETECTION - DISABLED
                // Browser-side PerformanceObserver is now the authoritative source
                // Proxy signals were causing noise (googleapis, AJAX, etc.)
                // See: recording_service.rs JavaScript for browser-based navigation detection
                // 
                // if let Some(traffic_event::Event::Request(ref req)) = event.event {
                //     if recording_svc.is_recording().await {
                //         if req.method.to_uppercase() == "GET" {
                //             let accept_header = req.headers.as_ref()
                //                 .and_then(|h| h.headers.get("accept").or_else(|| h.headers.get("Accept")))
                //                 .map(|s| s.as_str());
                //             recording_svc.add_navigation_event(&req.url, accept_header, 200).await;
                //         }
                //     }
                // }

                // SCOPE CHECK - Determine if we should record and broadcast this event
                let rules = db.scope_rules_cache.read().await;
                let should_record = if !rules.is_empty() {
                    // Extract URL from event
                    let url = match &event.event {
                        Some(traffic_event::Event::Request(req)) => Some(req.url.as_str()),
                        _ => None,
                    };

                    if let Some(url) = url {
                        let in_scope = crate::scope::is_in_scope(&rules, url);
                        if !in_scope {
                            debug!("⏭️ Out-of-scope (ignoring): {}", url);
                        }
                        in_scope
                    } else {
                        true // No URL = record by default
                    }
                } else {
                    true // No rules = record everything
                };
                drop(rules); // Release lock

                // Only broadcast and save if in scope
                if should_record {
                    // 1. Broadcast event to UI/Subscribers (fast path)
                    if let Err(e) = broadcast.send((agent_id_cl.clone(), event.clone())) {
                        warn!("   ⚠️  Failed to broadcast event: {}", e);
                    }

                    // 2. Save to DB asynchronously (background task)
                    let db_bg = db.clone();
                    let event_bg = event.clone();
                    let agent_id_bg = agent_id_cl.clone();
                    let registry_bg = registry.clone();
                    
                    tokio::spawn(async move {
                        // Retry loop for handling potential FK constraints (e.g. if DB was swapped)
                        let mut retry_count = 0;
                        const MAX_RETRIES: u32 = 1;
                        
                        loop {
                            match db_bg.save_request(&event_bg, &agent_id_bg).await {
                                Ok(_) => break, // Success
                                Err(e) => {
                                    // Check if it's a foreign key constraint failure (code 787)
                                    let is_fk_error = e.to_string().contains("FOREIGN KEY constraint failed") || 
                                                     e.to_string().contains("code: 787");
                                    
                                    if is_fk_error && retry_count < MAX_RETRIES {
                                        retry_count += 1;
                                        warn!("   ⚠️  Foreign key violation saving traffic for agent {}. Attempting to restore agent record...", agent_id_bg);
                                        
                                        // Attempt to restore agent record from registry
                                        if let Some(agent_info) = registry_bg.get_agent(&agent_id_bg) {
                                            if let Err(upsert_err) = db_bg.upsert_agent(
                                                &agent_info.id, 
                                                &agent_info.name, 
                                                &agent_info.hostname, 
                                                &agent_info.version
                                            ).await {
                                                error!("   ✗ Failed to restore agent record: {}", upsert_err);
                                                break; 
                                            } else {
                                                info!("   ✓ Agent record restored to DB. Retrying save...");
                                                continue;
                                            }
                                        } else {
                                            warn!("   ⚠️  Agent {} not found in registry, cannot restore.", agent_id_bg);
                                            break;
                                        }
                                    } else {
                                        error!("   ✗ Failed to save to DB: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
                // Note: Proxy continues to forward the request regardless of scope
            }
            info!(
                "🔌 Agent {} stream ended (processed {} events)",
                agent_id_cl, event_count
            );

            // Mark agent as offline in database
            if let Err(e) = db.mark_agent_offline(&agent_id_cl).await {
                error!(
                    "   ✗ Failed to mark agent {} as offline: {}",
                    agent_id_cl, e
                );
            } else {
                info!("   ✓ Agent {} marked as offline in database", agent_id_cl);
            }

            // Remove agent from registry
            registry.remove_agent(&agent_id_cl);
            info!("   ✓ Agent {} removed from session registry", agent_id_cl);
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
        let registry = self.agent_registry.clone();

        // Spawn task to handle inbound metrics events
        tokio::spawn(async move {
            let mut metrics_count = 0;
            while let Ok(Some(metrics_event)) = inbound.message().await {
                metrics_count += 1;
                debug!(
                    "📊 Metrics event #{} from {}: CPU: {:.2}%, Memory: {} MB",
                    metrics_count,
                    agent_id_cl,
                    metrics_event
                        .metrics
                        .as_ref()
                        .map(|m| m.cpu_usage_percent)
                        .unwrap_or(0.0),
                    metrics_event
                        .metrics
                        .as_ref()
                        .map(|m| m.memory_used_bytes / 1024 / 1024)
                        .unwrap_or(0)
                );

                // 1. Save to DB with Retry Logic
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 1;

                loop {
                    match db.save_system_metrics(&metrics_event).await {
                        Ok(_) => {
                            debug!("   ✓ Metrics saved to database");
                            break;
                        }
                        Err(e) => {
                            let is_fk_error = e.to_string().contains("FOREIGN KEY constraint failed") || 
                                             e.to_string().contains("code: 787");

                            if is_fk_error && retry_count < MAX_RETRIES {
                                retry_count += 1;
                                warn!("   ⚠️  Foreign key violation saving metrics for agent {}. Attempting to restore agent record...", agent_id_cl);
                                
                                if let Some(agent_info) = registry.get_agent(&agent_id_cl) {
                                    if let Err(upsert_err) = db.upsert_agent(
                                        &agent_info.id, 
                                        &agent_info.name, 
                                        &agent_info.hostname, 
                                        &agent_info.version
                                    ).await {
                                        error!("   ✗ Failed to restore agent record: {}", upsert_err);
                                        break;
                                    } else {
                                        info!("   ✓ Agent record restored to DB. Retrying save...");
                                        continue;
                                    }
                                } else {
                                    warn!("   ⚠️  Agent {} not found in registry, cannot restore.", agent_id_cl);
                                    break;
                                }
                            } else {
                                error!("   ✗ Failed to save metrics to DB: {}", e);
                                break;
                            }
                        }
                    }
                }

                // 2. Broadcast metrics to UI/Subscribers
                if let Err(e) = metrics_broadcast.send(metrics_event) {
                    warn!("   ⚠️  Failed to broadcast metrics: {}", e);
                }
            }
            info!(
                "📊 Agent {} metrics stream ended (processed {} metrics)",
                agent_id_cl, metrics_count
            );

            // Mark agent as offline in database (if not already marked by traffic stream)
            if let Err(e) = db.mark_agent_offline(&agent_id_cl).await {
                // This might fail if already marked offline by traffic stream, which is fine
                debug!(
                    "   ℹ️  Could not mark agent {} as offline: {}",
                    agent_id_cl, e
                );
            } else {
                info!(
                    "   ✓ Agent {} marked as offline (metrics stream ended)",
                    agent_id_cl
                );
            }

            // Remove agent from registry (if not already removed)
            registry.remove_agent(&agent_id_cl);
        });

        // For now, return an empty command stream
        // In the future, this could send configuration updates
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type HeartbeatStream = ReceiverStream<Result<HeartbeatResponse, Status>>;

    async fn heartbeat(
        &self,
        request: Request<Streaming<HeartbeatRequest>>,
    ) -> Result<Response<Self::HeartbeatStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(10);
        let registry = self.agent_registry.clone();

        tokio::spawn(async move {
            while let Ok(Some(req)) = inbound.message().await {
                registry.update_heartbeat(
                    &req.agent_id,
                    req.cpu_usage,
                    req.memory_usage_mb,
                    req.uptime_seconds,
                    req.public_ip
                );

                let resp = HeartbeatResponse {
                    success: true,
                    timestamp: chrono::Utc::now().timestamp(),
                };

                if let Err(_) = tx.send(Ok(resp)).await {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
