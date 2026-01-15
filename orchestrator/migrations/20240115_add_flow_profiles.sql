-- Flow Profiles: Store recorded browser flows (login, checkout, forms, etc.)
-- This migration adds support for the flow-engine module

CREATE TABLE IF NOT EXISTS flow_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    flow_type TEXT NOT NULL DEFAULT 'Custom',
    start_url TEXT NOT NULL,
    steps TEXT NOT NULL, -- JSON array of FlowStep
    meta TEXT, -- JSON FlowMeta object
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    agent_id TEXT, -- Agent that recorded this profile
    status TEXT DEFAULT 'active' -- active, archived, failed, recording
);

-- Flow Executions: Track replay attempts and results
CREATE TABLE IF NOT EXISTS flow_executions (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    status TEXT NOT NULL, -- running, success, failed, paused
    error_message TEXT,
    steps_completed INTEGER DEFAULT 0,
    total_steps INTEGER NOT NULL,
    session_cookies TEXT, -- JSON cookies extracted after successful flow
    extracted_data TEXT, -- JSON key-value pairs extracted during flow
    FOREIGN KEY(profile_id) REFERENCES flow_profiles(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_flow_profiles_status ON flow_profiles(status);
CREATE INDEX IF NOT EXISTS idx_flow_profiles_flow_type ON flow_profiles(flow_type);
CREATE INDEX IF NOT EXISTS idx_flow_executions_profile_id ON flow_executions(profile_id);
CREATE INDEX IF NOT EXISTS idx_flow_executions_status ON flow_executions(status);
