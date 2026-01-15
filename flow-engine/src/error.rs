//! Flow Engine Error Types

use thiserror::Error;

/// Main error type for the flow engine
#[derive(Debug, Error)]
pub enum FlowEngineError {
    #[error("Browser launch failed: {0}")]
    BrowserLaunch(String),

    #[error("Browser connection failed: {0}")]
    BrowserConnection(String),

    #[error("Page navigation failed: {0}")]
    Navigation(String),

    #[error("Element not found: {selector}")]
    ElementNotFound { selector: String },

    #[error("Selector generation failed: {0}")]
    SelectorGeneration(String),

    #[error("Recording error: {0}")]
    Recording(String),

    #[error("Replay error: {0}")]
    Replay(String),

    #[error("HAR processing error: {0}")]
    HarProcessing(String),

    #[error("Cookie extraction failed: {0}")]
    CookieExtraction(String),

    #[error("Session validation failed: {0}")]
    SessionValidation(String),

    #[error("Flow profile not found: {id}")]
    ProfileNotFound { id: String },

    #[error("Timeout waiting for {condition}: {details}")]
    Timeout { condition: String, details: String },

    #[error("User intervention required: {reason}")]
    InterventionRequired { reason: String },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for flow engine operations
pub type FlowResult<T> = Result<T, FlowEngineError>;
