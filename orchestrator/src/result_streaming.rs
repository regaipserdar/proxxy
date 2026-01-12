//! Result streaming infrastructure for real-time attack monitoring
//! 
//! This module provides real-time result broadcasting, progress tracking,
//! result highlighting, and export functionality for repeater and intruder attacks.

use crate::database::intruder::IntruderResult;
use crate::database::repeater::RepeaterExecution;
use attack_engine::{AttackError, AttackResult, HttpResponseData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, mpsc};
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Real-time result update for streaming to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultUpdate {
    pub update_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub update_type: ResultUpdateType,
    pub source: ResultSource,
}

/// Type of result update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultUpdateType {
    NewResult(StreamedResult),
    ProgressUpdate(ProgressStatistics),
    HighlightedResult(HighlightedResult),
    AttackCompleted(AttackCompletionSummary),
    AttackError(AttackErrorInfo),
}

/// Source of the result update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultSource {
    Repeater { tab_id: String },
    Intruder { attack_id: String },
}

/// Streamlined result for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamedResult {
    pub result_id: String,
    pub agent_id: String,
    pub status_code: Option<i32>,
    pub response_length: Option<usize>,
    pub duration_ms: Option<u64>,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub payload_values: Option<Vec<String>>,
    pub is_highlighted: bool,
    pub highlight_reasons: Vec<String>,
}

/// Progress statistics for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStatistics {
    pub total_requests: usize,
    pub completed_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub highlighted_results: usize,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub estimated_completion_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status_code_distribution: HashMap<i32, usize>,
    pub agent_performance: HashMap<String, AgentPerformanceStats>,
}

/// Performance statistics for individual agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceStats {
    pub agent_id: String,
    pub completed_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub average_response_time_ms: f64,
    pub requests_per_second: f64,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Highlighted result with highlighting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightedResult {
    pub result: StreamedResult,
    pub highlight_rules: Vec<HighlightRuleMatch>,
    pub priority_score: u8,
}

/// Information about which highlight rule matched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRuleMatch {
    pub rule_name: String,
    pub rule_priority: u8,
    pub match_reason: String,
}

/// Attack completion summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackCompletionSummary {
    pub total_duration: Duration,
    pub final_statistics: ProgressStatistics,
    pub top_highlighted_results: Vec<HighlightedResult>,
    pub export_options: Vec<ExportFormat>,
}

/// Attack error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackErrorInfo {
    pub error_type: String,
    pub message: String,
    pub affected_agents: Vec<String>,
    pub is_recoverable: bool,
    pub suggested_actions: Vec<String>,
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Html,
}

/// Configuration for result highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightingConfig {
    pub rules: Vec<HighlightRule>,
    pub max_highlighted_results: usize,
    pub auto_highlight_enabled: bool,
}

/// Rule for highlighting interesting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: HighlightCondition,
    pub priority: u8, // 1-10, higher is more important
    pub enabled: bool,
    pub color: Option<String>, // Hex color for UI display
}

/// Conditions for result highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HighlightCondition {
    StatusCode(Vec<i32>),
    StatusCodeRange { min: i32, max: i32 },
    ResponseLength { min: Option<usize>, max: Option<usize> },
    ResponseTime { min_ms: Option<u64>, max_ms: Option<u64> },
    ResponseContains { text: String, case_sensitive: bool },
    ResponseRegex(String),
    HeaderExists(String),
    HeaderValue { name: String, value: String, case_sensitive: bool },
    Combined { operator: LogicalOperator, conditions: Vec<HighlightCondition> },
}

/// Logical operators for combining highlight conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Statistics tracker for progress calculation
#[derive(Debug)]
struct StatisticsTracker {
    start_time: Instant,
    total_requests: usize,
    completed_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    highlighted_results: usize,
    response_times: Vec<u64>,
    status_codes: HashMap<i32, usize>,
    agent_stats: HashMap<String, AgentStatsTracker>,
    last_update: Instant,
}

