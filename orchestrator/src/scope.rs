use crate::models::settings::ScopeConfig;
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
fn matches_pattern(pattern: &str, host: &str, use_regex: bool) -> bool {
    if use_regex {
        // Regex mode
        match Regex::new(pattern) {
            Ok(re) => re.is_match(host),
            Err(e) => {
                warn!("Invalid regex pattern '{}': {}", pattern, e);
                false
            }
        }
    } else {
        // Glob mode
        match Pattern::new(pattern) {
            Ok(pat) => pat.matches(host),
            Err(e) => {
                warn!("Invalid glob pattern '{}': {}", pattern, e);
                false
            }
        }
    }
}

/// Check if URL is in scope based on configuration
/// 
/// Rules:
/// 1. If scope is disabled, everything is in scope
/// 2. Check exclude patterns first (highest priority)
/// 3. If no include patterns, everything is in scope
/// 4. Check include patterns
pub fn is_in_scope(config: &ScopeConfig, url: &str) -> bool {
    if !config.enabled {
        return true; // Scope disabled = everything in scope
    }
    
    let host = extract_host(url);
    
    // Check excludes first (highest priority)
    for pattern in &config.exclude_patterns {
        if matches_pattern(pattern, &host, config.use_regex) {
            return false;
        }
    }
    
    // If no includes, everything is in scope (except excludes)
    if config.include_patterns.is_empty() {
        return true;
    }
    
    // Check includes
    for pattern in &config.include_patterns {
        if matches_pattern(pattern, &host, config.use_regex) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://example.com/path"), "example.com");
        assert_eq!(extract_host("http://api.example.com:8080/v1"), "api.example.com");
        assert_eq!(extract_host("example.com"), "example.com");
    }

    #[test]
    fn test_scope_disabled() {
        let config = ScopeConfig {
            enabled: false,
            include_patterns: vec!["*.example.com".to_string()],
            exclude_patterns: vec![],
            use_regex: false,
        };
        
        assert!(is_in_scope(&config, "https://google.com"));
        assert!(is_in_scope(&config, "https://example.com"));
    }

    #[test]
    fn test_glob_include() {
        let config = ScopeConfig {
            enabled: true,
            include_patterns: vec!["*.example.com".to_string()],
            exclude_patterns: vec![],
            use_regex: false,
        };
        
        assert!(is_in_scope(&config, "https://api.example.com"));
        assert!(is_in_scope(&config, "https://www.example.com"));
        assert!(!is_in_scope(&config, "https://google.com"));
    }

    #[test]
    fn test_exclude_priority() {
        let config = ScopeConfig {
            enabled: true,
            include_patterns: vec!["*.example.com".to_string()],
            exclude_patterns: vec!["admin.example.com".to_string()],
            use_regex: false,
        };
        
        assert!(is_in_scope(&config, "https://api.example.com"));
        assert!(!is_in_scope(&config, "https://admin.example.com"));
    }

    #[test]
    fn test_empty_include() {
        let config = ScopeConfig {
            enabled: true,
            include_patterns: vec![],
            exclude_patterns: vec!["*.google.com".to_string()],
            use_regex: false,
        };
        
        // Empty include = everything in scope except excludes
        assert!(is_in_scope(&config, "https://example.com"));
        assert!(!is_in_scope(&config, "https://www.google.com"));
    }

    #[test]
    fn test_regex_mode() {
        let config = ScopeConfig {
            enabled: true,
            include_patterns: vec![r"^api\d+\.example\.com$".to_string()],
            exclude_patterns: vec![],
            use_regex: true,
        };
        
        assert!(is_in_scope(&config, "https://api1.example.com"));
        assert!(is_in_scope(&config, "https://api123.example.com"));
        assert!(!is_in_scope(&config, "https://api.example.com"));
    }
}
