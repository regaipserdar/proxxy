//! Selector Analyzer Module
//!
//! Smart selector generation with priority-based analysis and blacklist filtering.

use crate::error::{FlowEngineError, FlowResult};
use crate::flow::model::{SmartSelector, SelectorType, AlternativeSelector, ValidationResult};
use chrono::Utc;
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;

/// Selector generation configuration
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Maximum selector chain depth
    pub max_depth: usize,
    /// Minimum priority to accept a selector
    pub min_priority: u8,
    /// Number of alternatives to generate
    pub max_alternatives: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            min_priority: 30,
            max_alternatives: 3,
        }
    }
}

/// Selector analyzer for smart selector generation
pub struct SelectorAnalyzer {
    config: AnalyzerConfig,
    blacklist: SelectorBlacklist,
}

impl SelectorAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            blacklist: SelectorBlacklist::default(),
        }
    }

    /// Analyze an element and generate a smart selector
    pub fn analyze_element(&self, element_info: &ElementInfo) -> FlowResult<SmartSelector> {
        let mut candidates: Vec<(String, SelectorType, u8)> = Vec::new();

        // Priority 1: Stable IDs (highest priority)
        if let Some(ref test_id) = element_info.data_testid {
            if !self.blacklist.is_blacklisted(test_id) {
                candidates.push((
                    format!("[data-testid='{}']", test_id),
                    SelectorType::Css,
                    95,
                ));
            }
        }

        if let Some(ref data_cy) = element_info.data_cy {
            if !self.blacklist.is_blacklisted(data_cy) {
                candidates.push((
                    format!("[data-cy='{}']", data_cy),
                    SelectorType::Css,
                    95,
                ));
            }
        }

        if let Some(ref id) = element_info.id {
            if !self.blacklist.is_blacklisted(id) && !self.looks_dynamic(id) {
                candidates.push((
                    format!("#{}", id),
                    SelectorType::Css,
                    90,
                ));
            }
        }

        // Priority 2: Name attribute
        if let Some(ref name) = element_info.name {
            if !self.blacklist.is_blacklisted(name) {
                candidates.push((
                    format!("[name='{}']", name),
                    SelectorType::Css,
                    85,
                ));
            }
        }

        // Priority 3: ARIA labels
        if let Some(ref aria_label) = element_info.aria_label {
            candidates.push((
                aria_label.clone(),
                SelectorType::AriaLabel,
                80,
            ));
        }

        // Priority 4: Placeholder for inputs
        if let Some(ref placeholder) = element_info.placeholder {
            candidates.push((
                placeholder.clone(),
                SelectorType::Placeholder,
                75,
            ));
        }

        // Priority 5: Text content (for buttons/links)
        if let Some(ref text) = element_info.text_content {
            if text.len() < 50 && !text.is_empty() {
                candidates.push((
                    text.clone(),
                    SelectorType::Text,
                    70,
                ));
            }
        }

        // Priority 6: Tag + class combination
        if let Some(ref classes) = element_info.class_list {
            let stable_classes: Vec<&String> = classes
                .iter()
                .filter(|c| !self.blacklist.is_blacklisted(c) && !self.looks_dynamic(c))
                .collect();

            if !stable_classes.is_empty() {
                let class_selector = stable_classes
                    .iter()
                    .take(2)
                    .map(|c| format!(".{}", c))
                    .collect::<Vec<_>>()
                    .join("");
                
                candidates.push((
                    format!("{}{}", element_info.tag_name.to_lowercase(), class_selector),
                    SelectorType::Css,
                    50,
                ));
            }
        }

        // Priority 7: Type attribute for inputs
        if element_info.tag_name.eq_ignore_ascii_case("input") {
            if let Some(ref input_type) = element_info.input_type {
                candidates.push((
                    format!("input[type='{}']", input_type),
                    SelectorType::Css,
                    40,
                ));
            }
        }

        // Sort by priority (descending)
        candidates.sort_by(|a, b| b.2.cmp(&a.2));

        // Filter by minimum priority
        let valid_candidates: Vec<_> = candidates
            .into_iter()
            .filter(|(_, _, p)| *p >= self.config.min_priority)
            .collect();

        if valid_candidates.is_empty() {
            return Err(FlowEngineError::SelectorGeneration(
                "No valid selectors found for element".to_string(),
            ));
        }

        // Create primary selector
        let (primary_value, primary_type, primary_priority) = valid_candidates[0].clone();

        // Create alternatives
        let alternatives: Vec<AlternativeSelector> = valid_candidates
            .iter()
            .skip(1)
            .take(self.config.max_alternatives)
            .map(|(value, sel_type, priority)| AlternativeSelector {
                value: value.clone(),
                selector_type: sel_type.clone(),
                priority: *priority,
            })
            .collect();

        debug!(
            "Generated selector: {} (priority: {}, alternatives: {})",
            primary_value, primary_priority, alternatives.len()
        );

        Ok(SmartSelector {
            value: primary_value,
            selector_type: primary_type,
            priority: primary_priority,
            alternatives,
            validation_result: None,
        })
    }

    /// Check if a string looks like a dynamic/generated value
    fn looks_dynamic(&self, value: &str) -> bool {
        // Check for common dynamic patterns
        lazy_static::lazy_static! {
            static ref DYNAMIC_PATTERNS: Vec<Regex> = vec![
                Regex::new(r"^[a-f0-9]{8,}$").unwrap(),           // Hex hashes
                Regex::new(r"^\d{10,}$").unwrap(),                 // Long numbers
                Regex::new(r"^[a-z]{1,3}\d{4,}$").unwrap(),       // Short prefix + numbers
                Regex::new(r"__\w+__").unwrap(),                   // Double underscore wrappers
                Regex::new(r"^css-[a-z0-9]+$").unwrap(),          // CSS-in-JS
                Regex::new(r"^sc-[a-zA-Z]+$").unwrap(),           // Styled-components
                Regex::new(r"^emotion-\d+$").unwrap(),            // Emotion CSS
                Regex::new(r"^MuiBox-root-\d+$").unwrap(),        // MUI
                Regex::new(r"^v-[a-f0-9]+$").unwrap(),            // Vue scoped
                Regex::new(r"^_[A-Z][a-zA-Z]+_[a-z0-9]+$").unwrap(), // React CSS modules
            ];
        }

        for pattern in DYNAMIC_PATTERNS.iter() {
            if pattern.is_match(value) {
                return true;
            }
        }

        false
    }
}

