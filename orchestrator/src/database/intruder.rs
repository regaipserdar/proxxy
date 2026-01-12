use crate::Database;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderAttack {
    pub id: String,
    pub name: String,
    pub request_template: String, // JSON with §markers§
    pub attack_mode: String, // sniper, battering_ram, pitchfork, cluster_bomb
    pub payload_sets: String, // JSON array of payload configurations
    pub target_agents: String, // JSON array of agent IDs
    pub distribution_strategy: String, // round_robin, batch
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String, // configured, running, completed, stopped
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntruderResult {
    pub id: String,
    pub attack_id: String,
    pub request_data: String, // JSON serialized request with injected payloads
    pub response_data: Option<String>, // JSON serialized response
    pub agent_id: String,
    pub payload_values: String, // JSON array of payload values used
    pub executed_at: i64,
    pub duration_ms: Option<i64>,
    pub status_code: Option<i32>,
    pub response_length: Option<i64>,
    pub is_highlighted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadSet {
    pub id: String,
    pub name: String,
    pub payload_type: String, // wordlist, number_range, custom
    pub configuration: String, // JSON configuration
    pub created_at: i64,
}

/// Buffered result writer for high-volume intruder results
#[derive(Clone)]
pub struct IntruderResultBuffer {
    buffer: Arc<Mutex<VecDeque<IntruderResult>>>,
    pool: Pool<Sqlite>,
    batch_size: usize,
    flush_interval: std::time::Duration,
}

impl IntruderResultBuffer {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            pool,
            batch_size: 1000,
            flush_interval: std::time::Duration::from_secs(5),
        }
    }

    /// Add a result to the buffer
    pub async fn add_result(&self, result: IntruderResult) -> Result<(), sqlx::Error> {
        let mut buffer = self.buffer.lock().await;
        buffer.push_back(result);

        // Flush if buffer is full
        if buffer.len() >= self.batch_size {
            drop(buffer); // Release lock before flush
            self.flush().await?;
        }

        Ok(())
    }

    /// Flush all buffered results to database
    pub async fn flush(&self) -> Result<usize, sqlx::Error> {
        let mut buffer = self.buffer.lock().await;
        if buffer.is_empty() {
            return Ok(0);
        }

        let results: Vec<IntruderResult> = buffer.drain(..).collect();
        let count = results.len();
        drop(buffer); // Release lock

        // Batch insert using transaction
        let mut tx = self.pool.begin().await?;

        for result in results {
            sqlx::query(
                r#"
                INSERT INTO intruder_results (
                    id, attack_id, request_data, response_data, agent_id, 
                    payload_values, executed_at, duration_ms, status_code, 
                    response_length, is_highlighted
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&result.id)
            .bind(&result.attack_id)
            .bind(&result.request_data)
            .bind(&result.response_data)
            .bind(&result.agent_id)
            .bind(&result.payload_values)
            .bind(result.executed_at)
            .bind(result.duration_ms)
            .bind(result.status_code)
            .bind(result.response_length)
            .bind(result.is_highlighted)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(count)
    }

    /// Start periodic flush task
    pub fn start_periodic_flush(&self) -> tokio::task::JoinHandle<()> {
        let buffer = self.buffer.clone();
        let pool = self.pool.clone();
        let interval = self.flush_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                
                let buffer_size = {
                    let buffer = buffer.lock().await;
                    buffer.len()
                };

                if buffer_size > 0 {
                    let flush_buffer = IntruderResultBuffer {
                        buffer: buffer.clone(),
                        pool: pool.clone(),
                        batch_size: 1000,
                        flush_interval: interval,
                    };

                    if let Err(e) = flush_buffer.flush().await {
                        tracing::error!("Failed to flush intruder results: {}", e);
                    }
                }
            }
        })
    }
}

impl Database {
    // ============================================================================
    // INTRUDER ATTACK OPERATIONS
    // ============================================================================

