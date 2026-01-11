-- Agents Table
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    hostname TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL, -- 'Online', 'Offline'
    last_heartbeat INTEGER NOT NULL,
    capabilities TEXT -- JSON array of capabilities if needed
);

-- HTTP Transactions Table
CREATE TABLE IF NOT EXISTS http_transactions (
    request_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    
    -- Request Data
    req_method TEXT NOT NULL,
    req_url TEXT NOT NULL,
    req_headers TEXT, -- JSON
    req_body BLOB,
    req_timestamp INTEGER NOT NULL,
    
    -- Response Data (Nullable because we insert on Request first)
    res_status INTEGER,
    res_headers TEXT, -- JSON
    res_body BLOB,
    res_timestamp INTEGER,
    duration_ms INTEGER,
    
    -- Security
    tls_info TEXT, -- JSON
    
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_http_transactions_timestamp ON http_transactions(req_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_http_transactions_agent ON http_transactions(agent_id);

-- System Metrics Table
CREATE TABLE IF NOT EXISTS system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    cpu_usage_percent REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    network_rx_bytes INTEGER NOT NULL,
    network_tx_bytes INTEGER NOT NULL,
    network_rx_bytes_per_sec INTEGER NOT NULL,
    network_tx_bytes_per_sec INTEGER NOT NULL,
    disk_read_bytes INTEGER NOT NULL,
    disk_write_bytes INTEGER NOT NULL,
    disk_read_bytes_per_sec INTEGER NOT NULL,
    disk_write_bytes_per_sec INTEGER NOT NULL,
    disk_available_bytes INTEGER NOT NULL,
    disk_total_bytes INTEGER NOT NULL,
    process_cpu_percent REAL NOT NULL,
    process_memory_bytes INTEGER NOT NULL,
    process_uptime_seconds INTEGER NOT NULL,
    process_thread_count INTEGER NOT NULL,
    process_fd_count INTEGER NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

-- Orchestrator Metrics Table (Separate from Agents)
CREATE TABLE IF NOT EXISTS orchestrator_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    cpu_usage_percent REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    network_rx_bytes INTEGER NOT NULL,
    network_tx_bytes INTEGER NOT NULL,
    network_rx_bytes_per_sec INTEGER NOT NULL,
    network_tx_bytes_per_sec INTEGER NOT NULL,
    disk_read_bytes INTEGER NOT NULL,
    disk_write_bytes INTEGER NOT NULL,
    disk_read_bytes_per_sec INTEGER NOT NULL,
    disk_write_bytes_per_sec INTEGER NOT NULL,
    disk_available_bytes INTEGER NOT NULL,
    disk_total_bytes INTEGER NOT NULL,
    process_cpu_percent REAL NOT NULL,
    process_memory_bytes INTEGER NOT NULL,
    process_uptime_seconds INTEGER NOT NULL,
    process_thread_count INTEGER NOT NULL,
    process_fd_count INTEGER NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
);

-- Indexes for system metrics
CREATE INDEX IF NOT EXISTS idx_system_metrics_agent_timestamp ON system_metrics(agent_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp ON system_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_orchestrator_metrics_timestamp ON orchestrator_metrics(timestamp);
