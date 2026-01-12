use crate::Database;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterTab {
    pub id: String,
    pub name: String,
    pub request_template: String, // JSON serialized HttpRequestData
    pub target_agent_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterExecution {
    pub id: String,
    pub tab_id: String,
    pub request_data: String, // JSON serialized HttpRequestData
    pub response_data: Option<String>, // JSON serialized HttpResponseData
    pub agent_id: String,
    pub executed_at: i64,
    pub duration_ms: Option<i64>,
    pub status_code: Option<i32>,
}

impl Database {
    // ============================================================================
    // REPEATER OPERATIONS
    // ============================================================================

    /// Create a new repeater tab
    pub async fn create_repeater_tab(
        &self,
        name: &str,
        request_template: &str,
        target_agent_id: Option<&str>,
    ) -> Result<String, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO repeater_tabs (id, name, request_template, target_agent_id, created_at, updated_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(request_template)
        .bind(target_agent_id)
        .bind(timestamp)
        .bind(timestamp)
        .bind(true)
        .execute(&pool)
        .await?;

        Ok(id)
    }

    /// Get all active repeater tabs
    pub async fn get_repeater_tabs(&self) -> Result<Vec<RepeaterTab>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = sqlx::query(
            "SELECT id, name, request_template, target_agent_id, created_at, updated_at, is_active 
             FROM repeater_tabs 
             WHERE is_active = true 
             ORDER BY created_at DESC"
        )
        .fetch_all(&pool)
        .await?;

        let mut tabs = Vec::new();
        for row in rows {
            tabs.push(RepeaterTab {
                id: row.get("id"),
                name: row.get("name"),
                request_template: row.get("request_template"),
                target_agent_id: row.get("target_agent_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            });
        }

        Ok(tabs)
    }

    /// Get a specific repeater tab by ID
    pub async fn get_repeater_tab(&self, tab_id: &str) -> Result<Option<RepeaterTab>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            "SELECT id, name, request_template, target_agent_id, created_at, updated_at, is_active 
             FROM repeater_tabs 
             WHERE id = ?"
        )
        .bind(tab_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(RepeaterTab {
                id: row.get("id"),
                name: row.get("name"),
                request_template: row.get("request_template"),
                target_agent_id: row.get("target_agent_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update a repeater tab
    pub async fn update_repeater_tab(
        &self,
        tab_id: &str,
        name: Option<&str>,
        request_template: Option<&str>,
        target_agent_id: Option<Option<&str>>,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let timestamp = chrono::Utc::now().timestamp();

        // Build dynamic query based on provided fields
        let mut query_parts = Vec::new();
        let mut bind_values: Vec<Box<dyn sqlx::Encode<'_, Sqlite> + Send + Sync>> = Vec::new();

        if let Some(name) = name {
            query_parts.push("name = ?");
            bind_values.push(Box::new(name.to_string()));
        }

        if let Some(template) = request_template {
            query_parts.push("request_template = ?");
            bind_values.push(Box::new(template.to_string()));
        }

        if let Some(agent_id) = target_agent_id {
            query_parts.push("target_agent_id = ?");
            bind_values.push(Box::new(agent_id.map(|s| s.to_string())));
        }

        query_parts.push("updated_at = ?");
        bind_values.push(Box::new(timestamp));

        if query_parts.is_empty() {
            return Ok(()); // Nothing to update
        }

        let query_str = format!(
            "UPDATE repeater_tabs SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        
        // This is a simplified approach - in practice, you'd want to use a query builder
        // or handle the dynamic binding more elegantly
        if let Some(name) = name {
            query = query.bind(name);
        }
        if let Some(template) = request_template {
            query = query.bind(template);
        }
        if let Some(agent_id) = target_agent_id {
            query = query.bind(agent_id);
        }
        query = query.bind(timestamp);
        query = query.bind(tab_id);

        query.execute(&pool).await?;
        Ok(())
    }

    /// Delete a repeater tab (soft delete)
    pub async fn delete_repeater_tab(&self, tab_id: &str) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            "UPDATE repeater_tabs SET is_active = false, updated_at = ? WHERE id = ?"
        )
        .bind(timestamp)
        .bind(tab_id)
        .execute(&pool)
        .await?;

        Ok(())
    }

    /// Save a repeater execution result
    pub async fn save_repeater_execution(
        &self,
        tab_id: &str,
        request_data: &str,
        response_data: Option<&str>,
        agent_id: &str,
        duration_ms: Option<i64>,
        status_code: Option<i32>,
    ) -> Result<String, sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO repeater_history (id, tab_id, request_data, response_data, agent_id, executed_at, duration_ms, status_code)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(tab_id)
        .bind(request_data)
        .bind(response_data)
        .bind(agent_id)
        .bind(timestamp)
        .bind(duration_ms)
        .bind(status_code)
        .execute(&pool)
        .await?;

        Ok(id)
    }

    /// Get repeater execution history for a tab
    pub async fn get_repeater_history(
        &self,
        tab_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<RepeaterExecution>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let limit = limit.unwrap_or(100);

        let rows = sqlx::query(
            r#"
            SELECT id, tab_id, request_data, response_data, agent_id, executed_at, duration_ms, status_code
            FROM repeater_history 
            WHERE tab_id = ? 
            ORDER BY executed_at DESC 
            LIMIT ?
            "#
        )
        .bind(tab_id)
        .bind(limit)
        .fetch_all(&pool)
        .await?;

        let mut executions = Vec::new();
        for row in rows {
            executions.push(RepeaterExecution {
                id: row.get("id"),
                tab_id: row.get("tab_id"),
                request_data: row.get("request_data"),
                response_data: row.get("response_data"),
                agent_id: row.get("agent_id"),
                executed_at: row.get("executed_at"),
                duration_ms: row.get("duration_ms"),
                status_code: row.get("status_code"),
            });
        }

        Ok(executions)
    }

    /// Get a specific repeater execution by ID
    pub async fn get_repeater_execution(&self, execution_id: &str) -> Result<Option<RepeaterExecution>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query(
            r#"
            SELECT id, tab_id, request_data, response_data, agent_id, executed_at, duration_ms, status_code
            FROM repeater_history 
            WHERE id = ?
            "#
        )
        .bind(execution_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(RepeaterExecution {
                id: row.get("id"),
                tab_id: row.get("tab_id"),
                request_data: row.get("request_data"),
                response_data: row.get("response_data"),
                agent_id: row.get("agent_id"),
                executed_at: row.get("executed_at"),
                duration_ms: row.get("duration_ms"),
                status_code: row.get("status_code"),
            }))
        } else {
            Ok(None)
        }
    }
}