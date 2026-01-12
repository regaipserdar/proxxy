-- Payload Sets Migration
-- Migration 012: Add reusable payload configurations table
-- Dependencies: Core migrations (001-004), Intruder migrations (011)

-- Payload sets for reuse
CREATE TABLE IF NOT EXISTS payload_sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL, -- wordlist, number_range, custom
    configuration TEXT NOT NULL, -- JSON configuration
    created_at INTEGER NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_payload_sets_type ON payload_sets(type);
CREATE INDEX IF NOT EXISTS idx_payload_sets_created ON payload_sets(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_payload_sets_name ON payload_sets(name);