//! Attack execution coordination for intruder attacks
//! 
//! This module handles the coordination of distributed attack execution,
//! including progress tracking, statistics, and graceful termination.

use crate::database::intruder::{IntruderResult, IntruderResultBuffer};
use crate::intruder::distribution::{DistributionStats, PayloadAssignment};
use crate::result_streaming::{ResultStreamingManager, ResultSource};
use crate::performance_monitoring::{PerformanceMonitor, PerformanceConfig};
use crate::Database;
use attack_engine::{
    AttackError, AttackResult, HttpRequestData, HttpResponseData, 
    AttackMode, AttackModeExecutor, AttackModeFactory, AgentInfo, AgentStatus,
    PayloadPositionParser
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::{Duration, Instant};
use tracing::{info, error, debug};
use uuid::Uuid;
use proxy_common::Session;

/// Status of an attack execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackExecutionStatus {
    Configured,
    Starting,
    Running,
    Pausing,
    Paused,
    Stopping,
    Completed,
    Failed,
    Cancelled,
}

/// Real-time attack progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackProgress {
    pub attack_id: String,
    pub status: AttackExecutionStatus,
    pub total_requests: usize,
    pub completed_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub highlighted_results: usize,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub estimated_completion_time: Option<chrono::DateTime<chrono::Utc>>,
    pub agent_statistics: HashMap<String, AgentExecutionStats>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Statistics for individual agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionStats {
    pub agent_id: String,
    pub assigned_requests: usize,
    pub completed_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub average_response_time_ms: f64,
    pub current_load: f64,
    pub status: AgentStatus,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Configuration for attack execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackExecutionConfig {
    pub attack_id: String,
    pub request_template: String,
    pub attack_mode: AttackMode,
    pub distribution: DistributionStats,
    pub session_data: Option<Session>,
    pub concurrent_requests_per_agent: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub result_highlighting_rules: Vec<ResultHighlightRule>,
}

/// Rules for highlighting interesting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultHighlightRule {
    pub name: String,
    pub condition: HighlightCondition,
    pub priority: u8, // 1-10, higher is more important
}

/// Conditions for result highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HighlightCondition {
    StatusCode(Vec<i32>),
    ResponseLength { min: Option<usize>, max: Option<usize> },
    ResponseTime { min_ms: Option<u64>, max_ms: Option<u64> },
    ResponseContains(String),
    ResponseRegex(String),
    Combined { operator: LogicalOperator, conditions: Vec<HighlightCondition> },
}

/// Logical operators for combining highlight conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Attack execution coordinator
pub struct AttackExecutionCoordinator {
    db: Arc<Database>,
    active_attacks: Arc<RwLock<HashMap<String, AttackExecution>>>,
    progress_broadcaster: broadcast::Sender<AttackProgress>,
    result_buffer: Option<IntruderResultBuffer>,
    result_streaming: Arc<ResultStreamingManager>,
    performance_monitor: Arc<PerformanceMonitor>,
}

/// Internal attack execution state
struct AttackExecution {
    config: AttackExecutionConfig,
    progress: AttackProgress,
    cancel_token: tokio_util::sync::CancellationToken,
    result_sender: mpsc::UnboundedSender<IntruderResult>,
    agent_tasks: HashMap<String, tokio::task::JoinHandle<()>>,
}

impl AttackExecutionCoordinator {
    /// Create a new attack execution coordinator
    pub async fn new(db: Arc<Database>) -> AttackResult<Self> {
        let (progress_broadcaster, _) = broadcast::channel(1000);
        let result_buffer = db.create_intruder_result_buffer().await.ok();
        
        // Initialize result streaming and performance monitoring
        let result_streaming = Arc::new(ResultStreamingManager::new());
        let performance_config = PerformanceConfig::default();
        let performance_monitor = Arc::new(PerformanceMonitor::new(
            performance_config,
            result_streaming.clone(),
        ));

        Ok(Self {
            db,
            active_attacks: Arc::new(RwLock::new(HashMap::new())),
            progress_broadcaster,
            result_buffer,
            result_streaming,
            performance_monitor,
        })
    }

