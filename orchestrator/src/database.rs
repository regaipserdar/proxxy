use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
use crate::pb::{TrafficEvent, traffic_event, SystemMetricsEvent};
use tracing::info;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;
        
        // Parse the database URL and ensure create_if_missing is set
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Run migrations (path is relative to orchestrator crate root)
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("✓ Database initialized and migrated at {}", database_url);
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn upsert_agent(&self, id: &str, name: &str, hostname: &str, version: &str) -> Result<(), sqlx::Error> {
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
            "#
        )
        .bind(id)
        .bind(name)
        .bind(hostname)
        .bind(version)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_agent_name(&self, agent_id: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query("SELECT name FROM agents WHERE id = ?")
            .bind(agent_id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(row.map(|r| r.get("name")))
    }

    pub async fn mark_agent_offline(&self, agent_id: &str) -> Result<(), sqlx::Error> {
        let timestamp = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            UPDATE agents 
            SET status = 'Offline', last_heartbeat = ?
            WHERE id = ?
            "#
        )
        .bind(timestamp)
        .bind(agent_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_request(&self, event: &TrafficEvent, agent_id: &str) -> Result<(), sqlx::Error> {
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
                .execute(&self.pool)
                .await?;
            },
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
                    "#
                )
                .bind(res.status_code)
                .bind(headers_json)
                .bind(&res.body)
                .bind(timestamp)
                .bind(&event.request_id)
                .execute(&self.pool)
                .await?;
            },
            _ => {
                // Ignore other events for DB (WebSocket, etc. for now)
            }
        }
        Ok(())
    }

    pub async fn get_recent_requests(&self, limit: i64) -> Result<Vec<TrafficEvent>, sqlx::Error> {
        // This query needs to adapt to http_transactions.
        // It's tricky because we merged req/res into one row.
        // We'll reconstruct TrafficEvents. This might return incomplete events if response is missing?
        // Or we just return the Request part for list view?
        // For simplicity, let's return TrafficEvents as Requests, and maybe we need a separate query/struct for full method.
        // But the current UI expects TrafficEvent.
        
        // Let's modify the query to return Request events.
        
        let rows = sqlx::query(
            "SELECT request_id, req_method, req_url, req_headers, req_body, tls_info FROM http_transactions ORDER BY req_timestamp DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let request_id: String = row.get("request_id");
            let method: String = row.get("req_method");
            let url: String = row.get("req_url");
            let headers_json: String = row.get("req_headers");
            let body: Vec<u8> = row.get("req_body"); // Might be null? BLOB.
            let tls_json: String = row.get("tls_info");

            let headers: Option<crate::pb::HttpHeaders> = serde_json::from_str(&headers_json).ok();
            let tls: Option<crate::pb::TlsDetails> = serde_json::from_str(&tls_json).ok();

            events.push(TrafficEvent {
                request_id,
                event: Some(traffic_event::Event::Request(crate::pb::HttpRequestData {
                    method,
                    url,
                    headers,
                    body,
                    tls,
                })),
            });
        }
        Ok(events)
    }

    pub async fn save_system_metrics(&self, metrics_event: &SystemMetricsEvent) -> Result<(), sqlx::Error> {
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
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_recent_system_metrics(&self, agent_id: Option<&str>, limit: i64) -> Result<Vec<SystemMetricsEvent>, sqlx::Error> {
        let query = if let Some(agent_id) = agent_id {
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
        } else {
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
        };

        let rows = query.fetch_all(&self.pool).await?;

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
}