/// Agent-specific statistics tracker
#[derive(Debug)]
struct AgentStatsTracker {
    completed_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    response_times: Vec<u64>,
    last_activity: Option<Instant>,
}

/// Main result streaming manager
pub struct ResultStreamingManager {
    // Broadcasting channels
    result_broadcaster: broadcast::Sender<ResultUpdate>,
    
    // Statistics tracking
    statistics: Arc<RwLock<HashMap<String, StatisticsTracker>>>,
    
    // Highlighting configuration
    highlighting_config: Arc<RwLock<HighlightingConfig>>,
    
    // Result storage for export
    result_storage: Arc<RwLock<HashMap<String, Vec<StreamedResult>>>>,
    
    // Update interval for progress broadcasting
    update_interval: Duration,
}

impl ResultStreamingManager {
    /// Create a new result streaming manager
    pub fn new() -> Self {
        let (result_broadcaster, _) = broadcast::channel(10000);
        
        let default_highlighting = HighlightingConfig {
            rules: Self::create_default_highlight_rules(),
            max_highlighted_results: 1000,
            auto_highlight_enabled: true,
        };

        Self {
            result_broadcaster,
            statistics: Arc::new(RwLock::new(HashMap::new())),
            highlighting_config: Arc::new(RwLock::new(default_highlighting)),
            result_storage: Arc::new(RwLock::new(HashMap::new())),
            update_interval: Duration::from_millis(500),
        }
    }

    /// Subscribe to real-time result updates
    pub fn subscribe(&self) -> broadcast::Receiver<ResultUpdate> {
        self.result_broadcaster.subscribe()
    }

    /// Start tracking statistics for a new attack or repeater session
    pub async fn start_tracking(&self, source: ResultSource, total_requests: usize) -> AttackResult<()> {
        let source_id = self.get_source_id(&source);
        
        let tracker = StatisticsTracker {
            start_time: Instant::now(),
            total_requests,
            completed_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            highlighted_results: 0,
            response_times: Vec::new(),
            status_codes: HashMap::new(),
            agent_stats: HashMap::new(),
            last_update: Instant::now(),
        };

        {
            let mut statistics = self.statistics.write().await;
            statistics.insert(source_id.clone(), tracker);
        }

        // Initialize result storage
        {
            let mut storage = self.result_storage.write().await;
            storage.insert(source_id, Vec::new());
        }

        info!("Started result tracking for: {:?}", source);
        Ok(())
    }

    /// Stop tracking and clean up resources
    pub async fn stop_tracking(&self, source: &ResultSource) -> AttackResult<()> {
        let source_id = self.get_source_id(source);
        
        // Generate completion summary
        if let Some(final_stats) = self.get_current_statistics(&source_id).await {
            let completion_summary = AttackCompletionSummary {
                total_duration: Instant::now().duration_since(
                    self.statistics.read().await
                        .get(&source_id)
                        .map(|s| s.start_time)
                        .unwrap_or_else(Instant::now)
                ),
                final_statistics: final_stats,
                top_highlighted_results: self.get_top_highlighted_results(&source_id, 10).await,
                export_options: vec![
                    ExportFormat::Json,
                    ExportFormat::Csv,
                    ExportFormat::Html,
                ],
            };

            let update = ResultUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                update_type: ResultUpdateType::AttackCompleted(completion_summary),
                source: source.clone(),
            };

            let _ = self.result_broadcaster.send(update);
        }

        // Clean up tracking data
        {
            let mut statistics = self.statistics.write().await;
            statistics.remove(&source_id);
        }

