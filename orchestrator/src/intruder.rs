//! Intruder Manager - Manages distributed attack configurations and execution
//! 
//! This module provides the IntruderManager struct that handles attack configuration,
//! payload set management, agent selection, and attack template creation/validation.

pub mod distribution;
pub mod execution;

use crate::database::intruder::{IntruderAttack, PayloadSet};
use crate::Database;
use crate::session_integration::{SessionManager, SessionApplicationResult, ExpirationHandling, SessionSelectionCriteria, SessionRefreshResult};
use attack_engine::{
    AttackError, AttackResult, PayloadConfig, PayloadGeneratorFactory,
    PayloadPosition, PayloadPositionParser, AttackMode,
    DistributionStrategy, ExecutionConfig, AgentInfo, AgentStatus
};
use distribution::{IntruderPayloadDistributor, DistributionStats};
use execution::{AttackExecutionCoordinator, AttackProgress, AttackExecutionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use proxy_common::Session;

/// Configuration for creating a new intruder attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderAttackConfig {
    pub name: String,
    pub request_template: String, // Raw template with §markers§
    pub attack_mode: AttackMode,
    pub payload_sets: Vec<PayloadSetConfig>,
    pub target_agents: Vec<String>,
    pub distribution_strategy: DistributionStrategy,
    pub session_data: Option<Session>,
    pub execution_config: Option<ExecutionConfig>,
}

/// Configuration for a payload set within an attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadSetConfig {
    pub id: String,
    pub name: String,
    pub payload_config: PayloadConfig,
    pub position_index: usize, // Which §marker§ position this applies to
}

/// Validation result for attack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub payload_positions: Vec<PayloadPosition>,
    pub estimated_requests: Option<usize>,
}

/// Statistics for an attack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackStatistics {
    pub total_payloads: usize,
    pub estimated_requests: usize,
    pub estimated_duration_minutes: Option<f64>,
    pub payload_distribution: HashMap<String, usize>, // agent_id -> payload_count
}

/// Manager for intruder attack operations
pub struct IntruderManager {
    db: Arc<Database>,
    session_manager: Arc<SessionManager>,
    distributor: IntruderPayloadDistributor,
    execution_coordinator: AttackExecutionCoordinator,
}

impl IntruderManager {
    /// Create a new IntruderManager
    pub async fn new(db: Arc<Database>) -> AttackResult<Self> {
        let execution_coordinator = AttackExecutionCoordinator::new(db.clone()).await?;
        
        Ok(Self {
            db,
            session_manager: Arc::new(SessionManager::new()),
            distributor: IntruderPayloadDistributor::new(),
            execution_coordinator,
        })
    }