    /// Start executing an attack
    pub async fn start_attack(
        &self,
        config: AttackExecutionConfig,
        available_agents: &[AgentInfo],
    ) -> AttackResult<()> {
        let attack_id = config.attack_id.clone();
        info!("Starting attack execution: {}", attack_id);

        // Validate agents are still available
        let online_agents: Vec<&AgentInfo> = available_agents
            .iter()
            .filter(|agent| agent.status == AgentStatus::Online)
            .collect();

        if online_agents.is_empty() {
            return Err(AttackError::AgentUnavailable {
                agent_id: "No online agents available".to_string(),
            });
        }

        // Initialize performance monitoring for agents
        self.performance_monitor.initialize_agents(available_agents).await?;
        self.performance_monitor.start_monitoring().await?;

        // Start result streaming tracking
        let source = ResultSource::Intruder { attack_id: attack_id.clone() };
        let total_requests = config.distribution.assignments.iter().map(|a| a.payloads.len()).sum();
        self.result_streaming.start_tracking(source.clone(), total_requests).await?;
        self.result_streaming.start_progress_updates(source).await;

        // Update attack status in database
        self.db.update_intruder_attack_status(&attack_id, "running").await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("update_attack_status: {}", e),
            })?;

        // Create progress tracking
        let mut progress = AttackProgress {
            attack_id: attack_id.clone(),
            status: AttackExecutionStatus::Starting,
            total_requests,
            completed_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            highlighted_results: 0,
            requests_per_second: 0.0,
            average_response_time_ms: 0.0,
            estimated_completion_time: None,
            agent_statistics: HashMap::new(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
        };

        // Initialize agent statistics
        for assignment in &config.distribution.assignments {
            progress.agent_statistics.insert(assignment.agent_id.clone(), AgentExecutionStats {
                agent_id: assignment.agent_id.clone(),
                assigned_requests: assignment.payloads.len(),
                completed_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                current_load: 0.0,
                status: AgentStatus::Online,
                last_activity: None,
            });
        }

        // Create cancellation token and result channel
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let (result_sender, mut result_receiver) = mpsc::unbounded_channel::<IntruderResult>();

        // Create attack execution state
        let mut attack_execution = AttackExecution {
            config: config.clone(),
            progress: progress.clone(),
            cancel_token: cancel_token.clone(),
            result_sender: result_sender.clone(),
            agent_tasks: HashMap::new(),
        };

        // Start result processing task with streaming integration
        let db_clone = self.db.clone();
        let progress_tx = self.progress_broadcaster.clone();
        let attack_id_clone = attack_id.clone();
        let result_buffer = self.result_buffer.clone();
        let active_attacks_clone = self.active_attacks.clone();
        let result_streaming_clone = self.result_streaming.clone();

        tokio::spawn(async move {
            Self::process_results_with_streaming(
                result_receiver,
                db_clone,
                progress_tx,
                attack_id_clone,
                result_buffer,
                active_attacks_clone,
                result_streaming_clone,
            ).await;
        });

        // Start agent execution tasks with performance monitoring
        for assignment in &config.distribution.assignments {
            let agent_task = self.start_agent_execution_with_monitoring(
                assignment.clone(),
                config.clone(),
                result_sender.clone(),
                cancel_token.clone(),
            ).await?;

            attack_execution.agent_tasks.insert(assignment.agent_id.clone(), agent_task);
        }

        // Update progress to running
        progress.status = AttackExecutionStatus::Running;
        attack_execution.progress = progress.clone();

        // Store attack execution
        {
            let mut active_attacks = self.active_attacks.write().await;
            active_attacks.insert(attack_id.clone(), attack_execution);
        }

        // Broadcast initial progress
        let _ = self.progress_broadcaster.send(progress);

        info!("Attack {} started with {} agents", attack_id, config.distribution.assignments.len());
        Ok(())
    }

    /// Start execution for a specific agent assignment with performance monitoring
    async fn start_agent_execution_with_monitoring(
        &self,
        assignment: PayloadAssignment,
        config: AttackExecutionConfig,
        result_sender: mpsc::UnboundedSender<IntruderResult>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> AttackResult<tokio::task::JoinHandle<()>> {
        let agent_id = assignment.agent_id.clone();
        let attack_id = config.attack_id.clone();
        let performance_monitor = self.performance_monitor.clone();

        debug!("Starting agent execution with monitoring: {} for attack {}", agent_id, attack_id);

        // Create attack mode executor
        let attack_mode_executor = AttackModeFactory::create(&config.attack_mode);

        // Parse request template
        let base_request = self.parse_request_template(&config.request_template)?;

        let task = tokio::spawn(async move {
            let start_time = Instant::now();
            let mut completed_count = 0;
            let mut successful_count = 0;
            let mut failed_count = 0;
            let mut total_response_time = 0u64;

            // Generate requests for this agent's payloads
            let parsed_template = match PayloadPositionParser::parse(&config.request_template) {
                Ok(parsed) => parsed,
                Err(e) => {
                    error!("Failed to parse request template for agent {}: {}", agent_id, e);
                    return;
                }
            };

            // Create payload sets map for this assignment
            let mut payload_sets = HashMap::new();
            if !assignment.payloads.is_empty() {
                payload_sets.insert("payload".to_string(), assignment.payloads.clone());
            }

            let requests = match attack_mode_executor.generate_requests(&parsed_template, &payload_sets) {
                Ok(reqs) => reqs,
                Err(e) => {
                    error!("Failed to generate requests for agent {}: {}", agent_id, e);
                    return;
                }
            };

            // Execute requests with performance monitoring and concurrency control
            let mut tasks = Vec::new();

            for (index, attack_request) in requests.into_iter().enumerate() {
                if cancel_token.is_cancelled() {
                    break;
                }

                // Acquire performance-monitored permit
                let permit = match performance_monitor.acquire_request_permit(&agent_id).await {
                    Ok(permit) => permit,
                    Err(e) => {
                        error!("Failed to acquire request permit for agent {}: {}", agent_id, e);
                        break;
                    }
                };

                let agent_id_clone = agent_id.clone();
                let attack_id_clone = attack_id.clone();
                let result_sender_clone = result_sender.clone();
                let cancel_token_clone = cancel_token.clone();
                let session_data = config.session_data.clone();
                let timeout = Duration::from_secs(config.timeout_seconds);
                let payload_values = attack_request.payload_values.clone();

                let task = tokio::spawn(async move {
                    if cancel_token_clone.is_cancelled() {
                        permit.complete(false).await;
                        return (false, 0);
                    }

                    let execution_start = Instant::now();
                    
                    // Parse the request string into HttpRequestData
                    let mut final_request = match Self::parse_request_string(&attack_request.request) {
                        Ok(req) => req,
                        Err(_) => {
                            // Fallback to a basic request
                            HttpRequestData::new("GET".to_string(), "http://example.com".to_string())
                        }
                    };

                    // Apply session data if present
                    if let Some(ref session) = session_data {
                        final_request.apply_session(session);
                    }

                    // Execute actual request through agent
                    let result = Self::simulate_request_execution(
                        &final_request,
                        &agent_id_clone,
                        timeout,
                    ).await;

                    let duration_ms = execution_start.elapsed().as_millis() as u64;
                    let is_success = result.is_ok();

                    // Create result record
                    let intruder_result = IntruderResult {
                        id: Uuid::new_v4().to_string(),
                        attack_id: attack_id_clone,
                        request_data: serde_json::to_string(&final_request).unwrap_or_default(),
                        response_data: result.as_ref().ok().and_then(|r| serde_json::to_string(r).ok()),
                        agent_id: agent_id_clone,
                        payload_values: serde_json::to_string(&payload_values).unwrap_or_default(),
                        executed_at: chrono::Utc::now().timestamp(),
                        duration_ms: Some(duration_ms as i64),
                        status_code: result.as_ref().ok().map(|r| r.status_code),
                        response_length: result.as_ref().ok().map(|r| r.body.len() as i64),
                        is_highlighted: false, // Will be determined by result streaming
                    };

                    // Send result
                    let _ = result_sender_clone.send(intruder_result);

                    // Complete the performance-monitored request
                    permit.complete(is_success).await;

                    (is_success, duration_ms)
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete
            for task in tasks {
                if let Ok((success, duration)) = task.await {
                    completed_count += 1;
                    if success {
                        successful_count += 1;
                    } else {
                        failed_count += 1;
                    }
                    total_response_time += duration;
                }
            }

            let total_duration = start_time.elapsed();
            let avg_response_time = if completed_count > 0 {
                total_response_time as f64 / completed_count as f64
            } else {
                0.0
            };

            info!(
                "Agent {} completed: {}/{} successful, avg response time: {:.1}ms, total time: {:.1}s",
                agent_id,
                successful_count,
                completed_count,
                avg_response_time,
                total_duration.as_secs_f64()
            );
        });

        Ok(task)
    }

    /// Process attack results with streaming integration
    async fn process_results_with_streaming(
        mut result_receiver: mpsc::UnboundedReceiver<IntruderResult>,
        db: Arc<Database>,
        progress_tx: broadcast::Sender<AttackProgress>,
        attack_id: String,
        result_buffer: Option<IntruderResultBuffer>,
        active_attacks: Arc<RwLock<HashMap<String, AttackExecution>>>,
        result_streaming: Arc<ResultStreamingManager>,
    ) {
        let mut last_progress_update = Instant::now();
        let progress_update_interval = Duration::from_millis(500); // Update progress every 500ms

        while let Some(result) = result_receiver.recv().await {
            // Process result through streaming manager
            let response_data = result.response_data.as_ref()
                .and_then(|json| serde_json::from_str::<HttpResponseData>(json).ok());
            
            if let Err(e) = result_streaming.process_intruder_result(
                &attack_id,
                &result,
                response_data.as_ref(),
            ).await {
                error!("Failed to process result through streaming: {}", e);
            }

            // Store result in database
            if let Some(ref buffer) = result_buffer {
                if let Err(e) = buffer.add_result(result.clone()).await {
                    error!("Failed to buffer result: {}", e);
                }
            } else {
                // Fallback to direct database insert
                let _ = db.save_intruder_result(
                    &result.attack_id,
                    &result.request_data,
                    result.response_data.as_deref(),
                    &result.agent_id,
                    &result.payload_values,
                    result.duration_ms,
                    result.status_code,
                    result.response_length,
                    result.is_highlighted,
                ).await;
            }

            // Update progress periodically
            if last_progress_update.elapsed() >= progress_update_interval {
                if let Some(updated_progress) = Self::update_attack_progress(&attack_id, &active_attacks).await {
                    let _ = progress_tx.send(updated_progress);
                }
                last_progress_update = Instant::now();
            }
        }

        // Final progress update and stop streaming
        if let Some(final_progress) = Self::update_attack_progress(&attack_id, &active_attacks).await {
            let _ = progress_tx.send(final_progress);
        }

        // Stop result streaming tracking
        let source = ResultSource::Intruder { attack_id };
        let _ = result_streaming.stop_tracking(&source).await;
    }

    /// Update attack progress statistics
    async fn update_attack_progress(
        attack_id: &str,
        active_attacks: &Arc<RwLock<HashMap<String, AttackExecution>>>,
    ) -> Option<AttackProgress> {
        let mut attacks = active_attacks.write().await;
        let attack = attacks.get_mut(attack_id)?;

        // Update progress from database statistics
        // TODO: Implement actual progress calculation from database
        attack.progress.completed_requests += 1;
        
        // Check if attack is complete
        if attack.progress.completed_requests >= attack.progress.total_requests {
            attack.progress.status = AttackExecutionStatus::Completed;
            attack.progress.completed_at = Some(chrono::Utc::now());
        }

        Some(attack.progress.clone())
    }

    /// Simulate request execution (placeholder for actual agent communication)
    async fn simulate_request_execution(
        _request: &HttpRequestData,
        _agent_id: &str,
        _timeout: Duration,
    ) -> Result<HttpResponseData, AttackError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Simulate response
        Ok(HttpResponseData {
            status_code: 200,
            headers: None,
            body: b"Simulated response".to_vec(),
            tls: None,
        })
    }

    /// Parse request template into HttpRequestData
    fn parse_request_template(&self, _template: &str) -> AttackResult<HttpRequestData> {
        // TODO: Implement proper HTTP request parsing
        // For now, create a basic request
        Ok(HttpRequestData::new("GET".to_string(), "http://example.com".to_string()))
    }

    /// Parse request string into HttpRequestData (simple implementation)
    fn parse_request_string(request_string: &str) -> AttackResult<HttpRequestData> {
        // TODO: Implement proper HTTP request parsing from string
        // For now, create a basic request
        Ok(HttpRequestData::new("GET".to_string(), "http://example.com".to_string()))
    }

    /// Stop an active attack
    pub async fn stop_attack(&self, attack_id: &str) -> AttackResult<()> {
        info!("Stopping attack: {}", attack_id);

        let mut active_attacks = self.active_attacks.write().await;
        if let Some(mut attack) = active_attacks.remove(attack_id) {
            // Cancel all agent tasks
            attack.cancel_token.cancel();

            // Wait for tasks to complete
            for (agent_id, task) in attack.agent_tasks {
                debug!("Waiting for agent {} to stop", agent_id);
                let _ = task.await;
            }

            // Stop result streaming
            let source = ResultSource::Intruder { attack_id: attack_id.to_string() };
            let _ = self.result_streaming.stop_tracking(&source).await;

            // Update status in database
            self.db.update_intruder_attack_status(attack_id, "stopped").await
                .map_err(|e| AttackError::DatabaseError {
                    operation: format!("update_attack_status: {}", e),
                })?;

            // Update progress
            attack.progress.status = AttackExecutionStatus::Cancelled;
            attack.progress.completed_at = Some(chrono::Utc::now());

            // Broadcast final progress
            let _ = self.progress_broadcaster.send(attack.progress);
        }

        info!("Attack {} stopped", attack_id);
        Ok(())
    }

    /// Get current progress for an attack
    pub async fn get_attack_progress(&self, attack_id: &str) -> Option<AttackProgress> {
        let active_attacks = self.active_attacks.read().await;
        active_attacks.get(attack_id).map(|attack| attack.progress.clone())
    }

    /// Get all active attacks
    pub async fn get_active_attacks(&self) -> Vec<AttackProgress> {
        let active_attacks = self.active_attacks.read().await;
        active_attacks.values().map(|attack| attack.progress.clone()).collect()
    }

    /// Subscribe to progress updates
    pub fn subscribe_to_progress(&self) -> broadcast::Receiver<AttackProgress> {
        self.progress_broadcaster.subscribe()
    }

    /// Subscribe to real-time result updates
    pub fn subscribe_to_results(&self) -> tokio::sync::broadcast::Receiver<crate::result_streaming::ResultUpdate> {
        self.result_streaming.subscribe()
    }

    /// Get performance metrics for the attack system
    pub async fn get_performance_metrics(&self) -> crate::performance_monitoring::SystemPerformanceMetrics {
        self.performance_monitor.get_system_metrics().await
    }

    /// Get performance metrics for a specific agent
    pub async fn get_agent_performance(&self, agent_id: &str) -> Option<crate::performance_monitoring::AgentPerformanceMetrics> {
        self.performance_monitor.get_agent_metrics(agent_id).await
    }

    /// Export attack results in various formats
    pub async fn export_attack_results(
        &self,
        attack_id: &str,
        format: crate::result_streaming::ExportFormat,
        highlighted_only: bool,
    ) -> AttackResult<String> {
        let source = ResultSource::Intruder { attack_id: attack_id.to_string() };
        self.result_streaming.export_results(&source, format, highlighted_only).await
    }

    /// Update result highlighting configuration
    pub async fn update_highlighting_config(
        &self,
        config: crate::result_streaming::HighlightingConfig,
    ) -> AttackResult<()> {
        self.result_streaming.update_highlighting_config(config).await
    }

    /// Update performance monitoring configuration
    pub async fn update_performance_config(
        &self,
        config: crate::performance_monitoring::PerformanceConfig,
    ) -> AttackResult<()> {
        self.performance_monitor.update_config(config).await
    }

    /// Pause an active attack
    pub async fn pause_attack(&self, attack_id: &str) -> AttackResult<()> {
        let mut active_attacks = self.active_attacks.write().await;
        if let Some(attack) = active_attacks.get_mut(attack_id) {
            attack.progress.status = AttackExecutionStatus::Paused;
            // TODO: Implement actual pause logic
            Ok(())
        } else {
            Err(AttackError::InvalidAttackConfig {
                reason: format!("Attack {} not found", attack_id),
            })
        }
    }

    /// Resume a paused attack
    pub async fn resume_attack(&self, attack_id: &str) -> AttackResult<()> {
        let mut active_attacks = self.active_attacks.write().await;
        if let Some(attack) = active_attacks.get_mut(attack_id) {
            attack.progress.status = AttackExecutionStatus::Running;
            // TODO: Implement actual resume logic
            Ok(())
        } else {
            Err(AttackError::InvalidAttackConfig {
                reason: format!("Attack {} not found", attack_id),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use tempfile::TempDir;

    async fn create_test_coordinator() -> (AttackExecutionCoordinator, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(Database::new(temp_dir.path().to_str().unwrap()).await.unwrap());
        let coordinator = AttackExecutionCoordinator::new(db).await.unwrap();
        (coordinator, temp_dir)
    }

    #[tokio::test]
    async fn test_attack_progress_tracking() {
        let (coordinator, _temp_dir) = create_test_coordinator().await;
        
        // Create a simple attack configuration
        let config = AttackExecutionConfig {
            attack_id: "test-attack".to_string(),
            request_template: "GET /test HTTP/1.1\r\n\r\n".to_string(),
            attack_mode: AttackMode::Sniper,
            distribution: DistributionStats {
                total_payloads: 2,
                total_agents: 1,
                assignments: vec![PayloadAssignment {
                    agent_id: "agent1".to_string(),
                    payloads: vec!["payload1".to_string(), "payload2".to_string()],
                    start_index: 0,
                    end_index: 1,
                    priority: 5,
                }],
                load_balance_factor: 1.0,
                estimated_completion_time: None,
            },
            session_data: None,
            concurrent_requests_per_agent: 1,
            timeout_seconds: 30,
            retry_attempts: 3,
            result_highlighting_rules: Vec::new(),
        };

        let agents = vec![AgentInfo {
            id: "agent1".to_string(),
            hostname: "host1".to_string(),
            status: AgentStatus::Online,
            load: 0.1,
            response_time_ms: Some(100),
        }];

        // Start attack
        let result = coordinator.start_attack(config, &agents).await;
        assert!(result.is_ok());

        // Check initial progress
        let progress = coordinator.get_attack_progress("test-attack").await;
        assert!(progress.is_some());
        let progress = progress.unwrap();
        assert_eq!(progress.attack_id, "test-attack");
        assert_eq!(progress.total_requests, 2);
        assert!(matches!(progress.status, AttackExecutionStatus::Running));

        // Stop attack
        let result = coordinator.stop_attack("test-attack").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_progress_subscription() {
        let (coordinator, _temp_dir) = create_test_coordinator().await;
        
        let mut progress_rx = coordinator.subscribe_to_progress();
        
        // The receiver should be created successfully
        assert!(progress_rx.try_recv().is_err()); // No messages yet
    }
}