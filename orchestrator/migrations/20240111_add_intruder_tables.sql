-- Intruder Tables Migration
-- Migration 011: Add intruder attack configurations and results tables
-- Dependencies: Core migrations (001-004), Repeater migrations (010)

-- Intruder attack configurations
CREATE TABLE IF NOT EXISTS intruder_attacks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    request_template TEXT NOT NULL, -- JSON with §markers§
    attack_mode TEXT NOT NULL, -- sniper, battering_ram, pitchfork, cluster_bomb
    payload_sets TEXT NOT NULL, -- JSON array of payload configurations
    target_agents TEXT NOT NULL, -- JSON array of agent IDs
    distribution_strategy TEXT NOT NULL, -- round_robin, batch
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'configured' -- configured, running, completed, stopped
);

-- Intruder attack results
CREATE TABLE IF NOT EXISTS intruder_results (
    id TEXT PRIMARY KEY,
    attack_id TEXT NOT NULL,
    request_data TEXT NOT NULL, -- JSON serialized request with injected payloads
    response_data TEXT, -- JSON serialized response
    agent_id TEXT NOT NULL,
    payload_values TEXT NOT NULL, -- JSON array of payload values used
    executed_at INTEGER NOT NULL,
    duration_ms INTEGER,
    status_code INTEGER,
    response_length INTEGER,
    is_highlighted BOOLEAN DEFAULT false,
    FOREIGN KEY(attack_id) REFERENCES intruder_attacks(id) ON DELETE CASCADE,
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_intruder_attacks_status ON intruder_attacks(status);
CREATE INDEX IF NOT EXISTS idx_intruder_attacks_created ON intruder_attacks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_intruder_attacks_mode ON intruder_attacks(attack_mode);

CREATE INDEX IF NOT EXISTS idx_intruder_results_attack ON intruder_results(attack_id);
CREATE INDEX IF NOT EXISTS idx_intruder_results_executed ON intruder_results(executed_at DESC);
CREATE INDEX IF NOT EXISTS idx_intruder_results_agent ON intruder_results(agent_id);
CREATE INDEX IF NOT EXISTS idx_intruder_results_status ON intruder_results(status_code);
CREATE INDEX IF NOT EXISTS idx_intruder_results_highlighted ON intruder_results(is_highlighted);
CREATE INDEX IF NOT EXISTS idx_intruder_results_response_length ON intruder_results(response_length);