use proxy_core::pb::proxy_service_client::ProxyServiceClient;
use proxy_core::pb::{TrafficEvent, RegisterAgentRequest, SystemMetricsEvent, MetricsCommand};
use proxy_core::{SystemMetricsCollector, SystemMetricsCollectorConfig};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use std::time::Duration;

pub struct OrchestratorClient {
    endpoint: String,
    pub agent_id: String,
    name: String,
}

impl OrchestratorClient {
    pub fn new(endpoint: String, agent_id: String, name: String) -> Self {
        Self { endpoint, agent_id, name }
    }

    pub async fn register(&self) -> Result<(String, String), String> {
        let mut client = ProxyServiceClient::connect(self.endpoint.clone())
            .await
            .map_err(|e| e.to_string())?;
        
        let req = tonic::Request::new(RegisterAgentRequest {
            agent_id: self.agent_id.clone(),
            hostname: hostname::get().unwrap_or_default().to_string_lossy().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            name: self.name.clone(),
        });

        let resp = client.register_agent(req)
            .await
            .map_err(|e| e.to_string())?;
            
        let inner = resp.into_inner();
        
        if inner.success {
            info!("Successfully registered agent: {}", self.agent_id);
            info!("Received CA - cert length: {}, key length: {}", 
                inner.ca_cert_pem.len(), inner.ca_key_pem.len());
            info!("CA key preview: {}", &inner.ca_key_pem.chars().take(50).collect::<String>());
            Ok((inner.ca_cert_pem, inner.ca_key_pem))
        } else {
            Err(format!("Registration rejected: {}", inner.message))
        }
    }