        info!("Stopped result tracking for: {:?}", source);
        Ok(())
    }

    /// Process a new intruder result
    pub async fn process_intruder_result(
        &self,
        attack_id: &str,
        result: &IntruderResult,
        response_data: Option<&HttpResponseData>,
    ) -> AttackResult<()> {
        let source = ResultSource::Intruder { attack_id: attack_id.to_string() };
        let source_id = self.get_source_id(&source);

        // Parse payload values
        let payload_values: Option<Vec<String>> = serde_json::from_str::<Vec<String>>(&result.payload_values).ok();

        // Create streamed result
        let mut streamed_result = StreamedResult {
            result_id: result.id.clone(),
            agent_id: result.agent_id.clone(),
            status_code: result.status_code,
            response_length: result.response_length.map(|l| l as usize),
            duration_ms: result.duration_ms.map(|d| d as u64),
            executed_at: chrono::DateTime::from_timestamp(result.executed_at, 0)
                .unwrap_or_else(chrono::Utc::now),
            payload_values,
            is_highlighted: false,
            highlight_reasons: Vec::new(),
        };

        // Apply highlighting rules
        if let Some(response_data) = response_data {
            let highlighting = self.apply_highlighting_rules(&streamed_result, response_data).await;
            streamed_result.is_highlighted = highlighting.is_highlighted;
            streamed_result.highlight_reasons = highlighting.reasons;
        }

        // Update statistics
        self.update_statistics(&source_id, &streamed_result).await?;

        // Store result
        {
            let mut storage = self.result_storage.write().await;
            if let Some(results) = storage.get_mut(&source_id) {
                results.push(streamed_result.clone());
            }
        }

        // Broadcast new result
        let update = ResultUpdate {
            update_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            update_type: ResultUpdateType::NewResult(streamed_result.clone()),
            source,
        };

        let _ = self.result_broadcaster.send(update);

        // Broadcast highlighted result if applicable
        if streamed_result.is_highlighted {
            let highlighted = HighlightedResult {
                result: streamed_result,
                highlight_rules: Vec::new(), // TODO: Include matched rules
                priority_score: 5, // TODO: Calculate based on rules
            };

            let highlight_update = ResultUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                update_type: ResultUpdateType::HighlightedResult(highlighted),
                source: ResultSource::Intruder { attack_id: attack_id.to_string() },
            };

            let _ = self.result_broadcaster.send(highlight_update);
        }

        Ok(())
    }

    /// Process a new repeater result
    pub async fn process_repeater_result(
        &self,
        tab_id: &str,
        execution: &RepeaterExecution,
        response_data: Option<&HttpResponseData>,
    ) -> AttackResult<()> {
        let source = ResultSource::Repeater { tab_id: tab_id.to_string() };
        let source_id = self.get_source_id(&source);

        // Create streamed result
        let mut streamed_result = StreamedResult {
            result_id: execution.id.clone(),
            agent_id: execution.agent_id.clone(),
            status_code: execution.status_code,
            response_length: response_data.map(|r| r.body.len()),
            duration_ms: execution.duration_ms.map(|d| d as u64),
            executed_at: chrono::DateTime::from_timestamp(execution.executed_at, 0)
                .unwrap_or_else(chrono::Utc::now),
            payload_values: None, // Repeater doesn't use payloads
            is_highlighted: false,
            highlight_reasons: Vec::new(),
        };

        // Apply highlighting rules
        if let Some(response_data) = response_data {
            let highlighting = self.apply_highlighting_rules(&streamed_result, response_data).await;
            streamed_result.is_highlighted = highlighting.is_highlighted;
            streamed_result.highlight_reasons = highlighting.reasons;
        }

        // Update statistics
        self.update_statistics(&source_id, &streamed_result).await?;

        // Store result
        {
            let mut storage = self.result_storage.write().await;
            if let Some(results) = storage.get_mut(&source_id) {
                results.push(streamed_result.clone());
            }
        }

        // Broadcast new result
        let update = ResultUpdate {
            update_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            update_type: ResultUpdateType::NewResult(streamed_result),
            source,
        };

        let _ = self.result_broadcaster.send(update);

        Ok(())
    }

    /// Start periodic progress updates
    pub async fn start_progress_updates(&self, source: ResultSource) {
        let source_id = self.get_source_id(&source);
        let statistics = self.statistics.clone();
        let broadcaster = self.result_broadcaster.clone();
        let update_interval = self.update_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            
            loop {
                interval.tick().await;
                
                // Check if tracking is still active
                let stats_exist = {
                    let stats = statistics.read().await;
                    stats.contains_key(&source_id)
                };

                if !stats_exist {
                    break;
                }

                // Generate progress update
                if let Some(progress) = Self::calculate_progress_statistics(&statistics, &source_id).await {
                    let update = ResultUpdate {
                        update_id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now(),
                        update_type: ResultUpdateType::ProgressUpdate(progress),
                        source: source.clone(),
                    };

                    if broadcaster.send(update).is_err() {
                        break; // No more receivers
                    }
                }
            }
        });
    }

    /// Update highlighting configuration
    pub async fn update_highlighting_config(&self, config: HighlightingConfig) -> AttackResult<()> {
        {
            let mut highlighting = self.highlighting_config.write().await;
            *highlighting = config;
        }
        
        info!("Updated highlighting configuration");
        Ok(())
    }

    /// Get current highlighting configuration
    pub async fn get_highlighting_config(&self) -> HighlightingConfig {
        self.highlighting_config.read().await.clone()
    }

    /// Export results in the specified format
    pub async fn export_results(
        &self,
        source: &ResultSource,
        format: ExportFormat,
        filter_highlighted_only: bool,
    ) -> AttackResult<String> {
        let source_id = self.get_source_id(source);
        
        let results = {
            let storage = self.result_storage.read().await;
            storage.get(&source_id)
                .map(|results| {
                    if filter_highlighted_only {
                        results.iter().filter(|r| r.is_highlighted).cloned().collect()
                    } else {
                        results.clone()
                    }
                })
                .unwrap_or_default()
        };

        match format {
            ExportFormat::Json => self.export_as_json(&results),
            ExportFormat::Csv => self.export_as_csv(&results),
            ExportFormat::Xml => self.export_as_xml(&results),
            ExportFormat::Html => self.export_as_html(&results),
        }
    }

    /// Get current statistics for a source
    async fn get_current_statistics(&self, source_id: &str) -> Option<ProgressStatistics> {
        Self::calculate_progress_statistics(&self.statistics, source_id).await
    }

    /// Calculate progress statistics
    async fn calculate_progress_statistics(
        statistics: &Arc<RwLock<HashMap<String, StatisticsTracker>>>,
        source_id: &str,
    ) -> Option<ProgressStatistics> {
        let stats = statistics.read().await;
        let tracker = stats.get(source_id)?;

        let elapsed = tracker.start_time.elapsed().as_secs_f64();
        let requests_per_second = if elapsed > 0.0 {
            tracker.completed_requests as f64 / elapsed
        } else {
            0.0
        };

        let average_response_time = if !tracker.response_times.is_empty() {
            tracker.response_times.iter().sum::<u64>() as f64 / tracker.response_times.len() as f64
        } else {
            0.0
        };

        let estimated_completion_time = if requests_per_second > 0.0 && tracker.completed_requests < tracker.total_requests {
            let remaining_requests = tracker.total_requests - tracker.completed_requests;
            let estimated_seconds = remaining_requests as f64 / requests_per_second;
            Some(chrono::Utc::now() + chrono::Duration::seconds(estimated_seconds as i64))
        } else {
            None
        };

        let mut agent_performance = HashMap::new();
        for (agent_id, agent_stats) in &tracker.agent_stats {
            let agent_elapsed = agent_stats.last_activity
                .map(|last| last.elapsed().as_secs_f64())
                .unwrap_or(elapsed);
            
            let agent_rps = if agent_elapsed > 0.0 {
                agent_stats.completed_requests as f64 / agent_elapsed
            } else {
                0.0
            };

            let agent_avg_time = if !agent_stats.response_times.is_empty() {
                agent_stats.response_times.iter().sum::<u64>() as f64 / agent_stats.response_times.len() as f64
            } else {
                0.0
            };

            agent_performance.insert(agent_id.clone(), AgentPerformanceStats {
                agent_id: agent_id.clone(),
                completed_requests: agent_stats.completed_requests,
                successful_requests: agent_stats.successful_requests,
                failed_requests: agent_stats.failed_requests,
                average_response_time_ms: agent_avg_time,
                requests_per_second: agent_rps,
                last_activity: agent_stats.last_activity.map(|instant| {
                    chrono::Utc::now() - chrono::Duration::from_std(instant.elapsed()).unwrap_or_default()
                }),
            });
        }

        Some(ProgressStatistics {
            total_requests: tracker.total_requests,
            completed_requests: tracker.completed_requests,
            successful_requests: tracker.successful_requests,
            failed_requests: tracker.failed_requests,
            highlighted_results: tracker.highlighted_results,
            requests_per_second,
            average_response_time_ms: average_response_time,
            estimated_completion_time,
            status_code_distribution: tracker.status_codes.clone(),
            agent_performance,
        })
    }

    /// Update statistics with a new result
    async fn update_statistics(&self, source_id: &str, result: &StreamedResult) -> AttackResult<()> {
        let mut statistics = self.statistics.write().await;
        let tracker = statistics.get_mut(source_id)
            .ok_or_else(|| AttackError::InvalidAttackConfig {
                reason: format!("No statistics tracker found for source: {}", source_id),
            })?;

        tracker.completed_requests += 1;
        
        if let Some(status_code) = result.status_code {
            if status_code < 400 {
                tracker.successful_requests += 1;
            } else {
                tracker.failed_requests += 1;
            }
            
            *tracker.status_codes.entry(status_code).or_insert(0) += 1;
        } else {
            tracker.failed_requests += 1;
        }

        if result.is_highlighted {
            tracker.highlighted_results += 1;
        }

        if let Some(duration) = result.duration_ms {
            tracker.response_times.push(duration);
            
            // Keep only recent response times to prevent memory growth
            if tracker.response_times.len() > 10000 {
                tracker.response_times.drain(0..5000);
            }
        }

        // Update agent statistics
        let agent_stats = tracker.agent_stats.entry(result.agent_id.clone())
            .or_insert_with(|| AgentStatsTracker {
                completed_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                response_times: Vec::new(),
                last_activity: None,
            });

        agent_stats.completed_requests += 1;
        agent_stats.last_activity = Some(Instant::now());

        if let Some(status_code) = result.status_code {
            if status_code < 400 {
                agent_stats.successful_requests += 1;
            } else {
                agent_stats.failed_requests += 1;
            }
        } else {
            agent_stats.failed_requests += 1;
        }

        if let Some(duration) = result.duration_ms {
            agent_stats.response_times.push(duration);
            
            // Keep only recent response times per agent
            if agent_stats.response_times.len() > 1000 {
                agent_stats.response_times.drain(0..500);
            }
        }

        tracker.last_update = Instant::now();
        Ok(())
    }

    /// Apply highlighting rules to a result
    async fn apply_highlighting_rules(
        &self,
        result: &StreamedResult,
        response_data: &HttpResponseData,
    ) -> HighlightingResult {
        let config = self.highlighting_config.read().await;
        
        if !config.auto_highlight_enabled {
            return HighlightingResult {
                is_highlighted: false,
                reasons: Vec::new(),
            };
        }

        let mut reasons = Vec::new();
        
        for rule in &config.rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_highlight_condition(&rule.condition, result, response_data) {
                reasons.push(format!("{}: {}", rule.name, rule.description));
            }
        }

        HighlightingResult {
            is_highlighted: !reasons.is_empty(),
            reasons,
        }
    }

    /// Evaluate a highlight condition
    fn evaluate_highlight_condition(
        &self,
        condition: &HighlightCondition,
        result: &StreamedResult,
        response_data: &HttpResponseData,
    ) -> bool {
        match condition {
            HighlightCondition::StatusCode(codes) => {
                result.status_code.map_or(false, |code| codes.contains(&code))
            }
            HighlightCondition::StatusCodeRange { min, max } => {
                result.status_code.map_or(false, |code| code >= *min && code <= *max)
            }
            HighlightCondition::ResponseLength { min, max } => {
                let length = response_data.body.len();
                let min_ok = min.map_or(true, |m| length >= m);
                let max_ok = max.map_or(true, |m| length <= m);
                min_ok && max_ok
            }
            HighlightCondition::ResponseTime { min_ms, max_ms } => {
                if let Some(duration) = result.duration_ms {
                    let min_ok = min_ms.map_or(true, |m| duration >= m);
                    let max_ok = max_ms.map_or(true, |m| duration <= m);
                    min_ok && max_ok
                } else {
                    false
                }
            }
            HighlightCondition::ResponseContains { text, case_sensitive } => {
                let body_str = String::from_utf8_lossy(&response_data.body);
                if *case_sensitive {
                    body_str.contains(text)
                } else {
                    body_str.to_lowercase().contains(&text.to_lowercase())
                }
            }
            HighlightCondition::ResponseRegex(pattern) => {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    let body_str = String::from_utf8_lossy(&response_data.body);
                    regex.is_match(&body_str)
                } else {
                    false
                }
            }
            HighlightCondition::HeaderExists(header_name) => {
                response_data.headers.as_ref()
                    .map_or(false, |headers| headers.headers.contains_key(header_name))
            }
            HighlightCondition::HeaderValue { name, value, case_sensitive } => {
                response_data.headers.as_ref()
                    .and_then(|headers| headers.headers.get(name))
                    .map_or(false, |header_value| {
                        if *case_sensitive {
                            header_value == value
                        } else {
                            header_value.to_lowercase() == value.to_lowercase()
                        }
                    })
            }
            HighlightCondition::Combined { operator, conditions } => {
                match operator {
                    LogicalOperator::And => {
                        conditions.iter().all(|c| self.evaluate_highlight_condition(c, result, response_data))
                    }
                    LogicalOperator::Or => {
                        conditions.iter().any(|c| self.evaluate_highlight_condition(c, result, response_data))
                    }
                    LogicalOperator::Not => {
                        !conditions.iter().any(|c| self.evaluate_highlight_condition(c, result, response_data))
                    }
                }
            }
        }
    }

    /// Get source ID from ResultSource
    fn get_source_id(&self, source: &ResultSource) -> String {
        match source {
            ResultSource::Repeater { tab_id } => format!("repeater:{}", tab_id),
            ResultSource::Intruder { attack_id } => format!("intruder:{}", attack_id),
        }
    }

    /// Get top highlighted results
    async fn get_top_highlighted_results(&self, source_id: &str, limit: usize) -> Vec<HighlightedResult> {
        let storage = self.result_storage.read().await;
        let results = storage.get(source_id).cloned().unwrap_or_default();
        
        results.into_iter()
            .filter(|r| r.is_highlighted)
            .take(limit)
            .map(|result| HighlightedResult {
                result,
                highlight_rules: Vec::new(), // TODO: Store matched rules
                priority_score: 5, // TODO: Calculate based on rules
            })
            .collect()
    }

    /// Create default highlight rules
    fn create_default_highlight_rules() -> Vec<HighlightRule> {
        vec![
            HighlightRule {
                id: "error_status".to_string(),
                name: "Error Status Codes".to_string(),
                description: "HTTP error status codes (4xx, 5xx)".to_string(),
                condition: HighlightCondition::StatusCodeRange { min: 400, max: 599 },
                priority: 7,
                enabled: true,
                color: Some("#ff6b6b".to_string()),
            },
            HighlightRule {
                id: "success_status".to_string(),
                name: "Success Status Codes".to_string(),
                description: "HTTP success status codes (2xx)".to_string(),
                condition: HighlightCondition::StatusCodeRange { min: 200, max: 299 },
                priority: 3,
                enabled: true,
                color: Some("#51cf66".to_string()),
            },
            HighlightRule {
                id: "large_response".to_string(),
                name: "Large Response".to_string(),
                description: "Response larger than 100KB".to_string(),
                condition: HighlightCondition::ResponseLength { min: Some(100000), max: None },
                priority: 5,
                enabled: true,
                color: Some("#ffd43b".to_string()),
            },
            HighlightRule {
                id: "slow_response".to_string(),
                name: "Slow Response".to_string(),
                description: "Response time over 5 seconds".to_string(),
                condition: HighlightCondition::ResponseTime { min_ms: Some(5000), max_ms: None },
                priority: 6,
                enabled: true,
                color: Some("#ff922b".to_string()),
            },
        ]
    }

    /// Export results as JSON
    fn export_as_json(&self, results: &[StreamedResult]) -> AttackResult<String> {
        serde_json::to_string_pretty(results)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize results as JSON: {}", e),
            })
    }

    /// Export results as CSV
    fn export_as_csv(&self, results: &[StreamedResult]) -> AttackResult<String> {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Result ID,Agent ID,Status Code,Response Length,Duration (ms),Executed At,Is Highlighted,Highlight Reasons\n");
        
        // Data rows
        for result in results {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                result.result_id,
                result.agent_id,
                result.status_code.map_or("".to_string(), |c| c.to_string()),
                result.response_length.map_or("".to_string(), |l| l.to_string()),
                result.duration_ms.map_or("".to_string(), |d| d.to_string()),
                result.executed_at.format("%Y-%m-%d %H:%M:%S UTC"),
                result.is_highlighted,
                result.highlight_reasons.join("; ")
            ));
        }
        
        Ok(csv)
    }

    /// Export results as XML
    fn export_as_xml(&self, results: &[StreamedResult]) -> AttackResult<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<results>\n");
        
        for result in results {
            xml.push_str("  <result>\n");
            xml.push_str(&format!("    <id>{}</id>\n", result.result_id));
            xml.push_str(&format!("    <agent_id>{}</agent_id>\n", result.agent_id));
            if let Some(status) = result.status_code {
                xml.push_str(&format!("    <status_code>{}</status_code>\n", status));
            }
            if let Some(length) = result.response_length {
                xml.push_str(&format!("    <response_length>{}</response_length>\n", length));
            }
            if let Some(duration) = result.duration_ms {
                xml.push_str(&format!("    <duration_ms>{}</duration_ms>\n", duration));
            }
            xml.push_str(&format!("    <executed_at>{}</executed_at>\n", result.executed_at.format("%Y-%m-%d %H:%M:%S UTC")));
            xml.push_str(&format!("    <is_highlighted>{}</is_highlighted>\n", result.is_highlighted));
            xml.push_str("  </result>\n");
        }
        
        xml.push_str("</results>\n");
        Ok(xml)
    }

    /// Export results as HTML
    fn export_as_html(&self, results: &[StreamedResult]) -> AttackResult<String> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Attack Results</title>\n");
        html.push_str("<style>\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f2f2f2; }\n");
        html.push_str(".highlighted { background-color: #fff3cd; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("<h1>Attack Results</h1>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Result ID</th><th>Agent ID</th><th>Status Code</th><th>Response Length</th><th>Duration (ms)</th><th>Executed At</th><th>Highlighted</th></tr>\n");
        
        for result in results {
            let row_class = if result.is_highlighted { " class=\"highlighted\"" } else { "" };
            html.push_str(&format!("<tr{}>\n", row_class));
            html.push_str(&format!("  <td>{}</td>\n", result.result_id));
            html.push_str(&format!("  <td>{}</td>\n", result.agent_id));
            html.push_str(&format!("  <td>{}</td>\n", result.status_code.map_or("".to_string(), |c| c.to_string())));
            html.push_str(&format!("  <td>{}</td>\n", result.response_length.map_or("".to_string(), |l| l.to_string())));
            html.push_str(&format!("  <td>{}</td>\n", result.duration_ms.map_or("".to_string(), |d| d.to_string())));
            html.push_str(&format!("  <td>{}</td>\n", result.executed_at.format("%Y-%m-%d %H:%M:%S UTC")));
            html.push_str(&format!("  <td>{}</td>\n", if result.is_highlighted { "Yes" } else { "No" }));
            html.push_str("</tr>\n");
        }
        
        html.push_str("</table>\n");
        html.push_str("</body>\n</html>\n");
        Ok(html)
    }
}

