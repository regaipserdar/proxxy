use wildmatch::WildMatch;

/// Configuration for traffic scope filtering
#[derive(Debug, Clone, Default)]
pub struct ScopeMatcher {
    allow_list: Vec<String>,
    block_list: Vec<String>,
}

impl ScopeMatcher {
    /// Create a new ScopeMatcher
    pub fn new(allow_list: Vec<String>, block_list: Vec<String>) -> Self {
        Self {
            allow_list,
            block_list,
        }
    }

    /// Check if a host is allowed by the scope configuration
    ///
    /// Logic:
    /// 1. If block_list matches, return false (explicit deny).
    /// 2. If allow_list is empty, return true (allow all by default).
    /// 3. If allow_list is not empty, return true only if it matches.
    pub fn is_allowed(&self, host: &str) -> bool {
        // Check block list first
        for pattern in &self.block_list {
            if WildMatch::new(pattern).matches(host) {
                return false;
            }
        }

        // If allow list is empty, allow everything (except blocked)
        if self.allow_list.is_empty() {
            return true;
        }

        // Check allow list
        for pattern in &self.allow_list {
            if WildMatch::new(pattern).matches(host) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_matching() {
        // Test 1: Empty lists = allow all
        let matcher = ScopeMatcher::new(vec![], vec![]);
        assert!(matcher.is_allowed("google.com"));

        // Test 2: Explicit block
        let matcher = ScopeMatcher::new(vec![], vec!["*.google.com".to_string()]);
        assert!(!matcher.is_allowed("mail.google.com"));
        assert!(matcher.is_allowed("example.com"));

        // Test 3: Explicit allow (whitelist mode)
        let matcher = ScopeMatcher::new(vec!["*.example.com".to_string()], vec![]);
        assert!(matcher.is_allowed("api.example.com"));
        assert!(!matcher.is_allowed("google.com"));

        // Test 4: Mixed
        let matcher = ScopeMatcher::new(
            vec!["*.google.com".to_string()],
            vec!["ads.google.com".to_string()],
        );
        assert!(matcher.is_allowed("mail.google.com"));
        assert!(!matcher.is_allowed("ads.google.com")); // Block takes precedence
        assert!(!matcher.is_allowed("example.com")); // Not in allow list
    }
}