    /// Create a new intruder attack
    pub async fn create_intruder_attack(
        &self,
        name: &str,
        request_template: &str,
        attack_mode: &str,
        payload_sets: &str,
        target_agents: &str,
        distribution_strategy: &str,
    ) -> Result<String, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO intruder_attacks (
                id, name, request_template, attack_mode, payload_sets, 
                target_agents, distribution_strategy, created_at, updated_at, status
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(request_template)
        .bind(attack_mode)
        .bind(payload_sets)
        .bind(target_agents)
        .bind(distribution_strategy)
        .bind(timestamp)
        .bind(timestamp)
        .bind("configured")
        .execute(&pool)
        .await?;

        Ok(id)
    }

    /// Get all intruder attacks
    pub async fn get_intruder_attacks(&self, limit: Option<i64>) -> Result<Vec<IntruderAttack>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let limit = limit.unwrap_or(100);

        let rows = sqlx::query(
            r#"
            SELECT id, name, request_template, attack_mode, payload_sets, 
                   target_agents, distribution_strategy, created_at, updated_at, status
            FROM intruder_attacks 
            ORDER BY created_at DESC 
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&pool)
        .await?;

        let mut attacks = Vec::new();
        for row in rows {
            attacks.push(IntruderAttack {
                id: row.get("id"),
                name: row.get("name"),
                request_template: row.get("request_template"),
                attack_mode: row.get("attack_mode"),
                payload_sets: row.get("payload_sets"),
                target_agents: row.get("target_agents"),
                distribution_strategy: row.get("distribution_strategy"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                status: row.get("status"),
            });
        }

        Ok(attacks)
    }

