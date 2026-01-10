-- Project Settings Table (Key-Value JSON Store)
CREATE TABLE IF NOT EXISTS project_settings (
    id INTEGER PRIMARY KEY,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Pre-populate defaults
INSERT OR IGNORE INTO project_settings (key, value) VALUES 
    ('scope', '{"enabled":false,"include_patterns":[],"exclude_patterns":[],"use_regex":false}'),
    ('interception', '{"enabled":false,"rules":[]}');
