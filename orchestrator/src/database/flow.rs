//! Database operations for Flow Profiles
//!
//! CRUD operations for storing and retrieving browser flow recordings and executions.

use sqlx::Row;
use tracing::info;

/// Flow profile data as stored in database
#[derive(Debug, Clone)]
pub struct FlowProfileRow {
    pub id: String,
    pub name: String,
    pub flow_type: String,
    pub start_url: String,
    pub steps: String, // JSON
    pub meta: Option<String>, // JSON
    pub created_at: i64,
    pub updated_at: i64,
    pub agent_id: Option<String>,
    pub status: String,
}

/// Flow execution record
#[derive(Debug, Clone)]
pub struct FlowExecutionRow {
    pub id: String,
    pub profile_id: String,
    pub agent_id: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: String,
    pub error_message: Option<String>,
    pub steps_completed: i64,
    pub total_steps: i64,
    pub session_cookies: Option<String>, // JSON
    pub extracted_data: Option<String>, // JSON
}

impl super::Database {
    // ========== Flow Profiles ==========

    /// Save a new flow profile
    pub async fn save_flow_profile(
        &self,
        profile: &FlowProfileRow,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        sqlx::query(
            r#"
            INSERT INTO flow_profiles (
                id, name, flow_type, start_url, steps, meta, 
                created_at, updated_at, agent_id, status
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&profile.id)
        .bind(&profile.name)
        .bind(&profile.flow_type)
        .bind(&profile.start_url)
        .bind(&profile.steps)
        .bind(&profile.meta)
        .bind(profile.created_at)
        .bind(profile.updated_at)
        .bind(&profile.agent_id)
        .bind(&profile.status)
        .execute(&pool)
        .await?;

        info!("✓ Saved flow profile '{}' ({})", profile.name, profile.id);
        Ok(())
    }

    /// Get a flow profile by ID
    pub async fn get_flow_profile(&self, id: &str) -> Result<Option<FlowProfileRow>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            r#"
            SELECT id, name, flow_type, start_url, steps, meta, 
                   created_at, updated_at, agent_id, status
            FROM flow_profiles
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&pool)
        .await?;

        Ok(row.map(|r| FlowProfileRow {
            id: r.get("id"),
            name: r.get("name"),
            flow_type: r.get("flow_type"),
            start_url: r.get("start_url"),
            steps: r.get("steps"),
            meta: r.get("meta"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            agent_id: r.get("agent_id"),
            status: r.get("status"),
        }))
    }

