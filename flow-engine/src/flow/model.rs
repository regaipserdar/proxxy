//! Flow Data Models
//!
//! Core data structures for recording and replaying browser flows.
//! These models support any user-defined flow: login, checkout, form filling, etc.

use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of flow being recorded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlowType {
    /// Login/authentication flow - produces session cookies/tokens
    Login,
    /// Checkout flow - e-commerce purchase sequence
    Checkout,
    /// Form submission flow
    FormSubmission,
    /// Navigation sequence - multi-page workflow
    Navigation,
    /// Custom user-defined flow
    Custom(String),
}

impl Default for FlowType {
    fn default() -> Self {
        FlowType::Custom("general".to_string())
    }
}

/// A recorded browser flow profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowProfile {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Type of flow
    pub flow_type: FlowType,
    /// Starting URL for the flow
    pub start_url: String,
    /// Ordered list of steps in the flow
    pub steps: Vec<FlowStep>,
    /// Additional metadata
    pub meta: FlowMeta,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// When the profile was last updated
    pub updated_at: DateTime<Utc>,
    /// Agent ID that recorded this profile (if any)
    pub agent_id: Option<String>,
    /// Profile status
    pub status: ProfileStatus,
}

impl FlowProfile {
    /// Create a new empty flow profile
    pub fn new(name: impl Into<String>, start_url: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            flow_type: FlowType::default(),
            start_url: start_url.into(),
            steps: Vec::new(),
            meta: FlowMeta::default(),
            created_at: now,
            updated_at: now,
            agent_id: None,
            status: ProfileStatus::Active,
        }
    }

    /// Add a step to the flow
    pub fn add_step(&mut self, step: FlowStep) {
        self.steps.push(step);
        self.updated_at = Utc::now();
    }

    /// Get total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// Profile status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProfileStatus {
    #[default]
    Active,
    Archived,
    Failed,
    Recording,
}

/// Metadata for a flow profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowMeta {
    /// Description of what this flow does
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Expected duration in milliseconds
    pub expected_duration_ms: Option<u64>,
    /// Number of successful replays
    pub success_count: u32,
    /// Number of failed replays
    pub failure_count: u32,
    /// Last successful replay timestamp
    pub last_success: Option<DateTime<Utc>>,
    /// URLs that indicate successful login/session (for session flows)
    pub success_indicators: Vec<String>,
    /// Custom key-value metadata
    pub custom: serde_json::Value,
}

/// Individual step in a browser flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowStep {
    /// Navigate to a URL
    Navigate {
        url: String,
        /// Optional selector to wait for before considering navigation complete
        wait_for: Option<String>,
    },

    /// Click an element
    Click {
        selector: SmartSelector,
        /// Optional selector to wait for after click
        wait_for: Option<String>,
    },

    /// Type text into an element
    Type {
        selector: SmartSelector,
        /// The value to type (may be sensitive like passwords)
        #[serde(serialize_with = "serialize_secret", deserialize_with = "deserialize_secret")]
        value: SecretString,
        /// Whether this is sensitive data (passwords, tokens)
        is_masked: bool,
        /// Clear existing content before typing
        clear_first: bool,
    },

    /// Wait for a condition
    Wait {
        /// Duration to wait in milliseconds
        duration_ms: u64,
        /// Optional condition to wait for
        condition: Option<WaitCondition>,
    },

    /// Validate session/state
    CheckSession {
        /// URL to check for session validity
        validation_url: String,
        /// Indicators of successful session (text, selectors, etc.)
        success_indicators: Vec<String>,
    },

    /// Submit a form
    Submit {
        selector: SmartSelector,
        /// Wait for navigation after submit
        wait_for_navigation: bool,
    },

    /// Select an option from dropdown
    Select {
        selector: SmartSelector,
        /// Value to select
        value: String,
    },

    /// Hover over an element
    Hover {
        selector: SmartSelector,
    },

    /// Press a keyboard key
    KeyPress {
        key: String,
        /// Modifiers (ctrl, alt, shift, meta)
        modifiers: Vec<String>,
    },

    /// Take a screenshot (for debugging/verification)
    Screenshot {
        /// Optional filename
        filename: Option<String>,
    },

    /// Extract data from the page
    Extract {
        /// Selector to extract from
        selector: SmartSelector,
        /// What to extract: text, attribute, html
        extract_type: ExtractType,
        /// Variable name to store result
        variable_name: String,
    },

    /// Custom JavaScript execution
    ExecuteScript {
        /// JavaScript code to execute
        script: String,
        /// Optional variable to store result
        result_variable: Option<String>,
    },

    /// Custom action for extensibility
    Custom {
        action_type: String,
        parameters: serde_json::Value,
    },
}