/// Result of highlighting evaluation
struct HighlightingResult {
    is_highlighted: bool,
    reasons: Vec<String>,
}

impl Default for ResultStreamingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use attack_engine::HttpHeaders;
    use std::collections::HashMap;

    fn create_test_result() -> StreamedResult {
        StreamedResult {
            result_id: "test-result-1".to_string(),
            agent_id: "agent-1".to_string(),
            status_code: Some(200),
            response_length: Some(1024),
            duration_ms: Some(150),
            executed_at: chrono::Utc::now(),
            payload_values: Some(vec!["test".to_string()]),
            is_highlighted: false,
            highlight_reasons: Vec::new(),
        }
    }

    fn create_test_response() -> HttpResponseData {
        HttpResponseData {
            status_code: 200,
            headers: Some(HttpHeaders {
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                    headers
                },
            }),
            body: b"Test response body".to_vec(),
            tls: None,
        }
    }

    #[tokio::test]
    async fn test_result_streaming_manager_creation() {
        let manager = ResultStreamingManager::new();
        let config = manager.get_highlighting_config().await;
        
        assert!(config.auto_highlight_enabled);
        assert!(!config.rules.is_empty());
        assert_eq!(config.max_highlighted_results, 1000);
    }

    #[tokio::test]
    async fn test_start_stop_tracking() {
        let manager = ResultStreamingManager::new();
        let source = ResultSource::Intruder { attack_id: "test-attack".to_string() };
        
        // Start tracking
        let result = manager.start_tracking(source.clone(), 100).await;
        assert!(result.is_ok());
        
        // Stop tracking
        let result = manager.stop_tracking(&source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_highlight_condition_evaluation() {
        let manager = ResultStreamingManager::new();
        let result = create_test_result();
        let response = create_test_response();
        
        // Test status code condition
        let condition = HighlightCondition::StatusCode(vec![200, 201]);
        assert!(manager.evaluate_highlight_condition(&condition, &result, &response));
        
        let condition = HighlightCondition::StatusCode(vec![404, 500]);
        assert!(!manager.evaluate_highlight_condition(&condition, &result, &response));
        
        // Test response length condition
        let condition = HighlightCondition::ResponseLength { min: Some(10), max: Some(100) };
        assert!(manager.evaluate_highlight_condition(&condition, &result, &response));
        
        // Test response contains condition
        let condition = HighlightCondition::ResponseContains { 
            text: "Test response".to_string(), 
            case_sensitive: true 
        };
        assert!(manager.evaluate_highlight_condition(&condition, &result, &response));
    }

    #[tokio::test]
    async fn test_export_formats() {
        let manager = ResultStreamingManager::new();
        let results = vec![create_test_result()];
        
        // Test JSON export
        let json_result = manager.export_as_json(&results);
        assert!(json_result.is_ok());
        assert!(json_result.unwrap().contains("test-result-1"));
        
        // Test CSV export
        let csv_result = manager.export_as_csv(&results);
        assert!(csv_result.is_ok());
        assert!(csv_result.unwrap().contains("Result ID,Agent ID"));
        
        // Test XML export
        let xml_result = manager.export_as_xml(&results);
        assert!(xml_result.is_ok());
        assert!(xml_result.unwrap().contains("<results>"));
        
        // Test HTML export
        let html_result = manager.export_as_html(&results);
        assert!(html_result.is_ok());
        assert!(html_result.unwrap().contains("<table>"));
    }

    #[tokio::test]
    async fn test_subscription() {
        let manager = ResultStreamingManager::new();
        let mut receiver = manager.subscribe();
        
        // Should be able to create subscription
        assert!(receiver.try_recv().is_err()); // No messages yet
    }
}