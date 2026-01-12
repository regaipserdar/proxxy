use proxy_core::pb::proxy_service_client::ProxyServiceClient;
use proxy_core::pb::{MetricsCommand, RegisterAgentRequest, SystemMetricsEvent, TrafficEvent};
use proxy_core::{SystemMetricsCollector, SystemMetricsCollectorConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info, warn};

/// Tracks active attack requests for graceful shutdown
#[derive(Debug, Clone)]
struct AttackTracker {
    active_requests: Arc<Mutex<std::collections::HashSet<String>>>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl AttackTracker {
    fn new() -> Self {
        Self {
            active_requests: Arc::new(Mutex::new(std::collections::HashSet::new())),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        }
    }

    async fn add_request(&self, request_id: String) {
        let mut requests = self.active_requests.lock().await;
        requests.insert(request_id);
    }

    async fn remove_request(&self, request_id: &str) {
        let mut requests = self.active_requests.lock().await;
        requests.remove(request_id);
        
        // If no more active requests and shutdown was signaled, notify completion
        if requests.is_empty() {
            self.shutdown_signal.notify_waiters();
        }
    }

    async fn wait_for_completion(&self) {
        loop {
            {
                let requests = self.active_requests.lock().await;
                if requests.is_empty() {
                    break;
                }
            }
            
            // Wait for notification or timeout
            tokio::select! {
                _ = self.shutdown_signal.notified() => {
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Continue checking
                }
            }
        }
    }

    async fn get_active_count(&self) -> usize {
        let requests = self.active_requests.lock().await;
        requests.len()
    }
}

pub struct OrchestratorClient {
    endpoint: String,
    pub agent_id: String,
    name: String,
    attack_tracker: AttackTracker,
}

impl OrchestratorClient {
    pub fn new(endpoint: String, agent_id: String, name: String) -> Self {
        Self {
            endpoint,
            agent_id,
            name,
            attack_tracker: AttackTracker::new(),
        }
    }

    /// Unified HTTP request execution with session data injection
    async fn execute_http_request(
        client: &reqwest::Client,
        mut req_data: proxy_core::pb::HttpRequestData,
        req_id: String,
        session_id: Option<String>,
        session_headers: Option<std::collections::HashMap<String, String>>,
        attack_tracker: Option<AttackTracker>,
    ) -> proxy_core::pb::TrafficEvent {
        use proxy_core::pb::{traffic_event, HttpHeaders, HttpResponseData};

        // Track this request if it's an attack request
        if let Some(ref tracker) = attack_tracker {
            tracker.add_request(req_id.clone()).await;
        }

        // Inject session data if provided
        if let Some(session_headers) = session_headers {
            if req_data.headers.is_none() {
                req_data.headers = Some(HttpHeaders {
                    headers: std::collections::HashMap::new(),
                });
            }
            
            if let Some(ref mut headers) = req_data.headers {
                for (key, value) in session_headers {
                    headers.headers.insert(key, value);
                }
            }
        }

        // Log session information if present
        if let Some(session_id) = session_id {
            info!("Executing request with session: {}", session_id);
        }

        // Construct Request
        let mut builder = client.request(
            reqwest::Method::from_bytes(req_data.method.as_bytes())
                .unwrap_or(reqwest::Method::GET),
            &req_data.url,
        );

        if let Some(h) = req_data.headers {
            for (k, v) in h.headers {
                builder = builder.header(k, v);
            }
        }

        if !req_data.body.is_empty() {
            builder = builder.body(req_data.body);
        }

        // Execute request with error handling
        let result = match builder.send().await {
            Ok(resp) => {
                // Convert headers
                let mut headers_map = std::collections::HashMap::new();
                for (k, v) in resp.headers() {
                    headers_map.insert(
                        k.to_string(),
                        v.to_str().unwrap_or("").to_string(),
                    );
                }

                let status = resp.status().as_u16() as i32;
                let body = resp.bytes().await.unwrap_or_default().to_vec();

                proxy_core::pb::TrafficEvent {
                    request_id: req_id.clone(),
                    event: Some(traffic_event::Event::Response(HttpResponseData {
                        status_code: status,
                        headers: Some(HttpHeaders {
                            headers: headers_map,
                        }),
                        body,
                        tls: None,
                    })),
                }
            }
            Err(e) => {
                error!("HTTP request failed: {}", e);
                proxy_core::pb::TrafficEvent {
                    request_id: req_id.clone(),
                    event: Some(traffic_event::Event::Response(HttpResponseData {
                        status_code: 502,
                        headers: None,
                        body: format!("Request Error: {}", e).into_bytes(),
                        tls: None,
                    })),
                }
            }
        };

        // Remove from tracking
        if let Some(ref tracker) = attack_tracker {
            tracker.remove_request(&req_id).await;
        }

        result
    }

