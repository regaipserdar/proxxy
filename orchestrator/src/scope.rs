use crate::database::ScopeRule;
use glob::Pattern;
use regex::Regex;
use tracing::warn;

/// Extract hostname from URL
fn extract_host(url: &str) -> String {
    // Simple URL parsing - extract domain from http(s)://domain/path
    url.trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(url)
        .split(':')
        .next()
        .unwrap_or(url)
        .to_string()
}

/// Check if pattern matches the host
fn matches_pattern(pattern_str: &str, host: &str, is_regex: bool) -> bool {
    if is_regex {
        // Regex mode
        match Regex::new(pattern_str) {
            Ok(re) => re.is_match(host),
            Err(e) => {
                warn!("Invalid regex pattern '{}': {}", pattern_str, e);
                false
            }
        }
    } else {
        // Glob mode
        match Pattern::new(pattern_str) {
            Ok(pat) => pat.matches(host),
            Err(e) => {
                warn!("Invalid glob pattern '{}': {}", pattern_str, e);
                false
            }
        }
    }
}

/// Check if URL is in scope based on database rules
/// 
/// Rules:
/// 1. If no enabled rules exist, everything is in scope
/// 2. Check Exclude rules first (highest priority)
/// 3. If any Include rules exist, at least one must match (unless excluded)
/// 4. If ONLY Exclude rules exist, anything not excluded is in scope
pub fn is_in_scope(rules: &[ScopeRule], url: &str) -> bool {
    let enabled_rules: Vec<&ScopeRule> = rules.iter().filter(|r| r.enabled).collect();
    
    if enabled_rules.is_empty() {
        return true; // No rules = everything in scope
    }
    
    let host = extract_host(url);
    
    // 1. Check excludes first (blocking)
    for rule in enabled_rules.iter().filter(|r| r.rule_type == "Exclude") {
        if matches_pattern(&rule.pattern, &host, rule.is_regex) {
            return false;
        }
    }
    
    // 2. Check if there are any include rules
    let has_includes = enabled_rules.iter().any(|r| r.rule_type == "Include");
    
    if !has_includes {
        return true; // No includes, and not excluded
    }
    
    // 3. Check includes
    for rule in enabled_rules.iter().filter(|r| r.rule_type == "Include") {
        if matches_pattern(&rule.pattern, &host, rule.is_regex) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_rule(rule_type: &str, pattern: &str, is_regex: bool) -> ScopeRule {
        ScopeRule {
            id: "test".to_string(),
            rule_type: rule_type.to_string(),
            pattern: pattern.to_string(),
            is_regex,
            enabled: true,
            created_at: 0,
        }
    }

    #[test]
    fn test_empty_rules() {
        assert!(is_in_scope(&[], "https://example.com"));
    }

    #[test]
    fn test_exclude_only() {
        let rules = vec![mock_rule("Exclude", "*.google.com", false)];
        assert!(is_in_scope(&rules, "https://example.com"));
        assert!(!is_in_scope(&rules, "https://google.com"));
    }

    #[test]
    fn test_include_with_exclude() {
        let rules = vec![
            mock_rule("Include", "*.example.com", false),
            mock_rule("Exclude", "admin.example.com", false),
        ];
        
        assert!(is_in_scope(&rules, "https://api.example.com"));
        assert!(!is_in_scope(&rules, "https://admin.example.com"));
        assert!(!is_in_scope(&rules, "https://google.com"));
    }

    #[test]
    fn test_regex_rules() {
        let rules = vec![mock_rule("Include", r"^api\d+\.com$", true)];
        assert!(is_in_scope(&rules, "https://api1.com"));
        assert!(!is_in_scope(&rules, "https://api.com"));
    }
}