    pub async fn run(&self, mut rx: mpsc::Receiver<TrafficEvent>) {
        let max_delay = Duration::from_secs(60);
        let mut attempt = 0;

        // Shared HTTP client for replaying requests
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        loop {
            // 1. Registration Loop
            loop {
                info!("Attempting to register with Orchestrator at {}...", self.endpoint);
                match self.register().await {
                    Ok(_) => {
                        attempt = 0; // Reset backoff on success
                        break;
                    }
                    Err(e) => {
                        let delay = Duration::from_secs(2u64.pow(attempt.min(6))).min(max_delay);
                        warn!("Registration failed: {}. Retrying in {:?}...", e, delay);
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                    }
                }
            }

            // 2. Start metrics streaming alongside traffic streaming
            let metrics_handle = self.start_metrics_streaming().await;

            // 3. Traffic Streaming Loop
            info!("Starting traffic stream...");
            match ProxyServiceClient::connect(self.endpoint.clone()).await {
                Ok(mut client) => {
                    let (tx_stream, rx_stream) = mpsc::channel(1024);
                    let outbound = ReceiverStream::new(rx_stream);
                    
                    // Add agent-id metadata
                     let mut req = tonic::Request::new(outbound);
                     req.metadata_mut().insert("x-agent-id", self.agent_id.parse().unwrap()); // assuming uuid is ascii safe

                    match client.stream_traffic(req).await {
                        Ok(response) => {
                            let mut inbound = response.into_inner();
                            info!("Traffic stream established");

                            let tx_replay = tx_stream.clone();
                            let http_client = http_client.clone();
                            
                            // Spawn response handler (commands)
                            let stream_handle = tokio::spawn(async move {
                                use proxy_core::pb::{intercept_command, traffic_event, HttpResponseData, HttpHeaders};
                                
                                while let Ok(Some(cmd)) = inbound.message().await {
                                    if let Some(intercept_command::Command::Execute(exec_req)) = cmd.command {
                                        let req_data = exec_req.request.clone().unwrap_or_default();
                                        info!("Executing Replay Request: {} {}", req_data.method, req_data.url);
                                        
                                        let client = http_client.clone();
                                        let tx = tx_replay.clone();
                                        let req_id = exec_req.request_id.clone();
                                        
                                        tokio::spawn(async move {
                                            // Construct Request
                                            let mut builder = client.request(
                                                reqwest::Method::from_bytes(req_data.method.as_bytes()).unwrap_or(reqwest::Method::GET),
                                                &req_data.url
                                            );
                                            
                                            if let Some(h) = req_data.headers {
                                                for (k, v) in h.headers {
                                                    builder = builder.header(k, v);
                                                }
                                            }
                                            
                                            if !req_data.body.is_empty() {
                                                builder = builder.body(req_data.body);
                                            }
                                            
                                            // Execute
                                            let result_event = match builder.send().await {
                                                Ok(resp) => {
                                                    // Convert headers
                                                    let mut headers_map = std::collections::HashMap::new();
                                                    for (k, v) in resp.headers() {
                                                        headers_map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
                                                    }
                                                    
                                                    let status = resp.status().as_u16() as i32;
                                                    let body = resp.bytes().await.unwrap_or_default().to_vec();
                                                    
                                                    TrafficEvent {
                                                        request_id: req_id,
                                                        event: Some(traffic_event::Event::Response(HttpResponseData {
                                                            status_code: status,
                                                            headers: Some(HttpHeaders { headers: headers_map }),
                                                            body,
                                                            tls: None, 
                                                        }))
                                                    }
                                                },
                                                Err(e) => {
                                                    error!("Replay failed: {}", e);
                                                    // Return 502/Error?
                                                      TrafficEvent {
                                                        request_id: req_id,
                                                        event: Some(traffic_event::Event::Response(HttpResponseData {
                                                            status_code: 502,
                                                            headers: None,
                                                            body: format!("Replay Error: {}", e).into_bytes(),
                                                            tls: None, 
                                                        }))
                                                    }
                                                }
                                            };
                                            
                                            if let Err(e) = tx.send(result_event).await {
                                                warn!("Failed to send replay result: {}", e);
                                            }
                                        });
                                    }
                                }
                                info!("Stream closed by server");
                            });

                            // Forwarding loop
                            loop {
                                if stream_handle.is_finished() {
                                    warn!("Stream handler finished, reconnecting...");
                                    break;
                                }

                                match rx.recv().await {
                                    Some(event) => {
                                        if let Err(_) = tx_stream.send(event).await {
                                            warn!("Failed to send event, stream probably closed. Reconnecting...");
                                            break;
                                        }
                                    }
                                    None => {
                                        info!("Log channel closed, shutting down agent client.");
                                        // Cancel metrics streaming
                                        if let Some(handle) = metrics_handle {
                                            handle.abort();
                                        }
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to start stream: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect for streaming: {}. Retrying...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            
            // Cancel metrics streaming before reconnecting
            if let Some(handle) = metrics_handle {
                handle.abort();
            }
        }
    }

    /// Start system metrics streaming in a separate task
    async fn start_metrics_streaming(&self) -> Option<tokio::task::JoinHandle<()>> {
        info!("Starting system metrics streaming for agent: {}", self.agent_id);
        
        match ProxyServiceClient::connect(self.endpoint.clone()).await {
            Ok(mut client) => {
                let agent_id = self.agent_id.clone();
                let endpoint = self.endpoint.clone();
                
                let handle = tokio::spawn(async move {
                    let mut metrics_collector = SystemMetricsCollector::with_config(
                        agent_id.clone(),
                        SystemMetricsCollectorConfig::default()
                    );
                    
                    let max_delay = Duration::from_secs(60);
                    let mut attempt = 0;
                    
                    loop {
                        // Create channels for metrics streaming
                        let (metrics_tx, metrics_rx) = mpsc::channel::<SystemMetricsEvent>(100);
                        let (command_tx, command_rx) = mpsc::channel::<MetricsCommand>(10);
                        
                        let outbound = ReceiverStream::new(metrics_rx);
                        let mut req = tonic::Request::new(outbound);
                        req.metadata_mut().insert("x-agent-id", agent_id.parse().unwrap());
                        
                        match client.stream_metrics(req).await {
                            Ok(response) => {
                                info!("Metrics stream established for agent: {}", agent_id);
                                let mut inbound = response.into_inner();
                                
                                // Spawn command handler
                                let command_tx_clone = command_tx.clone();
                                let command_handle = tokio::spawn(async move {
                                    while let Ok(Some(cmd)) = inbound.message().await {
                                        if let Err(e) = command_tx_clone.send(cmd).await {
                                            warn!("Failed to forward metrics command: {}", e);
                                            break;
                                        }
                                    }
                                    info!("Metrics command stream closed");
                                });
                                
                                // Start metrics collection and streaming
                                let streaming_result = metrics_collector.start_streaming(
                                    metrics_tx,
                                    command_rx
                                ).await;
                                
                                command_handle.abort();
                                
                                if let Err(e) = streaming_result {
                                    error!("Metrics streaming failed: {}", e);
                                    let delay = Duration::from_secs(2u64.pow(attempt.min(6))).min(max_delay);
                                    warn!("Retrying metrics streaming in {:?}...", delay);
                                    tokio::time::sleep(delay).await;
                                    attempt += 1;
                                } else {
                                    attempt = 0; // Reset backoff on success
                                }
                            }
                            Err(e) => {
                                error!("Failed to establish metrics stream: {}", e);
                                let delay = Duration::from_secs(2u64.pow(attempt.min(6))).min(max_delay);
                                warn!("Retrying metrics streaming in {:?}...", delay);
                                tokio::time::sleep(delay).await;
                                attempt += 1;
                            }
                        }
                        
                        // Reconnect to client if needed
                        if let Err(e) = ProxyServiceClient::connect(endpoint.clone()).await {
                            error!("Failed to reconnect metrics client: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        } else {
                            client = ProxyServiceClient::connect(endpoint.clone()).await.unwrap();
                        }
                    }
                });
                
                Some(handle)
            }
            Err(e) => {
                error!("Failed to connect for metrics streaming: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proxy_core::pb::proxy_service_server::ProxyService;
    use proxy_core::pb::{RegisterAgentResponse, InterceptCommand, intercept_command, ExecuteRequest, HttpRequestData};
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::{Request, Response, Status, Streaming};
    use axum::{Router, routing::get};
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_repeater_flow() {
        // 1. Start Mock Target Server (HTTP)
        let target_router = Router::new().route("/replay", get(|| async { "Hello Replay" }));
        let target_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let target_listener = tokio::net::TcpListener::bind(target_addr).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(target_listener, target_router).await.unwrap();
        });

        // 2. Start Mock Orchestrator (gRPC)
        let (cmd_tx, cmd_rx) = mpsc::channel::<Result<InterceptCommand, Status>>(1);
        let (event_tx, mut event_rx) = mpsc::channel::<TrafficEvent>(1);
        
        let orchestrator_addr = "[::1]:50099".parse::<SocketAddr>().unwrap();
        let service = proxy_core::pb::proxy_service_server::ProxyServiceServer::new(TestProxy {
            cmd_rx: std::sync::Arc::new(tokio::sync::Mutex::new(Some(cmd_rx))),
            event_tx,
        });
        
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(service)
                .serve(orchestrator_addr)
                .await
                .unwrap();
        });
        
        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for startup

        // 3. Start Agent
        let (_log_tx, log_rx) = mpsc::channel(1); // Fake log channel input
        let client = OrchestratorClient::new("http://[::1]:50099".to_string(), "test-agent".to_string());
        
        tokio::spawn(async move {
            client.run(log_rx).await;
        });

        // 4. Send Execute Command
        let target_url = format!("http://127.0.0.1:{}/replay", target_port);
        let execute_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Execute(ExecuteRequest {
                request_id: "test-replay-1".to_string(),
                request: Some(HttpRequestData {
                    method: "GET".to_string(),
                    url: target_url,
                    headers: None,
                    body: vec![],
                    tls: None,
                }),
            }))
        };
        
        cmd_tx.send(Ok(execute_cmd)).await.unwrap();
        
        // 5. Verify Response
        let event = event_rx.recv().await.expect("Should receive traffic event");
        assert_eq!(event.request_id, "test-replay-1");
         if let Some(proxy_core::pb::traffic_event::Event::Response(resp)) = event.event {
            assert_eq!(resp.status_code, 200);
            assert_eq!(resp.body, b"Hello Replay");
        } else {
            panic!("Expected Response event");
        }
    }
    
    struct TestProxy {
        cmd_rx: std::sync::Arc<tokio::sync::Mutex<Option<mpsc::Receiver<Result<InterceptCommand, Status>>>>>,
        event_tx: mpsc::Sender<TrafficEvent>,
    }
    
    #[tonic::async_trait]
    impl ProxyService for TestProxy {
        async fn register_agent(&self, _req: Request<RegisterAgentRequest>) -> Result<Response<RegisterAgentResponse>, Status> {
             Ok(Response::new(RegisterAgentResponse { success: true, message: "".into() }))
        }
        
        type StreamTrafficStream = ReceiverStream<Result<InterceptCommand, Status>>;
        
        async fn stream_traffic(&self, req: Request<Streaming<TrafficEvent>>) -> Result<Response<Self::StreamTrafficStream>, Status> {
            let mut inbound = req.into_inner();
            let tx = self.event_tx.clone();
            tokio::spawn(async move {
                while let Ok(Some(msg)) = inbound.message().await {
                    tx.send(msg).await.ok();
                }
            });
            
            // Take the receiver from the mutex and return a stream
            // This only works for the first connection, which is fine for this test.
            let mut guard = self.cmd_rx.lock().await;
            if let Some(rx) = guard.take() {
                 Ok(Response::new(ReceiverStream::new(rx)))
            } else {
                Err(Status::aborted("Only one test connection allowed"))
            }
        }
    }
}
