-- Add scope_rules table for Target Scope filtering
CREATE TABLE scope_rules (
    id TEXT PRIMARY KEY,
    rule_type TEXT NOT NULL, -- 'Include' or 'Exclude'
    pattern TEXT NOT NULL,
    is_regex BOOLEAN NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);

-- Index for quick lookup
CREATE INDEX idx_scope_rules_type ON scope_rules(rule_type);