    /// List all flow profiles
    pub async fn list_flow_profiles(
        &self,
        status_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<FlowProfileRow>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = if let Some(status) = status_filter {
            sqlx::query(
                r#"
                SELECT id, name, flow_type, start_url, steps, meta, 
                       created_at, updated_at, agent_id, status
                FROM flow_profiles
                WHERE status = ?
                ORDER BY updated_at DESC
                LIMIT ?
                "#,
            )
            .bind(status)
            .bind(limit)
            .fetch_all(&pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, name, flow_type, start_url, steps, meta, 
                       created_at, updated_at, agent_id, status
                FROM flow_profiles
                ORDER BY updated_at DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(&pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|r| FlowProfileRow {
                id: r.get("id"),
                name: r.get("name"),
                flow_type: r.get("flow_type"),
                start_url: r.get("start_url"),
                steps: r.get("steps"),
                meta: r.get("meta"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                agent_id: r.get("agent_id"),
                status: r.get("status"),
            })
            .collect())
    }

    /// Update a flow profile
    pub async fn update_flow_profile(
        &self,
        id: &str,
        name: Option<&str>,
        steps: Option<&str>,
        meta: Option<&str>,
        status: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let updated_at = chrono::Utc::now().timestamp();

        // Build dynamic update query
        let mut updates = vec!["updated_at = ?".to_string()];
        let mut binds: Vec<String> = vec![updated_at.to_string()];

        if let Some(n) = name {
            updates.push("name = ?".to_string());
            binds.push(n.to_string());
        }
        if let Some(s) = steps {
            updates.push("steps = ?".to_string());
            binds.push(s.to_string());
        }
        if let Some(m) = meta {
            updates.push("meta = ?".to_string());
            binds.push(m.to_string());
        }
        if let Some(st) = status {
            updates.push("status = ?".to_string());
            binds.push(st.to_string());
        }

        let query = format!(
            "UPDATE flow_profiles SET {} WHERE id = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        q = q.bind(updated_at);
        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(s) = steps {
            q = q.bind(s);
        }
        if let Some(m) = meta {
            q = q.bind(m);
        }
        if let Some(st) = status {
            q = q.bind(st);
        }
        q = q.bind(id);

        let result = q.execute(&pool).await?;
        Ok(result.rows_affected() > 0)
    }

    /// Delete a flow profile
    pub async fn delete_flow_profile(&self, id: &str) -> Result<bool, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let result = sqlx::query("DELETE FROM flow_profiles WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await?;

        if result.rows_affected() > 0 {
            info!("✓ Deleted flow profile {}", id);
        }
        Ok(result.rows_affected() > 0)
    }

    // ========== Flow Executions ==========

    /// Start a new flow execution
    pub async fn start_flow_execution(
        &self,
        id: &str,
        profile_id: &str,
        agent_id: &str,
        total_steps: i64,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let started_at = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO flow_executions (
                id, profile_id, agent_id, started_at, status, 
                steps_completed, total_steps
            )
            VALUES (?, ?, ?, ?, 'running', 0, ?)
            "#,
        )
        .bind(id)
        .bind(profile_id)
        .bind(agent_id)
        .bind(started_at)
        .bind(total_steps)
        .execute(&pool)
        .await?;

        info!("✓ Started flow execution {} for profile {}", id, profile_id);
        Ok(())
    }

    /// Update execution progress
    pub async fn update_flow_execution_progress(
        &self,
        id: &str,
        steps_completed: i64,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        sqlx::query("UPDATE flow_executions SET steps_completed = ? WHERE id = ?")
            .bind(steps_completed)
            .bind(id)
            .execute(&pool)
            .await?;

        Ok(())
    }

    /// Complete a flow execution (success or failure)
    pub async fn complete_flow_execution(
        &self,
        id: &str,
        success: bool,
        error_message: Option<&str>,
        steps_completed: i64,
        session_cookies: Option<&str>,
        extracted_data: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let completed_at = chrono::Utc::now().timestamp();
        let status = if success { "success" } else { "failed" };

        sqlx::query(
            r#"
            UPDATE flow_executions SET
                completed_at = ?,
                status = ?,
                error_message = ?,
                steps_completed = ?,
                session_cookies = ?,
                extracted_data = ?
            WHERE id = ?
            "#,
        )
        .bind(completed_at)
        .bind(status)
        .bind(error_message)
        .bind(steps_completed)
        .bind(session_cookies)
        .bind(extracted_data)
        .bind(id)
        .execute(&pool)
        .await?;

        info!("✓ Completed flow execution {} with status: {}", id, status);
        Ok(())
    }

    /// Get recent executions for a profile
    pub async fn get_flow_executions(
        &self,
        profile_id: &str,
        limit: i64,
    ) -> Result<Vec<FlowExecutionRow>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = sqlx::query(
            r#"
            SELECT id, profile_id, agent_id, started_at, completed_at,
                   status, error_message, steps_completed, total_steps,
                   session_cookies, extracted_data
            FROM flow_executions
            WHERE profile_id = ?
            ORDER BY started_at DESC
            LIMIT ?
            "#,
        )
        .bind(profile_id)
        .bind(limit)
        .fetch_all(&pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| FlowExecutionRow {
                id: r.get("id"),
                profile_id: r.get("profile_id"),
                agent_id: r.get("agent_id"),
                started_at: r.get("started_at"),
                completed_at: r.get("completed_at"),
                status: r.get("status"),
                error_message: r.get("error_message"),
                steps_completed: r.get("steps_completed"),
                total_steps: r.get("total_steps"),
                session_cookies: r.get("session_cookies"),
                extracted_data: r.get("extracted_data"),
            })
            .collect())
    }

    /// Get last successful execution for a profile (for session reuse)
    pub async fn get_last_successful_execution(
        &self,
        profile_id: &str,
    ) -> Result<Option<FlowExecutionRow>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            r#"
            SELECT id, profile_id, agent_id, started_at, completed_at,
                   status, error_message, steps_completed, total_steps,
                   session_cookies, extracted_data
            FROM flow_executions
            WHERE profile_id = ? AND status = 'success'
            ORDER BY completed_at DESC
            LIMIT 1
            "#,
        )
        .bind(profile_id)
        .fetch_optional(&pool)
        .await?;

        Ok(row.map(|r| FlowExecutionRow {
            id: r.get("id"),
            profile_id: r.get("profile_id"),
            agent_id: r.get("agent_id"),
            started_at: r.get("started_at"),
            completed_at: r.get("completed_at"),
            status: r.get("status"),
            error_message: r.get("error_message"),
            steps_completed: r.get("steps_completed"),
            total_steps: r.get("total_steps"),
            session_cookies: r.get("session_cookies"),
            extracted_data: r.get("extracted_data"),
        }))
    }

    /// Cleanup orphaned executions (executions that are stuck in 'running' state after a crash/restart)
    pub async fn cleanup_orphaned_executions(&self) -> Result<u64, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let result = sqlx::query(
            "UPDATE flow_executions SET status = 'failed', error_message = 'Orphaned (Orchestrator restarted)' WHERE status = 'running'"
        )
        .execute(&pool)
        .await?;

        let count = result.rows_affected();
        if count > 0 {
            info!("✓ Cleaned up {} orphaned flow executions", count);
        }
        Ok(count)
    }
}
