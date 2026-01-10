use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopeConfig {
    pub enabled: bool,
    pub include_patterns: Vec<String>,  // ["*.example.com", "api.*.io"]
    pub exclude_patterns: Vec<String>,  // ["*.google.com"]
    pub use_regex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InterceptionConfig {
    pub enabled: bool,
    pub rules: Vec<InterceptionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptionRule {
    pub id: String,
    pub enabled: bool,
    pub name: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleCondition {
    Method { methods: Vec<String> },          // ["POST", "PUT"]
    UrlContains { pattern: String },          // "api/login"
    HeaderMatch { header: String, value: String },
    All,  // Match everything
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Pause,   // Hold request, show in UI for editing
    Drop,    // Silently drop
    Modify,  // Future: auto-modify headers/body
}