/// What to extract from an element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExtractType {
    Text,
    InnerHtml,
    OuterHtml,
    Attribute(String),
    Value,
}

/// Condition to wait for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for element to appear
    ElementVisible(String),
    /// Wait for element to disappear
    ElementHidden(String),
    /// Wait for URL to match pattern
    UrlMatches(String),
    /// Wait for network to be idle
    NetworkIdle,
    /// Wait for page load complete
    PageLoaded,
    /// Wait for specific text to appear
    TextPresent(String),
}

/// Smart selector with self-healing capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSelector {
    /// Primary selector value
    pub value: String,
    /// Type of selector
    pub selector_type: SelectorType,
    /// Priority (1-100, higher = more reliable)
    pub priority: u8,
    /// Alternative selectors for fallback
    pub alternatives: Vec<AlternativeSelector>,
    /// Last validation result
    pub validation_result: Option<ValidationResult>,
}

impl SmartSelector {
    /// Create a new CSS selector
    pub fn css(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            selector_type: SelectorType::Css,
            priority: 50,
            alternatives: Vec::new(),
            validation_result: None,
        }
    }

    /// Create a new XPath selector
    pub fn xpath(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            selector_type: SelectorType::XPath,
            priority: 50,
            alternatives: Vec::new(),
            validation_result: None,
        }
    }

    /// Create a stable ID-based selector
    pub fn id(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            value: format!("#{}", id),
            selector_type: SelectorType::Css,
            priority: 90, // IDs are highly reliable
            alternatives: Vec::new(),
            validation_result: None,
        }
    }

    /// Create a test-id selector (data-testid, data-cy, etc.)
    pub fn test_id(attr: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            value: format!("[{}=\"{}\"]", attr.into(), value.into()),
            selector_type: SelectorType::Css,
            priority: 95, // Test IDs are most reliable
            alternatives: Vec::new(),
            validation_result: None,
        }
    }

    /// Add an alternative selector
    pub fn with_alternative(mut self, alt: AlternativeSelector) -> Self {
        self.alternatives.push(alt);
        self
    }
}

/// Type of selector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectorType {
    /// CSS selector
    Css,
    /// XPath selector
    XPath,
    /// Text content match
    Text,
    /// ARIA label
    AriaLabel,
    /// Placeholder text
    Placeholder,
}

/// Alternative selector for fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeSelector {
    pub value: String,
    pub selector_type: SelectorType,
    pub priority: u8,
}

/// Result of selector validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the selector is valid
    pub is_valid: bool,
    /// Number of matching elements
    pub match_count: usize,
    /// Whether the element is visible
    pub is_visible: bool,
    /// Whether the element is interactable
    pub is_interactable: bool,
    /// When validation was performed
    pub validated_at: DateTime<Utc>,
}

// Helper functions for SecretString serialization
fn serialize_secret<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // In production, we might want to encrypt this
    serializer.serialize_str(secret.expose_secret())
}

fn deserialize_secret<'de, D>(deserializer: D) -> Result<SecretString, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(SecretString::new(s.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_profile_creation() {
        let profile = FlowProfile::new("Test Login", "https://example.com/login");
        assert_eq!(profile.name, "Test Login");
        assert_eq!(profile.start_url, "https://example.com/login");
        assert_eq!(profile.steps.len(), 0);
        assert_eq!(profile.status, ProfileStatus::Active);
    }

    #[test]
    fn test_add_steps() {
        let mut profile = FlowProfile::new("Checkout Flow", "https://shop.example.com");
        
        profile.add_step(FlowStep::Navigate {
            url: "https://shop.example.com/cart".to_string(),
            wait_for: None,
        });
        
        profile.add_step(FlowStep::Click {
            selector: SmartSelector::id("checkout-btn"),
            wait_for: Some("#payment-form".to_string()),
        });
        
        assert_eq!(profile.step_count(), 2);
    }

    #[test]
    fn test_smart_selector_creation() {
        let css = SmartSelector::css(".login-button");
        assert_eq!(css.selector_type, SelectorType::Css);
        assert_eq!(css.priority, 50);

        let test_id = SmartSelector::test_id("data-testid", "submit-btn");
        assert_eq!(test_id.priority, 95);
        assert!(test_id.value.contains("data-testid"));
    }

    #[test]
    fn test_flow_serialization() {
        let profile = FlowProfile::new("Test", "https://example.com");
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: FlowProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, profile.name);
    }
}