    pub async fn register(&self) -> Result<(String, String), String> {
        let mut client = ProxyServiceClient::connect(self.endpoint.clone())
            .await
            .map_err(|e| e.to_string())?;

        let req = tonic::Request::new(RegisterAgentRequest {
            agent_id: self.agent_id.clone(),
            hostname: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            name: self.name.clone(),
        });

        let resp = client
            .register_agent(req)
            .await
            .map_err(|e| e.to_string())?;

        let inner = resp.into_inner();

        if inner.success {
            info!("Successfully registered agent: {}", self.agent_id);
            info!(
                "Received CA - cert length: {}, key length: {}",
                inner.ca_cert_pem.len(),
                inner.ca_key_pem.len()
            );
            info!(
                "CA key preview: {}",
                &inner.ca_key_pem.chars().take(50).collect::<String>()
            );
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
                info!(
                    "Attempting to register with Orchestrator at {}...",
                    self.endpoint
                );
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
                    req.metadata_mut()
                        .insert("x-agent-id", self.agent_id.parse().unwrap()); // assuming uuid is ascii safe

                    match client.stream_traffic(req).await {
                        Ok(response) => {
                            let mut inbound = response.into_inner();
                            info!("Traffic stream established");

                            let tx_replay = tx_stream.clone();
                            let http_client = http_client.clone();
                            let attack_tracker = self.attack_tracker.clone();

                            // Spawn response handler (commands)
                            let stream_handle = tokio::spawn(async move {
                                use proxy_core::pb::{
                                    intercept_command, traffic_event, HttpHeaders, HttpResponseData,
                                    attack_command, AttackCommand, RepeaterRequest, IntruderRequest,
                                };

                                while let Ok(Some(cmd)) = inbound.message().await {
                                    match cmd.command {
                                        Some(intercept_command::Command::Execute(exec_req)) => {
                                            let req_data = exec_req.request.clone().unwrap_or_default();
                                            info!(
                                                "Executing Replay Request: {} {}",
                                                req_data.method, req_data.url
                                            );

                                            let client = http_client.clone();
                                            let tx = tx_replay.clone();
                                            let req_id = exec_req.request_id.clone();

                                            tokio::spawn(async move {
                                                let result_event = Self::execute_http_request(
                                                    &client, req_data, req_id, None, None, None
                                                ).await;

                                                if let Err(e) = tx.send(result_event).await {
                                                    warn!("Failed to send replay result: {}", e);
                                                }
                                            });
                                        }
                                        Some(intercept_command::Command::Attack(attack_cmd)) => {
                                            match attack_cmd.command {
                                                Some(attack_command::Command::RepeaterRequest(repeater_req)) => {
                                                    let req_data = repeater_req.request.clone().unwrap_or_default();
                                                    info!(
                                                        "Executing Repeater Request: {} {}",
                                                        req_data.method, req_data.url
                                                    );

                                                    let client = http_client.clone();
                                                    let tx = tx_replay.clone();
                                                    let req_id = repeater_req.request_id.clone();
                                                    let session_id = if repeater_req.session_id.is_empty() { 
                                                        None 
                                                    } else { 
                                                        Some(repeater_req.session_id.clone()) 
                                                    };
                                                    let session_headers = if repeater_req.session_headers.is_empty() {
                                                        None
                                                    } else {
                                                        Some(repeater_req.session_headers.clone())
                                                    };
                                                    let tracker = Some(attack_tracker.clone());

                                                    tokio::spawn(async move {
                                                        let result_event = Self::execute_http_request(
                                                            &client, req_data, req_id, session_id, session_headers, tracker
                                                        ).await;

                                                        if let Err(e) = tx.send(result_event).await {
                                                            warn!("Failed to send repeater result: {}", e);
                                                        }
                                                    });
                                                }
                                                Some(attack_command::Command::IntruderRequest(intruder_req)) => {
                                                    let req_data = intruder_req.request.clone().unwrap_or_default();
                                                    info!(
                                                        "Executing Intruder Request: {} {} (payloads: {:?})",
                                                        req_data.method, req_data.url, intruder_req.payload_values
                                                    );

                                                    let client = http_client.clone();
                                                    let tx = tx_replay.clone();
                                                    let req_id = intruder_req.request_id.clone();
                                                    let session_id = if intruder_req.session_id.is_empty() { 
                                                        None 
                                                    } else { 
                                                        Some(intruder_req.session_id.clone()) 
                                                    };
                                                    let session_headers = if intruder_req.session_headers.is_empty() {
                                                        None
                                                    } else {
                                                        Some(intruder_req.session_headers.clone())
                                                    };
                                                    let tracker = Some(attack_tracker.clone());

                                                    tokio::spawn(async move {
                                                        let result_event = Self::execute_http_request(
                                                            &client, req_data, req_id, session_id, session_headers, tracker
                                                        ).await;

                                                        if let Err(e) = tx.send(result_event).await {
                                                            warn!("Failed to send intruder result: {}", e);
                                                        }
                                                    });
                                                }
                                                Some(attack_command::Command::StopAttack(_)) => {
                                                    info!("Received stop attack command");
                                                    // Graceful attack termination - wait for active requests to complete
                                                    let active_count = attack_tracker.get_active_count().await;
                                                    if active_count > 0 {
                                                        info!("Waiting for {} active attack requests to complete", active_count);
                                                        tokio::time::timeout(
                                                            Duration::from_secs(30),
                                                            attack_tracker.wait_for_completion()
                                                        ).await.unwrap_or_else(|_| {
                                                            warn!("Timeout waiting for attack requests to complete");
                                                        });
                                                    }
                                                    info!("Attack termination complete");
                                                }
                                                None => {
                                                    warn!("Received empty attack command");
                                                }
                                            }
                                        }
                                        Some(intercept_command::Command::Lifecycle(lifecycle_cmd)) => {
                                            use proxy_core::pb::lifecycle_command::Action;
                                            match lifecycle_cmd.action() {
                                                Action::Restart => {
                                                    info!("Received restart command (force: {})", lifecycle_cmd.force);
                                                    
                                                    if !lifecycle_cmd.force {
                                                        // Graceful restart - wait for active attacks to complete
                                                        let active_count = attack_tracker.get_active_count().await;
                                                        if active_count > 0 {
                                                            info!("Waiting for {} active requests before restart", active_count);
                                                            tokio::time::timeout(
                                                                Duration::from_secs(60),
                                                                attack_tracker.wait_for_completion()
                                                            ).await.unwrap_or_else(|_| {
                                                                warn!("Timeout waiting for requests to complete before restart");
                                                            });
                                                        }
                                                    }
                                                    
                                                    info!("Initiating agent restart...");
                                                    // TODO: Implement actual restart mechanism
                                                    // For now, we break the stream to trigger reconnection
                                                    break;
                                                }
                                                Action::Shutdown => {
                                                    info!("Received shutdown command (force: {})", lifecycle_cmd.force);
                                                    
                                                    if !lifecycle_cmd.force {
                                                        // Graceful shutdown - wait for active attacks to complete
                                                        let active_count = attack_tracker.get_active_count().await;
                                                        if active_count > 0 {
                                                            info!("Waiting for {} active requests before shutdown", active_count);
                                                            tokio::time::timeout(
                                                                Duration::from_secs(60),
                                                                attack_tracker.wait_for_completion()
                                                            ).await.unwrap_or_else(|_| {
                                                                warn!("Timeout waiting for requests to complete before shutdown");
                                                            });
                                                        }
                                                    }
                                                    
                                                    info!("Initiating agent shutdown...");
                                                    // TODO: Implement actual shutdown mechanism
                                                    // For now, we break the stream
                                                    break;
                                                }
                                            }
                                        }
                                        _ => {
                                            warn!("Received unknown command type");
                                        }
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
        info!(
            "Starting system metrics streaming for agent: {}",
            self.agent_id
        );

        match ProxyServiceClient::connect(self.endpoint.clone()).await {
            Ok(mut client) => {
                let agent_id = self.agent_id.clone();
                let endpoint = self.endpoint.clone();

                let handle = tokio::spawn(async move {
                    let mut metrics_collector = SystemMetricsCollector::with_config(
                        agent_id.clone(),
                        SystemMetricsCollectorConfig::default(),
                    );

                    let max_delay = Duration::from_secs(60);
                    let mut attempt = 0;

                    loop {
                        // Create channels for metrics streaming
                        let (metrics_tx, metrics_rx) = mpsc::channel::<SystemMetricsEvent>(100);
                        let (command_tx, command_rx) = mpsc::channel::<MetricsCommand>(10);

                        let outbound = ReceiverStream::new(metrics_rx);
                        let mut req = tonic::Request::new(outbound);
                        req.metadata_mut()
                            .insert("x-agent-id", agent_id.parse().unwrap());

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
                                let streaming_result = metrics_collector
                                    .start_streaming(metrics_tx, command_rx)
                                    .await;

                                command_handle.abort();

                                if let Err(e) = streaming_result {
                                    error!("Metrics streaming failed: {}", e);
                                    let delay = Duration::from_secs(2u64.pow(attempt.min(6)))
                                        .min(max_delay);
                                    warn!("Retrying metrics streaming in {:?}...", delay);
                                    tokio::time::sleep(delay).await;
                                    attempt += 1;
                                } else {
                                    attempt = 0; // Reset backoff on success
                                }
                            }
                            Err(e) => {
                                error!("Failed to establish metrics stream: {}", e);
                                let delay =
                                    Duration::from_secs(2u64.pow(attempt.min(6))).min(max_delay);
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
    use axum::{routing::get, Router};
    use proxy_core::pb::proxy_service_server::ProxyService;
    use proxy_core::pb::{
        intercept_command, ExecuteRequest, HttpRequestData, InterceptCommand, RegisterAgentResponse,
    };
    use std::net::SocketAddr;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::{Request, Response, Status, Streaming};

    #[tokio::test]
    async fn test_attack_commands_flow() {
        // 1. Start Mock Target Server (HTTP)
        let target_router = Router::new()
            .route("/repeater", get(|| async { "Repeater Response" }))
            .route("/intruder", get(|| async { "Intruder Response" }));
        let target_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let target_listener = tokio::net::TcpListener::bind(target_addr).await.unwrap();
        let target_port = target_listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(target_listener, target_router).await.unwrap();
        });

        // 2. Start Mock Orchestrator (gRPC)
        let (cmd_tx, cmd_rx) = mpsc::channel::<Result<InterceptCommand, Status>>(10);
        let (event_tx, mut event_rx) = mpsc::channel::<TrafficEvent>(10);

        let orchestrator_addr = "[::1]:50098".parse::<SocketAddr>().unwrap();
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
        let client = OrchestratorClient::new(
            "http://[::1]:50098".to_string(),
            "test-agent-attack".to_string(),
            "test-attack-name".to_string(),
        );

        tokio::spawn(async move {
            client.run(log_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for agent connection

        // 4. Test Repeater Request
        let repeater_url = format!("http://127.0.0.1:{}/repeater", target_port);
        let repeater_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Attack(
                proxy_core::pb::AttackCommand {
                    command: Some(proxy_core::pb::attack_command::Command::RepeaterRequest(
                        proxy_core::pb::RepeaterRequest {
                            request_id: "test-repeater-1".to_string(),
                            request: Some(HttpRequestData {
                                method: "GET".to_string(),
                                url: repeater_url,
                                headers: None,
                                body: vec![],
                                tls: None,
                            }),
                            session_id: "test-session".to_string(),
                            session_headers: {
                                let mut headers = std::collections::HashMap::new();
                                headers.insert("X-Session-Token".to_string(), "test-token".to_string());
                                headers
                            },
                        }
                    ))
                }
            )),
        };

        cmd_tx.send(Ok(repeater_cmd)).await.unwrap();

        // 5. Verify Repeater Response
        let event = event_rx.recv().await.expect("Should receive repeater response");
        assert_eq!(event.request_id, "test-repeater-1");
        if let Some(proxy_core::pb::traffic_event::Event::Response(resp)) = event.event {
            assert_eq!(resp.status_code, 200);
            assert_eq!(resp.body, b"Repeater Response");
        } else {
            panic!("Expected Response event for repeater");
        }

        // 6. Test Intruder Request
        let intruder_url = format!("http://127.0.0.1:{}/intruder", target_port);
        let intruder_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Attack(
                proxy_core::pb::AttackCommand {
                    command: Some(proxy_core::pb::attack_command::Command::IntruderRequest(
                        proxy_core::pb::IntruderRequest {
                            attack_id: "test-attack-1".to_string(),
                            request_id: "test-intruder-1".to_string(),
                            request: Some(HttpRequestData {
                                method: "GET".to_string(),
                                url: intruder_url,
                                headers: None,
                                body: vec![],
                                tls: None,
                            }),
                            payload_values: vec!["payload1".to_string(), "payload2".to_string()],
                            session_id: "test-session".to_string(),
                            session_headers: {
                                let mut headers = std::collections::HashMap::new();
                                headers.insert("X-Attack-Token".to_string(), "attack-token".to_string());
                                headers
                            },
                        }
                    ))
                }
            )),
        };

        cmd_tx.send(Ok(intruder_cmd)).await.unwrap();

        // 7. Verify Intruder Response
        let event = event_rx.recv().await.expect("Should receive intruder response");
        assert_eq!(event.request_id, "test-intruder-1");
        if let Some(proxy_core::pb::traffic_event::Event::Response(resp)) = event.event {
            assert_eq!(resp.status_code, 200);
            assert_eq!(resp.body, b"Intruder Response");
        } else {
            panic!("Expected Response event for intruder");
        }

        // 8. Test Stop Attack Command
        let stop_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Attack(
                proxy_core::pb::AttackCommand {
                    command: Some(proxy_core::pb::attack_command::Command::StopAttack(true))
                }
            )),
        };

        cmd_tx.send(Ok(stop_cmd)).await.unwrap();
        
        // Give some time for the stop command to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_lifecycle_commands() {
        // 1. Start Mock Orchestrator (gRPC)
        let (cmd_tx, cmd_rx) = mpsc::channel::<Result<InterceptCommand, Status>>(10);
        let (event_tx, _event_rx) = mpsc::channel::<TrafficEvent>(10);

        let orchestrator_addr = "[::1]:50097".parse::<SocketAddr>().unwrap();
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

        // 2. Start Agent
        let (_log_tx, log_rx) = mpsc::channel(1);
        let client = OrchestratorClient::new(
            "http://[::1]:50097".to_string(),
            "test-agent-lifecycle".to_string(),
            "test-lifecycle-name".to_string(),
        );

        tokio::spawn(async move {
            client.run(log_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for agent connection

        // 3. Test Restart Command
        let restart_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Lifecycle(
                proxy_core::pb::LifecycleCommand {
                    action: proxy_core::pb::lifecycle_command::Action::Restart as i32,
                    force: false,
                }
            )),
        };

        cmd_tx.send(Ok(restart_cmd)).await.unwrap();
        
        // Give some time for the restart command to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 4. Test Shutdown Command
        let shutdown_cmd = InterceptCommand {
            command: Some(intercept_command::Command::Lifecycle(
                proxy_core::pb::LifecycleCommand {
                    action: proxy_core::pb::lifecycle_command::Action::Shutdown as i32,
                    force: true,
                }
            )),
        };

        cmd_tx.send(Ok(shutdown_cmd)).await.unwrap();
        
        // Give some time for the shutdown command to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    struct TestProxy {
        cmd_rx: std::sync::Arc<
            tokio::sync::Mutex<Option<mpsc::Receiver<Result<InterceptCommand, Status>>>>,
        >,
        event_tx: mpsc::Sender<TrafficEvent>,
    }

    #[tonic::async_trait]
    impl ProxyService for TestProxy {
        async fn register_agent(
            &self,
            _req: Request<RegisterAgentRequest>,
        ) -> Result<Response<RegisterAgentResponse>, Status> {
            Ok(Response::new(RegisterAgentResponse {
                success: true,
                message: "".into(),
                ca_cert_pem: "cert".into(),
                ca_key_pem: "key".into(),
            }))
        }

        type StreamTrafficStream = ReceiverStream<Result<InterceptCommand, Status>>;

        async fn stream_traffic(
            &self,
            req: Request<Streaming<TrafficEvent>>,
        ) -> Result<Response<Self::StreamTrafficStream>, Status> {
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

        type StreamMetricsStream = ReceiverStream<Result<MetricsCommand, Status>>;

        async fn stream_metrics(
            &self,
            _req: Request<Streaming<SystemMetricsEvent>>,
        ) -> Result<Response<Self::StreamMetricsStream>, Status> {
            let (_, rx) = mpsc::channel(1);
            Ok(Response::new(ReceiverStream::new(rx)))
        }
    }
}
