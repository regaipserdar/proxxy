//! Runtime Traffic Policy Configuration
//!
//! This module defines the dynamic policy structures that can be updated
//! at runtime via gRPC from the Orchestrator UI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime Traffic Policy (Operator Configuration)
/// This structure can be continuously updated from the UI via gRPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPolicy {
    /// Scope: Which domains should we log? (To prevent noise)
    pub scope: ScopeConfig,

    /// Rules: Blocking, Modification, and Interception rules
    pub interception_rules: Vec<InterceptionRule>,

    /// Match & Replace: Automatic text replacement rules
    pub match_replace_rules: Vec<MatchReplaceRule>,
}

impl Default for TrafficPolicy {
    fn default() -> Self {
        Self {
            scope: ScopeConfig::default(),
            interception_rules: Vec::new(),
            match_replace_rules: Vec::new(),
        }
    }
}

/// Scope Configuration (Target Definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Only process these domains (Whitelist)
    /// Examples: ["*.google.com", "api.target.com"]
    pub include: Vec<String>,

    /// Ignore these domains (Blacklist)
    /// Examples: ["*.google-analytics.com", "*.facebook.com"]
    pub exclude: Vec<String>,

    /// What to do with out-of-scope traffic?
    pub out_of_scope_action: OutOfScopeAction,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            include: vec!["*".to_string()], // Include everything by default
            exclude: Vec::new(),
            out_of_scope_action: OutOfScopeAction::Pass,
        }
    }
}

impl ScopeConfig {
    /// Check if a URL is within the defined scope
    pub fn is_allowed(&self, url: &str) -> bool {
        // Extract hostname from URL
        let hostname = match url::Url::parse(url) {
            Ok(parsed) => parsed.host_str().unwrap_or("").to_string(),
            Err(_) => return false,
        };

        // Check exclude list first (blacklist takes priority)
        for pattern in &self.exclude {
            if wildmatch::WildMatch::new(pattern).matches(&hostname) {
                return false;
            }
        }

        // Check include list
        for pattern in &self.include {
            if wildmatch::WildMatch::new(pattern).matches(&hostname) {
                return true;
            }
        }

        // If no include pattern matched, it's out of scope
        false
    }
}

/// Action to take for out-of-scope traffic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutOfScopeAction {
    /// Write to database but don't show in UI
    LogOnly,
    /// Drop the connection immediately (Save bandwidth)
    Drop,
    /// Pass through without processing (Passthrough)
    Pass,
}

/// Advanced Rule Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptionRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,

    /// Conditions that must be met for the rule to trigger (AND logic)
    pub conditions: Vec<RuleCondition>,

    /// Action to take when conditions are met
    pub action: RuleAction,
}

impl InterceptionRule {
    /// Check if this rule matches the given request
    pub fn matches(&self, req: &RequestContext) -> bool {
        if !self.enabled {
            return false;
        }

        // All conditions must match (AND logic)
        self.conditions
            .iter()
            .all(|condition| condition.matches(req))
    }
}

/// Request context for rule matching
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub port: u16,
}

/// Rule Condition - What to check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// URL contains string (e.g., "/admin")
    UrlContains(String),

    /// URL matches regex
    UrlRegex(String),

    /// HTTP Method matches (e.g., "POST")
    Method(String),

    /// Check if a header exists (e.g., "X-Custom-Token")
    HasHeader(String),

    /// Header value matches regex
    HeaderValueMatch { key: String, regex: String },

    /// Body contains regex pattern
    BodyRegex(String),

    /// Port matches
    Port(u16),
}

impl RuleCondition {
    /// Check if this condition matches the request
    pub fn matches(&self, req: &RequestContext) -> bool {
        match self {
            RuleCondition::UrlContains(s) => req.url.contains(s),
            RuleCondition::UrlRegex(pattern) => regex::Regex::new(pattern)
                .map(|re| re.is_match(&req.url))
                .unwrap_or(false),
            RuleCondition::Method(m) => req.method.eq_ignore_ascii_case(m),
            RuleCondition::HasHeader(key) => req.headers.contains_key(key),
            RuleCondition::HeaderValueMatch {
                key,
                regex: pattern,
            } => req.headers.get(key).map_or(false, |value| {
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(value))
                    .unwrap_or(false)
            }),
            RuleCondition::BodyRegex(pattern) => {
                let body_str = String::from_utf8_lossy(&req.body);
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(&body_str))
                    .unwrap_or(false)
            }
            RuleCondition::Port(p) => req.port == *p,
        }
    }
}

/// Actions (The core of your question)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Stop the request and present it to the user in the UI for approval (Intercept)
    Pause,

    /// Return 403 Forbidden (Block)
    Block { reason: String },

    /// Drop the TCP connection with RST packet (Drop - Silent Kill)
    Drop,

    /// Delay the request by X milliseconds (Timeout testing)
    Delay(u64),

    /// Automatically inject/modify a header
    InjectHeader { key: String, value: String },

    /// Modify the request body
    ModifyBody { find: String, replace: String },
}

/// Automatic Find and Replace (Similar to Burp Suite "Match and Replace")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReplaceRule {
    pub enabled: bool,
    pub match_regex: String,
    pub replace_string: String,
    pub location: MatchLocation,
}

/// Where to apply the match/replace rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchLocation {
    RequestHeader,
    RequestBody,
    ResponseHeader,
    ResponseBody,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_config_wildcard() {
        let scope = ScopeConfig {
            include: vec!["*.google.com".to_string()],
            exclude: vec!["analytics.google.com".to_string()],
            out_of_scope_action: OutOfScopeAction::Drop,
        };

        assert!(scope.is_allowed("https://www.google.com/search"));
        assert!(!scope.is_allowed("https://analytics.google.com/track"));
        assert!(!scope.is_allowed("https://facebook.com"));
    }

    #[test]
    fn test_rule_condition_url_contains() {
        let req = RequestContext {
            url: "https://example.com/admin/users".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            port: 443,
        };

        let condition = RuleCondition::UrlContains("/admin".to_string());
        assert!(condition.matches(&req));
    }

    #[test]
    fn test_rule_condition_method() {
        let req = RequestContext {
            url: "https://example.com/api".to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            port: 443,
        };

        let condition = RuleCondition::Method("POST".to_string());
        assert!(condition.matches(&req));
    }
}
