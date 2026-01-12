use crate::pb::{traffic_event, SystemMetricsEvent, TrafficEvent};
use crate::models::settings::{ScopeConfig, InterceptionConfig};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::path::{PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use std::fs;
use serde::{Serialize, de::DeserializeOwned};

// Include database modules for repeater and intruder
pub mod repeater;
pub mod intruder;

pub use repeater::*;
pub use intruder::*;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub last_modified: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: Arc<RwLock<Option<Pool<Sqlite>>>>,
    projects_dir: PathBuf,
    active_project: Arc<RwLock<Option<String>>>,
}

impl Database {
    pub async fn new(projects_dir: &str) -> Result<Self, std::io::Error> {
        let path = PathBuf::from(projects_dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        Ok(Self {
            pool: Arc::new(RwLock::new(None)),
            projects_dir: path,
            active_project: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, std::io::Error> {
        let mut projects = Vec::new();
        let active = self.active_project.read().await.clone();
        
        // Read directory entries
        let mut entries = fs::read_dir(&self.projects_dir)?;
        
        while let Some(entry) = entries.next() {
            let entry = entry?;
            let path = entry.path();
            
            // Strategy: Each folder in 'projects_dir' ending in '.proxxy' is a project.
            if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
                if !dir_name.ends_with(".proxxy") {
                    continue;
                }

                let name = dir_name.trim_end_matches(".proxxy").to_string();
                let db_path = path.join("proxxy.db");
                
                let metadata = fs::metadata(&path)?;
                let last_modified: chrono::DateTime<chrono::Utc> = metadata.modified()?.into();
                
                // Calculate size (simplified: size of db file if exists)
                let size_bytes = if db_path.exists() {
                     fs::metadata(&db_path)?.len() as i64
                } else {
                    0
                };

                projects.push(Project {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size_bytes,
                    last_modified: last_modified.to_rfc3339(),
                    is_active: active.as_ref() == Some(&dir_name.trim_end_matches(".proxxy").to_string()),
                });
            }
        }
        
        // Sort by last modified DESC
        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        
        Ok(projects)
    }

    pub async fn create_project(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Validate name (alphanumeric, -, _)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Invalid project name. Use only alphanumeric, -, and _".into());
        }

        let folder_name = format!("{}.proxxy", name);
        let project_path = self.projects_dir.join(folder_name);
        if !project_path.exists() {
            fs::create_dir_all(&project_path)?;
        }
        
        // Initialize DB immediately? Or wait for load?
        // Let's just create the folder. Load will handle DB init.
        
        Ok(())
    }

    pub async fn load_project(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
         let folder_name = format!("{}.proxxy", name);
         let project_path = self.projects_dir.join(folder_name);
         if !project_path.exists() {
             return Err("Project does not exist".into());
         }

         let db_path = project_path.join("proxxy.db");
         let db_url = format!("sqlite:{}", db_path.to_string_lossy());

         use sqlx::sqlite::SqliteConnectOptions;
         use std::str::FromStr;

         // Initialize connection
         let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
         let pool = SqlitePoolOptions::new()
             .max_connections(5)
             .connect_with(options)
             .await?;
         
         // Run migrations
         sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
         sqlx::query("PRAGMA synchronous=NORMAL").execute(&pool).await?;
         sqlx::migrate!("./migrations").run(&pool).await?;

         // Update state
         let mut pool_guard = self.pool.write().await;
         *pool_guard = Some(pool);
         
         let mut active_guard = self.active_project.write().await;
         *active_guard = Some(name.to_string());

         info!("✓ Loaded project '{}' from {}", name, db_path.display());
         Ok(())
    }

    pub async fn unload_project(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut pool_guard = self.pool.write().await;
        *pool_guard = None;
        
        let mut active_guard = self.active_project.write().await;
        *active_guard = None;
        
        info!("✓ Project unloaded");
        Ok(())
    }

    pub async fn delete_project(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let active = self.active_project.read().await.clone();
        if active.as_deref() == Some(name) {
            self.unload_project().await?;
        }

        let folder_name = format!("{}.proxxy", name);
        let project_path = self.projects_dir.join(folder_name);
        if project_path.exists() {
            fs::remove_dir_all(&project_path)?;
            info!("✓ Deleted project '{}' and its directory", name);
        }

        Ok(())
    }

    pub async fn get_pool(&self) -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
        let guard = self.pool.read().await;
        if let Some(pool) = guard.as_ref() {
            Ok(pool.clone())
        } else {
            Err("No active project loaded".into())
        }
    }

    pub async fn pool(&self) -> Option<Pool<Sqlite>> {
        self.pool.read().await.clone()
    }

    pub async fn upsert_agent(
        &self,
        id: &str,
        name: &str,
        hostname: &str,
        version: &str,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let timestamp = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO agents (id, name, hostname, version, status, last_heartbeat)
            VALUES (?, ?, ?, ?, 'Online', ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                hostname = excluded.hostname,
                version = excluded.version,
                status = 'Online',
                last_heartbeat = excluded.last_heartbeat
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(hostname)
        .bind(version)
        .bind(timestamp)
        .execute(&pool)
        .await?;
        Ok(())
    }

    pub async fn get_agent_name(&self, agent_id: &str) -> Result<Option<String>, sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(None),
        };
        let row = sqlx::query("SELECT name FROM agents WHERE id = ?")
            .bind(agent_id)
            .fetch_optional(&pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    pub async fn mark_agent_offline(&self, agent_id: &str) -> Result<(), sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(()),
        };
        let timestamp = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            UPDATE agents 
            SET status = 'Offline', last_heartbeat = ?
            WHERE id = ?
            "#,
        )
        .bind(timestamp)
        .bind(agent_id)
        .execute(&pool)
        .await?;
        Ok(())
    }

    pub async fn save_request(
        &self,
        event: &TrafficEvent,
        agent_id: &str,
    ) -> Result<(), sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(()),
        };

        match &event.event {
            Some(traffic_event::Event::Request(req)) => {
                let headers_json = serde_json::to_string(&req.headers).unwrap_or_default();
                let tls_json = serde_json::to_string(&req.tls).unwrap_or_default();
                let timestamp = chrono::Utc::now().timestamp();

                sqlx::query(
                    r#"
                    INSERT INTO http_transactions (
                        request_id, agent_id, req_method, req_url, req_headers, req_body, req_timestamp, tls_info
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(&event.request_id)
                .bind(agent_id)
                .bind(&req.method)
                .bind(&req.url)
                .bind(headers_json)
                .bind(&req.body)
                .bind(timestamp)
                .bind(tls_json)
                .execute(&pool)
                .await?;
            }
            Some(traffic_event::Event::Response(res)) => {
                let headers_json = serde_json::to_string(&res.headers).unwrap_or_default();
                let timestamp = chrono::Utc::now().timestamp();

                sqlx::query(
                    r#"
                    UPDATE http_transactions SET
                        res_status = ?,
                        res_headers = ?,
                        res_body = ?,
                        res_timestamp = ?
                    WHERE request_id = ?
                    "#,
                )
                .bind(res.status_code)
                .bind(headers_json)
                .bind(&res.body)
                .bind(timestamp)
                .bind(&event.request_id)
                .execute(&pool)
                .await?;
            }
            _ => {
                // Ignore other events for DB (WebSocket, etc. for now)
            }
        }
        Ok(())
    }

    pub async fn get_recent_requests(&self, agent_id: Option<&str>, limit: i64) -> Result<Vec<(String, TrafficEvent)>, sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(Vec::new()),
        };

        // This query needs to adapt to http_transactions.
        // It's tricky because we merged req/res into one row.
        // We'll reconstruct TrafficEvents. This might return incomplete events if response is missing?
        // Or we just return the Request part for list view?
        // For simplicity, let's return TrafficEvents as Requests, and maybe we need a separate query/struct for full method.
        // But the current UI expects TrafficEvent.

        // Let's modify the query to return Request events.

        let query = if let Some(aid) = agent_id {
            sqlx::query(
                "SELECT request_id, agent_id, req_method, req_url, req_headers, req_body, tls_info, res_status, res_headers, res_body FROM http_transactions WHERE agent_id = ? ORDER BY req_timestamp DESC LIMIT ?"
            )
            .bind(aid)
            .bind(limit)
        } else {
            sqlx::query(
                "SELECT request_id, agent_id, req_method, req_url, req_headers, req_body, tls_info, res_status, res_headers, res_body FROM http_transactions ORDER BY req_timestamp DESC LIMIT ?"
            )
            .bind(limit)
        };

        let rows = query.fetch_all(&pool).await?;

        let mut results = Vec::new();
        for row in rows {
            let request_id: String = row.get("request_id");
            let agent_id: String = row.get("agent_id");
            let method: String = row.get("req_method");
            let url: String = row.get("req_url");
            let headers_json: String = row.get("req_headers");
            let body: Vec<u8> = row.get("req_body");
            let tls_json: String = row.get("tls_info");

            let headers: Option<crate::pb::HttpHeaders> = serde_json::from_str(&headers_json).ok();
            let tls: Option<crate::pb::TlsDetails> = serde_json::from_str(&tls_json).ok();

            results.push((agent_id, TrafficEvent {
                request_id,
                event: Some(traffic_event::Event::Request(crate::pb::HttpRequestData {
                    method,
                    url,
                    headers,
                    body,
                    tls,
                })),
            }));
        }
        Ok(results)
    }

    pub async fn get_request_by_id(
        &self,
        request_id: &str,
    ) -> Result<Option<(String, crate::pb::HttpRequestData)>, sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(None),
        };
        let row = sqlx::query(
            "SELECT req_method, req_url, req_headers, req_body, tls_info, agent_id FROM http_transactions WHERE request_id = ?"
        )
        .bind(request_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(row) = row {
            let method: String = row.get("req_method");
            let url: String = row.get("req_url");
            let headers_json: String = row.get("req_headers");
            let body: Vec<u8> = row.get("req_body");
            let tls_json: String = row.get("tls_info");
            let agent_id: String = row.get("agent_id");

            let headers: Option<crate::pb::HttpHeaders> = serde_json::from_str(&headers_json).ok();
            let tls: Option<crate::pb::TlsDetails> = serde_json::from_str(&tls_json).ok();

            Ok(Some((agent_id, crate::pb::HttpRequestData {
                method,
                url,
                headers,
                body,
                tls,
            })))
        } else {
            Ok(None)
        }
    }

    pub async fn get_agent_id_for_request(
        &self,
        request_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let pool = match self.get_pool().await {
             Ok(p) => p,
             Err(_) => return Ok(None),
        };
        let row = sqlx::query("SELECT agent_id FROM http_transactions WHERE request_id = ?")
            .bind(request_id)
            .fetch_optional(&pool)
            .await?;

        Ok(row.map(|r| r.get("agent_id")))
    }

    pub async fn save_orchestrator_metrics(
        &self,
        metrics_event: &SystemMetricsEvent,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if let Some(metrics) = &metrics_event.metrics {
            let network_rx = metrics.network.as_ref().map(|n| n.rx_bytes_total as i64).unwrap_or(0);
            let network_tx = metrics.network.as_ref().map(|n| n.tx_bytes_total as i64).unwrap_or(0);
            let network_rx_rate = metrics.network.as_ref().map(|n| n.rx_bytes_per_sec as i64).unwrap_or(0);
            let network_tx_rate = metrics.network.as_ref().map(|n| n.tx_bytes_per_sec as i64).unwrap_or(0);

            let disk_read = metrics.disk.as_ref().map(|d| d.read_bytes_total as i64).unwrap_or(0);
            let disk_write = metrics.disk.as_ref().map(|d| d.write_bytes_total as i64).unwrap_or(0);
            let disk_read_rate = metrics.disk.as_ref().map(|d| d.read_bytes_per_sec as i64).unwrap_or(0);
            let disk_write_rate = metrics.disk.as_ref().map(|d| d.write_bytes_per_sec as i64).unwrap_or(0);
            let disk_available = metrics.disk.as_ref().map(|d| d.available_bytes as i64).unwrap_or(0);
            let disk_total = metrics.disk.as_ref().map(|d| d.total_bytes as i64).unwrap_or(0);

            let process_cpu = metrics.process.as_ref().map(|p| p.cpu_usage_percent).unwrap_or(0.0);
            let process_memory = metrics.process.as_ref().map(|p| p.memory_bytes as i64).unwrap_or(0);
            let process_uptime = metrics.process.as_ref().map(|p| p.uptime_seconds as i64).unwrap_or(0);
            let process_threads = metrics.process.as_ref().map(|p| p.thread_count as i64).unwrap_or(0);
            let process_fds = metrics.process.as_ref().map(|p| p.file_descriptor_count as i64).unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO orchestrator_metrics (
                    timestamp, cpu_usage_percent, memory_used_bytes, memory_total_bytes,
                    network_rx_bytes, network_tx_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec,
                    disk_read_bytes, disk_write_bytes, disk_read_bytes_per_sec, disk_write_bytes_per_sec,
                    disk_available_bytes, disk_total_bytes,
                    process_cpu_percent, process_memory_bytes, process_uptime_seconds,
                    process_thread_count, process_fd_count
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(metrics_event.timestamp)
            .bind(metrics.cpu_usage_percent)
            .bind(metrics.memory_used_bytes as i64)
            .bind(metrics.memory_total_bytes as i64)
            .bind(network_rx)
            .bind(network_tx)
            .bind(network_rx_rate)
            .bind(network_tx_rate)
            .bind(disk_read)
            .bind(disk_write)
            .bind(disk_read_rate)
            .bind(disk_write_rate)
            .bind(disk_available)
            .bind(disk_total)
            .bind(process_cpu)
            .bind(process_memory)
            .bind(process_uptime)
            .bind(process_threads)
            .bind(process_fds)
            .execute(&pool)
            .await?;
        }
        Ok(())
    }
    pub async fn save_system_metrics(
        &self,
        metrics_event: &SystemMetricsEvent,
    ) -> Result<(), sqlx::Error> {
        let pool = self.get_pool().await.map_err(|e| sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        if let Some(metrics) = &metrics_event.metrics {
            let network_rx = metrics.network.as_ref().map(|n| n.rx_bytes_total as i64).unwrap_or(0);
            let network_tx = metrics.network.as_ref().map(|n| n.tx_bytes_total as i64).unwrap_or(0);
            let network_rx_rate = metrics.network.as_ref().map(|n| n.rx_bytes_per_sec as i64).unwrap_or(0);
            let network_tx_rate = metrics.network.as_ref().map(|n| n.tx_bytes_per_sec as i64).unwrap_or(0);

            let disk_read = metrics.disk.as_ref().map(|d| d.read_bytes_total as i64).unwrap_or(0);
            let disk_write = metrics.disk.as_ref().map(|d| d.write_bytes_total as i64).unwrap_or(0);
            let disk_read_rate = metrics.disk.as_ref().map(|d| d.read_bytes_per_sec as i64).unwrap_or(0);
            let disk_write_rate = metrics.disk.as_ref().map(|d| d.write_bytes_per_sec as i64).unwrap_or(0);
            let disk_available = metrics.disk.as_ref().map(|d| d.available_bytes as i64).unwrap_or(0);
            let disk_total = metrics.disk.as_ref().map(|d| d.total_bytes as i64).unwrap_or(0);

            let process_cpu = metrics.process.as_ref().map(|p| p.cpu_usage_percent).unwrap_or(0.0);
            let process_memory = metrics.process.as_ref().map(|p| p.memory_bytes as i64).unwrap_or(0);
            let process_uptime = metrics.process.as_ref().map(|p| p.uptime_seconds as i64).unwrap_or(0);
            let process_threads = metrics.process.as_ref().map(|p| p.thread_count as i64).unwrap_or(0);
            let process_fds = metrics.process.as_ref().map(|p| p.file_descriptor_count as i64).unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO system_metrics (
                    agent_id, timestamp, cpu_usage_percent, memory_used_bytes, memory_total_bytes,
                    network_rx_bytes, network_tx_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec,
                    disk_read_bytes, disk_write_bytes, disk_read_bytes_per_sec, disk_write_bytes_per_sec,
                    disk_available_bytes, disk_total_bytes,
                    process_cpu_percent, process_memory_bytes, process_uptime_seconds,
                    process_thread_count, process_fd_count
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&metrics_event.agent_id)
            .bind(metrics_event.timestamp)
            .bind(metrics.cpu_usage_percent)
            .bind(metrics.memory_used_bytes as i64)
            .bind(metrics.memory_total_bytes as i64)
            .bind(network_rx)
            .bind(network_tx)
            .bind(network_rx_rate)
            .bind(network_tx_rate)
            .bind(disk_read)
            .bind(disk_write)
            .bind(disk_read_rate)
            .bind(disk_write_rate)
            .bind(disk_available)
            .bind(disk_total)
            .bind(process_cpu)
            .bind(process_memory)
            .bind(process_uptime)
            .bind(process_threads)
            .bind(process_fds)
            .execute(&pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_recent_system_metrics(
        &self,
        agent_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SystemMetricsEvent>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = if let Some(agent_id) = agent_id {
            if agent_id == "orchestrator" {
                // Fetch from orchestrator_metrics table
                sqlx::query(
                    r#"
                    SELECT 'orchestrator' as agent_id, timestamp, cpu_usage_percent, memory_used_bytes, memory_total_bytes,
                           network_rx_bytes, network_tx_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec,
                           disk_read_bytes, disk_write_bytes, disk_read_bytes_per_sec, disk_write_bytes_per_sec,
                           disk_available_bytes, disk_total_bytes,
                           process_cpu_percent, process_memory_bytes, process_uptime_seconds,
                           process_thread_count, process_fd_count
                    FROM orchestrator_metrics 
                    ORDER BY timestamp DESC 
                    LIMIT ?
                    "#
                )
                .bind(limit)
                .fetch_all(&pool)
                .await?
            } else {
                // Fetch from system_metrics table for proxy agents
                sqlx::query(
                    r#"
                    SELECT agent_id, timestamp, cpu_usage_percent, memory_used_bytes, memory_total_bytes,
                           network_rx_bytes, network_tx_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec,
                           disk_read_bytes, disk_write_bytes, disk_read_bytes_per_sec, disk_write_bytes_per_sec,
                           disk_available_bytes, disk_total_bytes,
                           process_cpu_percent, process_memory_bytes, process_uptime_seconds,
                           process_thread_count, process_fd_count
                    FROM system_metrics 
                    WHERE agent_id = ?
                    ORDER BY timestamp DESC 
                    LIMIT ?
                    "#
                )
                .bind(agent_id)
                .bind(limit)
                .fetch_all(&pool)
                .await?
            }
        } else {
            // General query from system_metrics (agents only)
            sqlx::query(
                r#"
                SELECT agent_id, timestamp, cpu_usage_percent, memory_used_bytes, memory_total_bytes,
                       network_rx_bytes, network_tx_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec,
                       disk_read_bytes, disk_write_bytes, disk_read_bytes_per_sec, disk_write_bytes_per_sec,
                       disk_available_bytes, disk_total_bytes,
                       process_cpu_percent, process_memory_bytes, process_uptime_seconds,
                       process_thread_count, process_fd_count
                FROM system_metrics 
                ORDER BY timestamp DESC 
                LIMIT ?
                "#
            )
            .bind(limit)
            .fetch_all(&pool)
            .await?
        };

        let mut events = Vec::new();
        for row in rows {
            let agent_id: String = row.get("agent_id");
            let timestamp: i64 = row.get("timestamp");
            let cpu_usage_percent: f32 = row.get("cpu_usage_percent");
            let memory_used_bytes: i64 = row.get("memory_used_bytes");
            let memory_total_bytes: i64 = row.get("memory_total_bytes");

            let network_rx_bytes: i64 = row.get("network_rx_bytes");
            let network_tx_bytes: i64 = row.get("network_tx_bytes");
            let network_rx_bytes_per_sec: i64 = row.get("network_rx_bytes_per_sec");
            let network_tx_bytes_per_sec: i64 = row.get("network_tx_bytes_per_sec");

            let disk_read_bytes: i64 = row.get("disk_read_bytes");
            let disk_write_bytes: i64 = row.get("disk_write_bytes");
            let disk_read_bytes_per_sec: i64 = row.get("disk_read_bytes_per_sec");
            let disk_write_bytes_per_sec: i64 = row.get("disk_write_bytes_per_sec");
            let disk_available_bytes: i64 = row.get("disk_available_bytes");
            let disk_total_bytes: i64 = row.get("disk_total_bytes");

            let process_cpu_percent: f32 = row.get("process_cpu_percent");
            let process_memory_bytes: i64 = row.get("process_memory_bytes");
            let process_uptime_seconds: i64 = row.get("process_uptime_seconds");
            let process_thread_count: i64 = row.get("process_thread_count");
            let process_fd_count: i64 = row.get("process_fd_count");

            let network_metrics = crate::pb::NetworkMetrics {
                rx_bytes_total: network_rx_bytes as u64,
                tx_bytes_total: network_tx_bytes as u64,
                rx_bytes_per_sec: network_rx_bytes_per_sec as u64,
                tx_bytes_per_sec: network_tx_bytes_per_sec as u64,
                interfaces: vec![], // Not stored in simplified schema
            };

            let disk_metrics = crate::pb::DiskMetrics {
                read_bytes_total: disk_read_bytes as u64,
                write_bytes_total: disk_write_bytes as u64,
                read_bytes_per_sec: disk_read_bytes_per_sec as u64,
                write_bytes_per_sec: disk_write_bytes_per_sec as u64,
                available_bytes: disk_available_bytes as u64,
                total_bytes: disk_total_bytes as u64,
            };

            let process_metrics = crate::pb::ProcessMetrics {
                cpu_usage_percent: process_cpu_percent,
                memory_bytes: process_memory_bytes as u64,
                uptime_seconds: process_uptime_seconds as u64,
                thread_count: process_thread_count as u32,
                file_descriptor_count: process_fd_count as u32,
            };

            let system_metrics = crate::pb::SystemMetrics {
                cpu_usage_percent,
                memory_used_bytes: memory_used_bytes as u64,
                memory_total_bytes: memory_total_bytes as u64,
                network: Some(network_metrics),
                disk: Some(disk_metrics),
                process: Some(process_metrics),
            };

            events.push(SystemMetricsEvent {
                agent_id,
                timestamp,
                metrics: Some(system_metrics),
            });
        }
        Ok(events)
    }

    // ============================================================================
    // SETTINGS MANAGEMENT
    // ============================================================================

    /// Get a setting value by key (generic)
    pub async fn get_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let row = sqlx::query("SELECT value FROM project_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&pool)
            .await?;

        if let Some(row) = row {
            let value_json: String = row.get("value");
            let value: T = serde_json::from_str(&value_json)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Save a setting value by key (generic)
    pub async fn save_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<(), sqlx::Error> {
        let pool = match self.get_pool().await {
            Ok(p) => p,
            Err(e) => return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                e.to_string(),
            ))),
        };

        let value_json = serde_json::to_string(value)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let timestamp = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO project_settings (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#
        )
        .bind(key)
        .bind(value_json)
        .bind(timestamp)
        .execute(&pool)
        .await?;

        Ok(())
    }

    /// Get scope configuration
    pub async fn get_scope_config(&self) -> Result<ScopeConfig, sqlx::Error> {
        Ok(self.get_setting("scope").await?.unwrap_or_default())
    }

    /// Save scope configuration
    pub async fn save_scope_config(&self, config: &ScopeConfig) -> Result<(), sqlx::Error> {
        self.save_setting("scope", config).await
    }

    /// Get interception configuration
    pub async fn get_interception_config(&self) -> Result<InterceptionConfig, sqlx::Error> {
        Ok(self.get_setting("interception").await?.unwrap_or_default())
    }

    /// Save interception configuration
    pub async fn save_interception_config(&self, config: &InterceptionConfig) -> Result<(), sqlx::Error> {
        self.save_setting("interception", config).await
    }

    // ============================================================================
    // PROJECT IMPORT/EXPORT (.proxxy format)
    // ============================================================================

    /// Export project to .proxxy file (ZIP archive)
    pub async fn export_project(&self, name: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        
        let folder_name = format!("{}.proxxy", name);
        let project_path = self.projects_dir.join(folder_name);
        if !project_path.exists() {
            return Err("Project does not exist".into());
        }

        let db_path = project_path.join("proxxy.db");
        if !db_path.exists() {
            return Err("Project database not found".into());
        }

        // Create ZIP archive
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Add database file
        zip.start_file("proxxy.db", options)?;
        let db_content = std::fs::read(&db_path)?;
        zip.write_all(&db_content)?;

        // Add metadata
        let metadata = serde_json::json!({
            "name": name,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "version": "1.0"
        });
        zip.start_file("metadata.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;

        zip.finish()?;
        info!("✓ Exported project '{}' to {}", name, output_path);
        Ok(())
    }

    /// Import project from .proxxy file
    pub async fn import_project(&self, proxxy_path: &str, project_name: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        use std::io::Read;
        
        let file = std::fs::File::open(proxxy_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Read metadata
        let mut metadata_file = archive.by_name("metadata.json")?;
        let mut metadata_content = String::new();
        metadata_file.read_to_string(&mut metadata_content)?;
        drop(metadata_file); // Release borrow
        
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content)?;
        
        let original_name = metadata["name"].as_str().ok_or("Invalid metadata")?;
        let final_name = project_name.unwrap_or(original_name);

        // Create project directory with .proxxy extension
        let folder_name = format!("{}.proxxy", final_name);
        let project_path = self.projects_dir.join(folder_name);
        if project_path.exists() {
            return Err(format!("Project '{}' already exists", final_name).into());
        }
        fs::create_dir_all(&project_path)?;

        // Extract database
        let mut db_file = archive.by_name("proxxy.db")?;
        let mut db_content = Vec::new();
        db_file.read_to_end(&mut db_content)?;
        
        let db_path = project_path.join("proxxy.db");
        std::fs::write(&db_path, db_content)?;

        info!("✓ Imported project '{}' from {}", final_name, proxxy_path);
        Ok(final_name.to_string())
    }
}