    /// Get a specific intruder attack by ID
    pub async fn get_intruder_attack(&self, attack_id: &str) -> Result<Option<IntruderAttack>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            r#"
            SELECT id, name, request_template, attack_mode, payload_sets, 
                   target_agents, distribution_strategy, created_at, updated_at, status
            FROM intruder_attacks 
            WHERE id = ?
            "#
        )
        .bind(attack_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(IntruderAttack {
                id: row.get("id"),
                name: row.get("name"),
                request_template: row.get("request_template"),
                attack_mode: row.get("attack_mode"),
                payload_sets: row.get("payload_sets"),
                target_agents: row.get("target_agents"),
                distribution_strategy: row.get("distribution_strategy"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                status: row.get("status"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update intruder attack status
    pub async fn update_intruder_attack_status(
        &self,
        attack_id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            "UPDATE intruder_attacks SET status = ?, updated_at = ? WHERE id = ?"
        )
        .bind(status)
        .bind(timestamp)
        .bind(attack_id)
        .execute(&pool)
        .await?;

        Ok(())
    }

    /// Delete an intruder attack and all its results
    pub async fn delete_intruder_attack(&self, attack_id: &str) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        // Use transaction to ensure consistency
        let mut tx = pool.begin().await?;

        // Delete results first (foreign key constraint)
        sqlx::query("DELETE FROM intruder_results WHERE attack_id = ?")
            .bind(attack_id)
            .execute(&mut *tx)
            .await?;

        // Delete attack
        sqlx::query("DELETE FROM intruder_attacks WHERE id = ?")
            .bind(attack_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // ============================================================================
    // INTRUDER RESULT OPERATIONS
    // ============================================================================

    /// Create a buffered result writer for high-volume inserts
    pub async fn create_intruder_result_buffer(&self) -> Result<IntruderResultBuffer, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        Ok(IntruderResultBuffer::new(pool))
    }

    /// Save a single intruder result (for low-volume operations)
    pub async fn save_intruder_result(
        &self,
        attack_id: &str,
        request_data: &str,
        response_data: Option<&str>,
        agent_id: &str,
        payload_values: &str,
        duration_ms: Option<i64>,
        status_code: Option<i32>,
        response_length: Option<i64>,
        is_highlighted: bool,
    ) -> Result<String, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO intruder_results (
                id, attack_id, request_data, response_data, agent_id, 
                payload_values, executed_at, duration_ms, status_code, 
                response_length, is_highlighted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(attack_id)
        .bind(request_data)
        .bind(response_data)
        .bind(agent_id)
        .bind(payload_values)
        .bind(timestamp)
        .bind(duration_ms)
        .bind(status_code)
        .bind(response_length)
        .bind(is_highlighted)
        .execute(&pool)
        .await?;

        Ok(id)
    }

    /// Get intruder results for an attack
    pub async fn get_intruder_results(
        &self,
        attack_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<IntruderResult>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let limit = limit.unwrap_or(1000);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, attack_id, request_data, response_data, agent_id, 
                   payload_values, executed_at, duration_ms, status_code, 
                   response_length, is_highlighted
            FROM intruder_results 
            WHERE attack_id = ? 
            ORDER BY executed_at DESC 
            LIMIT ? OFFSET ?
            "#
        )
        .bind(attack_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(IntruderResult {
                id: row.get("id"),
                attack_id: row.get("attack_id"),
                request_data: row.get("request_data"),
                response_data: row.get("response_data"),
                agent_id: row.get("agent_id"),
                payload_values: row.get("payload_values"),
                executed_at: row.get("executed_at"),
                duration_ms: row.get("duration_ms"),
                status_code: row.get("status_code"),
                response_length: row.get("response_length"),
                is_highlighted: row.get("is_highlighted"),
            });
        }

        Ok(results)
    }

    /// Get attack statistics
    pub async fn get_intruder_attack_stats(&self, attack_id: &str) -> Result<serde_json::Value, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(serde_json::json!({})),
        };

        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_requests,
                COUNT(CASE WHEN response_data IS NOT NULL THEN 1 END) as completed_requests,
                COUNT(CASE WHEN is_highlighted = true THEN 1 END) as highlighted_results,
                AVG(duration_ms) as avg_duration_ms,
                MIN(executed_at) as started_at,
                MAX(executed_at) as last_result_at
            FROM intruder_results 
            WHERE attack_id = ?
            "#
        )
        .bind(attack_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            Ok(serde_json::json!({
                "total_requests": row.get::<i64, _>("total_requests"),
                "completed_requests": row.get::<i64, _>("completed_requests"),
                "highlighted_results": row.get::<i64, _>("highlighted_results"),
                "avg_duration_ms": row.get::<Option<f64>, _>("avg_duration_ms"),
                "started_at": row.get::<Option<i64>, _>("started_at"),
                "last_result_at": row.get::<Option<i64>, _>("last_result_at")
            }))
        } else {
            Ok(serde_json::json!({
                "total_requests": 0,
                "completed_requests": 0,
                "highlighted_results": 0,
                "avg_duration_ms": null,
                "started_at": null,
                "last_result_at": null
            }))
        }
    }

    // ============================================================================
    // PAYLOAD SET OPERATIONS
    // ============================================================================

    /// Create a new payload set
    pub async fn create_payload_set(
        &self,
        name: &str,
        payload_type: &str,
        configuration: &str,
    ) -> Result<String, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO payload_sets (id, name, type, configuration, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(name)
        .bind(payload_type)
        .bind(configuration)
        .bind(timestamp)
        .execute(&pool)
        .await?;

        Ok(id)
    }

    /// Get all payload sets
    pub async fn get_payload_sets(&self) -> Result<Vec<PayloadSet>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = sqlx::query(
            "SELECT id, name, type, configuration, created_at FROM payload_sets ORDER BY created_at DESC"
        )
        .fetch_all(&pool)
        .await?;

        let mut sets = Vec::new();
        for row in rows {
            sets.push(PayloadSet {
                id: row.get("id"),
                name: row.get("name"),
                payload_type: row.get("type"),
                configuration: row.get("configuration"),
                created_at: row.get("created_at"),
            });
        }

        Ok(sets)
    }

    /// Get a specific payload set by ID
    pub async fn get_payload_set(&self, set_id: &str) -> Result<Option<PayloadSet>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            "SELECT id, name, type, configuration, created_at FROM payload_sets WHERE id = ?"
        )
        .bind(set_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(PayloadSet {
                id: row.get("id"),
                name: row.get("name"),
                payload_type: row.get("type"),
                configuration: row.get("configuration"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a payload set
    pub async fn delete_payload_set(&self, set_id: &str) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        sqlx::query("DELETE FROM payload_sets WHERE id = ?")
            .bind(set_id)
            .execute(&pool)
            .await?;

        Ok(())
    }
}