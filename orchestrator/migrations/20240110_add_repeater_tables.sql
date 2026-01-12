-- Repeater Tables Migration
-- Migration 010: Add repeater tabs and history tables
-- Dependencies: Core migrations (001-004)

-- Repeater tabs and configurations
CREATE TABLE IF NOT EXISTS repeater_tabs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    request_template TEXT NOT NULL, -- JSON serialized HttpRequestData
    target_agent_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT true,
    FOREIGN KEY(target_agent_id) REFERENCES agents(id)
);

-- Repeater execution history
CREATE TABLE IF NOT EXISTS repeater_history (
    id TEXT PRIMARY KEY,
    tab_id TEXT NOT NULL,
    request_data TEXT NOT NULL, -- JSON serialized HttpRequestData
    response_data TEXT, -- JSON serialized HttpResponseData
    agent_id TEXT NOT NULL,
    executed_at INTEGER NOT NULL,
    duration_ms INTEGER,
    status_code INTEGER,
    FOREIGN KEY(tab_id) REFERENCES repeater_tabs(id) ON DELETE CASCADE,
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_repeater_tabs_active ON repeater_tabs(is_active);
CREATE INDEX IF NOT EXISTS idx_repeater_tabs_created ON repeater_tabs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_repeater_tabs_agent ON repeater_tabs(target_agent_id);

CREATE INDEX IF NOT EXISTS idx_repeater_history_tab ON repeater_history(tab_id);
CREATE INDEX IF NOT EXISTS idx_repeater_history_executed ON repeater_history(executed_at DESC);
CREATE INDEX IF NOT EXISTS idx_repeater_history_agent ON repeater_history(agent_id);
CREATE INDEX IF NOT EXISTS idx_repeater_history_status ON repeater_history(status_code);