impl Default for SelectorAnalyzer {
    fn default() -> Self {
        Self::new(AnalyzerConfig::default())
    }
}

/// Information about a DOM element
#[derive(Debug, Clone, Default)]
pub struct ElementInfo {
    pub tag_name: String,
    pub id: Option<String>,
    pub class_list: Option<Vec<String>>,
    pub name: Option<String>,
    pub input_type: Option<String>,
    pub placeholder: Option<String>,
    pub aria_label: Option<String>,
    pub data_testid: Option<String>,
    pub data_cy: Option<String>,
    pub text_content: Option<String>,
    pub href: Option<String>,
}

/// Selector blacklist for unreliable patterns
pub struct SelectorBlacklist {
    patterns: HashSet<String>,
    regex_patterns: Vec<Regex>,
}

impl SelectorBlacklist {
    pub fn new() -> Self {
        let mut patterns = HashSet::new();
        
        // Tailwind utility classes
        for class in [
            "flex", "hidden", "block", "inline", "grid",
            "p-", "m-", "px-", "py-", "mx-", "my-", "pt-", "pb-", "pl-", "pr-",
            "w-", "h-", "min-w-", "min-h-", "max-w-", "max-h-",
            "text-", "font-", "leading-", "tracking-",
            "bg-", "border-", "rounded", "shadow",
            "hover:", "focus:", "active:", "disabled:",
            "sm:", "md:", "lg:", "xl:", "2xl:",
        ] {
            patterns.insert(class.to_string());
        }

        // Bootstrap classes
        for class in [
            "container", "row", "col", "btn", "form-control",
            "d-flex", "d-none", "d-block", "justify-content-",
            "align-items-", "text-center", "text-left", "text-right",
        ] {
            patterns.insert(class.to_string());
        }

        let regex_patterns = vec![
            Regex::new(r"^col-\d+$").unwrap(),
            Regex::new(r"^col-(sm|md|lg|xl)-\d+$").unwrap(),
            Regex::new(r"^mb?-\d+$").unwrap(),
            Regex::new(r"^pb?-\d+$").unwrap(),
        ];

        Self {
            patterns,
            regex_patterns,
        }
    }

    /// Check if a value is blacklisted
    pub fn is_blacklisted(&self, value: &str) -> bool {
        // Check exact matches
        if self.patterns.contains(value) {
            return true;
        }

        // Check prefix matches
        for pattern in &self.patterns {
            if pattern.ends_with('-') && value.starts_with(pattern) {
                return true;
            }
            if pattern.ends_with(':') && value.starts_with(pattern) {
                return true;
            }
        }

        // Check regex patterns
        for regex in &self.regex_patterns {
            if regex.is_match(value) {
                return true;
            }
        }

        false
    }
}

impl Default for SelectorBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

/// Selector validator
pub struct SelectorValidator;

impl SelectorValidator {
    /// Validate a selector on a page (returns validation result)
    pub fn create_validation_result(is_valid: bool, match_count: usize, is_visible: bool) -> ValidationResult {
        ValidationResult {
            is_valid,
            match_count,
            is_visible,
            is_interactable: is_visible && is_valid,
            validated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_detection() {
        let analyzer = SelectorAnalyzer::default();

        // Should detect dynamic
        assert!(analyzer.looks_dynamic("abc123456789"));
        assert!(analyzer.looks_dynamic("css-1a2b3c4d"));
        assert!(analyzer.looks_dynamic("sc-aBcDeFg"));

        // Should not detect as dynamic
        assert!(!analyzer.looks_dynamic("login-button"));
        assert!(!analyzer.looks_dynamic("submit"));
        assert!(!analyzer.looks_dynamic("user-email"));
    }

    #[test]
    fn test_blacklist() {
        let blacklist = SelectorBlacklist::default();

        // Should be blacklisted
        assert!(blacklist.is_blacklisted("flex"));
        assert!(blacklist.is_blacklisted("p-4"));
        assert!(blacklist.is_blacklisted("text-white"));
        assert!(blacklist.is_blacklisted("hover:bg-blue-500"));

        // Should not be blacklisted
        assert!(!blacklist.is_blacklisted("login-form"));
        assert!(!blacklist.is_blacklisted("submit-button"));
    }

    #[test]
    fn test_element_analysis() {
        let analyzer = SelectorAnalyzer::default();

        let element = ElementInfo {
            tag_name: "button".to_string(),
            id: Some("submit-btn".to_string()),
            data_testid: Some("login-submit".to_string()),
            ..Default::default()
        };

        let selector = analyzer.analyze_element(&element).unwrap();

        // Should prefer data-testid over id
        assert_eq!(selector.priority, 95);
        assert!(selector.value.contains("data-testid"));
    }
}