    /// Create a new intruder attack configuration
    pub async fn create_attack(&self, config: IntruderAttackConfig) -> AttackResult<String> {
        // Validate the attack configuration
        let validation = self.validate_attack_config(&config).await?;
        if !validation.is_valid {
            return Err(AttackError::InvalidPayloadConfig {
                reason: format!("Attack validation failed: {}", validation.errors.join(", ")),
            });
        }

        // Serialize payload sets and target agents
        let payload_sets_json = serde_json::to_string(&config.payload_sets)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize payload sets: {}", e),
            })?;

        let target_agents_json = serde_json::to_string(&config.target_agents)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize target agents: {}", e),
            })?;

        let distribution_strategy_str = match config.distribution_strategy {
            DistributionStrategy::RoundRobin => "round_robin".to_string(),
            DistributionStrategy::Batch { batch_size } => format!("batch:{}", batch_size),
            DistributionStrategy::LoadBalanced => "load_balanced".to_string(),
        };

        let attack_mode_str = match config.attack_mode {
            AttackMode::Sniper => "sniper",
            AttackMode::BatteringRam => "battering_ram",
            AttackMode::Pitchfork => "pitchfork",
            AttackMode::ClusterBomb => "cluster_bomb",
        }.to_string();

        // Create attack in database
        let attack_id = self.db.create_intruder_attack(
            &config.name,
            &config.request_template,
            &attack_mode_str,
            &payload_sets_json,
            &target_agents_json,
            &distribution_strategy_str,
        ).await.map_err(|e| AttackError::DatabaseError {
            operation: format!("create_intruder_attack: {}", e),
        })?;

        Ok(attack_id)
    }

    /// Get an attack configuration by ID
    pub async fn get_attack(&self, attack_id: &str) -> AttackResult<Option<IntruderAttack>> {
        self.db.get_intruder_attack(attack_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_intruder_attack: {}", e),
            })
    }

    /// List all attack configurations
    pub async fn list_attacks(&self, limit: Option<i64>) -> AttackResult<Vec<IntruderAttack>> {
        self.db.get_intruder_attacks(limit)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_intruder_attacks: {}", e),
            })
    }

    /// Update attack status
    pub async fn update_attack_status(&self, attack_id: &str, status: &str) -> AttackResult<()> {
        self.db.update_intruder_attack_status(attack_id, status)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("update_intruder_attack_status: {}", e),
            })
    }

    /// Delete an attack and all its results
    pub async fn delete_attack(&self, attack_id: &str) -> AttackResult<()> {
        self.db.delete_intruder_attack(attack_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("delete_intruder_attack: {}", e),
            })
    }

    /// Validate an attack configuration
    pub async fn validate_attack_config(&self, config: &IntruderAttackConfig) -> AttackResult<AttackValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate attack name
        if config.name.trim().is_empty() {
            errors.push("Attack name cannot be empty".to_string());
        }

        // Validate request template
        if config.request_template.trim().is_empty() {
            errors.push("Request template cannot be empty".to_string());
        }

        // Parse payload positions
        let payload_positions = match PayloadPositionParser::parse(&config.request_template) {
            Ok(parsed) => parsed.positions,
            Err(e) => {
                errors.push(format!("Invalid payload position syntax: {}", e));
                Vec::new()
            }
        };

        // Validate payload positions match payload sets
        if payload_positions.len() != config.payload_sets.len() {
            errors.push(format!(
                "Mismatch between payload positions ({}) and payload sets ({})",
                payload_positions.len(),
                config.payload_sets.len()
            ));
        }

        // Validate each payload set
        let mut total_payloads = 0;
        for (index, payload_set) in config.payload_sets.iter().enumerate() {
            // Validate payload set configuration
            match PayloadGeneratorFactory::create(&payload_set.payload_config) {
                Ok(generator) => {
                    if let Err(e) = generator.validate() {
                        errors.push(format!("Payload set '{}' validation failed: {}", payload_set.name, e));
                    } else {
                        // Get payload count
                        match generator.count().await {
                            Ok(count) => {
                                total_payloads += count;
                                if count == 0 {
                                    warnings.push(format!("Payload set '{}' contains no payloads", payload_set.name));
                                } else if count > 100_000 {
                                    warnings.push(format!("Payload set '{}' contains {} payloads (large set)", payload_set.name, count));
                                }
                            }
                            Err(e) => {
                                errors.push(format!("Failed to count payloads for set '{}': {}", payload_set.name, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Invalid payload configuration for set '{}': {}", payload_set.name, e));
                }
            }

            // Validate position index
            if payload_set.position_index >= payload_positions.len() {
                errors.push(format!(
                    "Payload set '{}' position index {} exceeds available positions ({})",
                    payload_set.name, payload_set.position_index, payload_positions.len()
                ));
            }
        }

        // Validate target agents
        if config.target_agents.is_empty() {
            errors.push("At least one target agent must be specified".to_string());
        }

        // Estimate total requests based on attack mode
        let estimated_requests = if !config.payload_sets.is_empty() && errors.is_empty() {
            self.estimate_request_count(&config.attack_mode, &config.payload_sets).await.ok()
        } else {
            None
        };

        // Warn about large attacks
        if let Some(count) = estimated_requests {
            if count > 10_000 {
                warnings.push(format!("Attack will generate {} requests (large attack)", count));
            }
        }

        Ok(AttackValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            payload_positions,
            estimated_requests,
        })
    }

    /// Estimate the number of requests an attack will generate
    async fn estimate_request_count(&self, attack_mode: &AttackMode, payload_sets: &[PayloadSetConfig]) -> AttackResult<usize> {
        let mut payload_counts = Vec::new();
        
        for payload_set in payload_sets {
            let generator = PayloadGeneratorFactory::create(&payload_set.payload_config)?;
            let count = generator.count().await?;
            payload_counts.push(count);
        }

        let total_requests = match attack_mode {
            AttackMode::Sniper => {
                // Single position, iterate through all payloads
                payload_counts.iter().sum()
            }
            AttackMode::BatteringRam => {
                // Multiple positions, same payload in all - use max count
                payload_counts.iter().max().copied().unwrap_or(0)
            }
            AttackMode::Pitchfork => {
                // Multiple positions, parallel iteration - use max count
                payload_counts.iter().max().copied().unwrap_or(0)
            }
            AttackMode::ClusterBomb => {
                // Multiple positions, all combinations - multiply all counts
                payload_counts.iter().product()
            }
        };

        Ok(total_requests)
    }

    /// Get attack statistics
    pub async fn get_attack_statistics(&self, attack_id: &str) -> AttackResult<serde_json::Value> {
        self.db.get_intruder_attack_stats(attack_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_intruder_attack_stats: {}", e),
            })
    }

    /// Validate agent selection for distributed attacks
    pub async fn validate_agent_selection(&self, agent_ids: &[String], available_agents: &[AgentInfo]) -> AttackResult<Vec<String>> {
        let mut errors = Vec::new();
        let available_agent_ids: HashMap<String, &AgentInfo> = available_agents
            .iter()
            .map(|agent| (agent.id.clone(), agent))
            .collect();

        for agent_id in agent_ids {
            match available_agent_ids.get(agent_id) {
                Some(agent) => {
                    if agent.status != AgentStatus::Online {
                        errors.push(format!("Agent '{}' is not online (status: {:?})", agent_id, agent.status));
                    }
                }
                None => {
                    errors.push(format!("Agent '{}' not found", agent_id));
                }
            }
        }

        if errors.is_empty() {
            Ok(Vec::new())
        } else {
            Err(AttackError::AgentUnavailable {
                agent_id: format!("Multiple agents: {}", errors.join(", ")),
            })
        }
    }

    /// Create attack template from HTTP request data
    pub fn create_attack_template(&self, request_data: &str, payload_positions: &[String]) -> AttackResult<String> {
        let mut template = request_data.to_string();
        
        // Replace specified values with §marker§ syntax
        for (index, value) in payload_positions.iter().enumerate() {
            if !value.is_empty() {
                let marker = format!("§payload{}§", index + 1);
                template = template.replace(value, &marker);
            }
        }

        // Validate that markers were inserted
        if !template.contains('§') {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "No payload positions were marked in the template".to_string(),
            });
        }

        Ok(template)
    }

    /// Parse attack template and extract payload positions
    pub fn parse_attack_template(&self, template: &str) -> AttackResult<Vec<PayloadPosition>> {
        let parsed = PayloadPositionParser::parse(template)?;
        Ok(parsed.positions)
    }

    // ============================================================================
    // SESSION MANAGEMENT METHODS
    // ============================================================================

    /// Add or update a session for use in intruder attacks
    pub async fn add_session(&self, session: Session) -> AttackResult<()> {
        self.session_manager.add_session(session).await
    }

    /// Get available sessions
    pub async fn get_sessions(&self) -> Vec<Session> {
        self.session_manager.get_sessions().await
    }

    /// Get active sessions only
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        self.session_manager.get_active_sessions().await
    }

    /// Get a specific session by ID
    pub async fn get_session(&self, session_id: &Uuid) -> Option<Session> {
        self.session_manager.get_session(session_id).await
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &Uuid) -> AttackResult<()> {
        self.session_manager.remove_session(session_id).await
    }

    /// Select best session based on criteria
    pub async fn select_session(&self, criteria: &SessionSelectionCriteria) -> Option<Session> {
        self.session_manager.select_session(criteria).await
    }

    /// Validate session against target URL
    pub async fn validate_session(&self, session_id: &Uuid, validation_url: &str) -> AttackResult<bool> {
        self.session_manager.validate_session(session_id, validation_url).await
    }

    /// Detect authentication failure from response
    pub async fn detect_authentication_failure(
        &self,
        response: &attack_engine::HttpResponseData,
        request_url: &str,
    ) -> bool {
        self.session_manager.detect_authentication_failure(response, request_url).await
    }

    /// Handle authentication failure and attempt refresh
    pub async fn handle_authentication_failure(
        &self,
        session_id: &Uuid,
        failure_url: &str,
        response: &attack_engine::HttpResponseData,
    ) -> AttackResult<SessionRefreshResult> {
        self.session_manager.handle_authentication_failure(session_id, failure_url, response).await
    }

    /// Refresh session manually
    pub async fn refresh_session_manually(
        &self,
        session_id: &Uuid,
        new_session_data: Session,
    ) -> AttackResult<SessionRefreshResult> {
        self.session_manager.refresh_session_manually(session_id, new_session_data).await
    }

    /// Apply session data to attack request template
    pub async fn apply_session_to_attack_template(
        &self,
        request_template: &str,
        session_id: &Uuid,
        expiration_handling: Option<ExpirationHandling>,
    ) -> AttackResult<(String, SessionApplicationResult)> {
        // Parse request template to HttpRequestData
        // This is a simplified implementation - in practice, you'd need proper HTTP parsing
        let request = attack_engine::HttpRequestData {
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            headers: None,
            body: Vec::new(),
            tls: None,
        };

        // TODO: Implement proper HTTP request parsing from template string
        // For now, we'll work with the template as-is and apply session headers

        let handling = expiration_handling.unwrap_or(ExpirationHandling::Fail);
        let (_modified_request, session_result) = self.session_manager
            .apply_session_to_request(request, session_id, handling).await?;

        // TODO: Convert back to template string format
        // For now, return the original template with session info
        Ok((request_template.to_string(), session_result))
    }

    // ============================================================================
    // PAYLOAD SET MANAGEMENT
    // ============================================================================

    /// Create a new payload set
    pub async fn create_payload_set(&self, name: &str, payload_config: &PayloadConfig) -> AttackResult<String> {
        // Validate payload configuration
        let generator = PayloadGeneratorFactory::create(payload_config)?;
        generator.validate()?;

        let payload_type = match payload_config {
            PayloadConfig::Wordlist { .. } => "wordlist",
            PayloadConfig::NumberRange { .. } => "number_range",
            PayloadConfig::Custom { .. } => "custom",
        };

        let config_json = serde_json::to_string(payload_config)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to serialize payload config: {}", e),
            })?;

        self.db.create_payload_set(name, payload_type, &config_json)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("create_payload_set: {}", e),
            })
    }

    /// Get all payload sets
    pub async fn list_payload_sets(&self) -> AttackResult<Vec<PayloadSet>> {
        self.db.get_payload_sets()
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_payload_sets: {}", e),
            })
    }

    /// Get a specific payload set
    pub async fn get_payload_set(&self, set_id: &str) -> AttackResult<Option<PayloadSet>> {
        self.db.get_payload_set(set_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("get_payload_set: {}", e),
            })
    }

    /// Delete a payload set
    pub async fn delete_payload_set(&self, set_id: &str) -> AttackResult<()> {
        self.db.delete_payload_set(set_id)
            .await
            .map_err(|e| AttackError::DatabaseError {
                operation: format!("delete_payload_set: {}", e),
            })
    }

    /// Preview payloads from a configuration (limited to first 100)
    pub async fn preview_payloads(&self, payload_config: &PayloadConfig) -> AttackResult<Vec<String>> {
        let generator = PayloadGeneratorFactory::create(payload_config)?;
        let mut payloads = generator.generate().await?;
        
        // Limit preview to first 100 payloads
        if payloads.len() > 100 {
            payloads.truncate(100);
        }
        
        Ok(payloads)
    }

    // ============================================================================
    // PAYLOAD DISTRIBUTION METHODS
    // ============================================================================

    /// Distribute payloads across agents using the specified strategy
    pub async fn distribute_payloads(
        &self,
        payloads: Vec<String>,
        available_agents: &[AgentInfo],
        strategy: &DistributionStrategy,
    ) -> AttackResult<DistributionStats> {
        self.distributor.distribute_payloads(payloads, available_agents, strategy).await
    }

    /// Update agent load information for load balancing
    pub async fn update_agent_load(&self, agent_id: &str, load: distribution::AgentLoad) {
        self.distributor.update_agent_load(agent_id, load).await;
    }

    /// Record agent failure for reliability tracking
    pub async fn record_agent_failure(&self, agent_id: &str) {
        self.distributor.record_agent_failure(agent_id).await;
    }

    /// Redistribute payloads when an agent fails during attack execution
    pub async fn redistribute_on_failure(
        &self,
        failed_agent_id: &str,
        original_distribution: &DistributionStats,
        available_agents: &[AgentInfo],
    ) -> AttackResult<DistributionStats> {
        self.distributor.redistribute_on_failure(
            failed_agent_id,
            original_distribution,
            available_agents,
        ).await
    }

    /// Get current agent load statistics
    pub async fn get_agent_loads(&self) -> HashMap<String, distribution::AgentLoad> {
        self.distributor.get_agent_loads().await
    }

    /// Get agent failure history for monitoring
    pub async fn get_failure_history(&self) -> HashMap<String, Vec<chrono::DateTime<chrono::Utc>>> {
        self.distributor.get_failure_history().await
    }

    // ============================================================================
    // ATTACK EXECUTION COORDINATION METHODS
    // ============================================================================

    /// Start executing an attack
    pub async fn start_attack_execution(
        &self,
        config: AttackExecutionConfig,
        available_agents: &[AgentInfo],
    ) -> AttackResult<()> {
        self.execution_coordinator.start_attack(config, available_agents).await
    }

    /// Stop an active attack
    pub async fn stop_attack_execution(&self, attack_id: &str) -> AttackResult<()> {
        self.execution_coordinator.stop_attack(attack_id).await
    }

    /// Pause an active attack
    pub async fn pause_attack_execution(&self, attack_id: &str) -> AttackResult<()> {
        self.execution_coordinator.pause_attack(attack_id).await
    }

    /// Resume a paused attack
    pub async fn resume_attack_execution(&self, attack_id: &str) -> AttackResult<()> {
        self.execution_coordinator.resume_attack(attack_id).await
    }

    /// Get current progress for an attack
    pub async fn get_attack_progress(&self, attack_id: &str) -> Option<AttackProgress> {
        self.execution_coordinator.get_attack_progress(attack_id).await
    }

    /// Get all active attacks
    pub async fn get_active_attacks(&self) -> Vec<AttackProgress> {
        self.execution_coordinator.get_active_attacks().await
    }

    /// Subscribe to real-time progress updates
    pub fn subscribe_to_progress_updates(&self) -> tokio::sync::broadcast::Receiver<AttackProgress> {
        self.execution_coordinator.subscribe_to_progress()
    }

    /// Create an execution configuration from attack configuration
    pub async fn create_execution_config(
        &self,
        attack: &IntruderAttack,
        available_agents: &[AgentInfo],
    ) -> AttackResult<AttackExecutionConfig> {
        // Parse attack mode
        let attack_mode = match attack.attack_mode.as_str() {
            "sniper" => AttackMode::Sniper,
            "battering_ram" => AttackMode::BatteringRam,
            "pitchfork" => AttackMode::Pitchfork,
            "cluster_bomb" => AttackMode::ClusterBomb,
            _ => return Err(AttackError::InvalidPayloadConfig {
                reason: format!("Unknown attack mode: {}", attack.attack_mode),
            }),
        };

        // Parse distribution strategy
        let distribution_strategy = if attack.distribution_strategy.starts_with("batch:") {
            let batch_size = attack.distribution_strategy
                .strip_prefix("batch:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);
            DistributionStrategy::Batch { batch_size }
        } else {
            match attack.distribution_strategy.as_str() {
                "round_robin" => DistributionStrategy::RoundRobin,
                "load_balanced" => DistributionStrategy::LoadBalanced,
                _ => DistributionStrategy::RoundRobin,
            }
        };

        // Parse payload sets
        let payload_sets: Vec<PayloadSetConfig> = serde_json::from_str(&attack.payload_sets)
            .map_err(|e| AttackError::InvalidPayloadConfig {
                reason: format!("Failed to parse payload sets: {}", e),
            })?;

        // Generate payloads for distribution
        let mut all_payloads = Vec::new();
        for payload_set in &payload_sets {
            let generator = PayloadGeneratorFactory::create(&payload_set.payload_config)?;
            let payloads = generator.generate().await?;
            all_payloads.extend(payloads);
        }

        // Create distribution
        let distribution = self.distribute_payloads(
            all_payloads,
            available_agents,
            &distribution_strategy,
        ).await?;

        // Create execution config
        Ok(AttackExecutionConfig {
            attack_id: attack.id.clone(),
            request_template: attack.request_template.clone(),
            attack_mode,
            distribution,
            session_data: None, // TODO: Load session data if specified
            concurrent_requests_per_agent: 10, // Default value
            timeout_seconds: 30, // Default value
            retry_attempts: 3, // Default value
            result_highlighting_rules: Vec::new(), // TODO: Load highlighting rules
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_manager() -> (IntruderManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(Database::new(temp_dir.path().to_str().unwrap()).await.unwrap());
        let manager = IntruderManager::new(db).await.unwrap();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_create_attack_template() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let request_data = "GET /api/user?id=123&name=admin HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let payload_positions = vec!["123".to_string(), "admin".to_string()];
        
        let template = manager.create_attack_template(request_data, &payload_positions).unwrap();
        
        assert!(template.contains("§payload1§"));
        assert!(template.contains("§payload2§"));
        assert!(!template.contains("123"));
        assert!(!template.contains("admin"));
    }

    #[tokio::test]
    async fn test_validate_attack_config() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let config = IntruderAttackConfig {
            name: "Test Attack".to_string(),
            request_template: "GET /api/test?param=§payload1§ HTTP/1.1\r\n\r\n".to_string(),
            attack_mode: AttackMode::Sniper,
            payload_sets: vec![PayloadSetConfig {
                id: "test-set".to_string(),
                name: "Test Set".to_string(),
                payload_config: PayloadConfig::Custom {
                    values: vec!["test1".to_string(), "test2".to_string()],
                },
                position_index: 0,
            }],
            target_agents: vec!["agent1".to_string()],
            distribution_strategy: DistributionStrategy::RoundRobin,
            session_data: None,
            execution_config: None,
        };
        
        let validation = manager.validate_attack_config(&config).await.unwrap();
        assert!(validation.is_valid, "Validation errors: {:?}", validation.errors);
        assert_eq!(validation.payload_positions.len(), 1);
        assert_eq!(validation.estimated_requests, Some(2));
    }

    #[tokio::test]
    async fn test_estimate_request_count() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let payload_sets = vec![
            PayloadSetConfig {
                id: "set1".to_string(),
                name: "Set 1".to_string(),
                payload_config: PayloadConfig::Custom {
                    values: vec!["a".to_string(), "b".to_string()],
                },
                position_index: 0,
            },
            PayloadSetConfig {
                id: "set2".to_string(),
                name: "Set 2".to_string(),
                payload_config: PayloadConfig::Custom {
                    values: vec!["1".to_string(), "2".to_string(), "3".to_string()],
                },
                position_index: 1,
            },
        ];
        
        // Test different attack modes
        let sniper_count = manager.estimate_request_count(&AttackMode::Sniper, &payload_sets).await.unwrap();
        assert_eq!(sniper_count, 5); // 2 + 3
        
        let battering_ram_count = manager.estimate_request_count(&AttackMode::BatteringRam, &payload_sets).await.unwrap();
        assert_eq!(battering_ram_count, 3); // max(2, 3)
        
        let pitchfork_count = manager.estimate_request_count(&AttackMode::Pitchfork, &payload_sets).await.unwrap();
        assert_eq!(pitchfork_count, 3); // max(2, 3)
        
        let cluster_bomb_count = manager.estimate_request_count(&AttackMode::ClusterBomb, &payload_sets).await.unwrap();
        assert_eq!(cluster_bomb_count, 6); // 2 * 3
    }

    #[tokio::test]
    async fn test_validate_agent_selection() {
        let (manager, _temp_dir) = create_test_manager().await;
        
        let available_agents = vec![
            AgentInfo {
                id: "agent1".to_string(),
                hostname: "host1".to_string(),
                status: AgentStatus::Online,
                load: 0.5,
                response_time_ms: Some(100),
            },
            AgentInfo {
                id: "agent2".to_string(),
                hostname: "host2".to_string(),
                status: AgentStatus::Offline,
                load: 0.0,
                response_time_ms: None,
            },
        ];
        
        // Test valid selection
        let result = manager.validate_agent_selection(&["agent1".to_string()], &available_agents).await;
        assert!(result.is_ok());
        
        // Test offline agent
        let result = manager.validate_agent_selection(&["agent2".to_string()], &available_agents).await;
        assert!(result.is_err());
        
        // Test non-existent agent
        let result = manager.validate_agent_selection(&["agent3".to_string()], &available_agents).await;
        assert!(result.is_err());
    }
}