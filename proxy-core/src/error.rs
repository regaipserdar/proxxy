//! Error types for proxy operations

use std::fmt;

/// Main error type for proxy operations
#[derive(Debug)]
pub enum ProxyError {
    /// Network-related errors
    Network(String),
    /// Certificate-related errors
    Certificate(String),
    /// Configuration errors
    Configuration(String),
    /// Database errors
    Database(String),
    /// HTTP processing errors
    Http(String),
    /// General I/O errors
    Io(std::io::Error),
    /// General errors
    General(String),
    /// Body capture errors
    BodyCapture(BodyCaptureError),
}

/// Error types specific to response body capture operations
#[derive(Debug, Clone)]
pub enum BodyCaptureError {
    /// Error reading from the response body stream
    StreamReadError(String),
    /// Memory allocation failed during body capture
    MemoryAllocationError,
    /// Database storage failed for captured body
    DatabaseStorageError(String),
    /// Invalid configuration provided
    ConfigurationError(String),
    /// Response body exceeded configured size limit
    SizeLimitExceeded(usize),
    /// Overall response timeout exceeded
    TimeoutError,
    /// Per-chunk stream read timeout exceeded
    StreamTimeoutError,
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::Network(msg) => write!(f, "Network error: {}", msg),
            ProxyError::Certificate(msg) => write!(f, "Certificate error: {}", msg),
            ProxyError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            ProxyError::Database(msg) => write!(f, "Database error: {}", msg),
            ProxyError::Http(msg) => write!(f, "HTTP error: {}", msg),
            ProxyError::Io(err) => write!(f, "I/O error: {}", err),
            ProxyError::General(msg) => write!(f, "General error: {}", msg),
            ProxyError::BodyCapture(err) => write!(f, "Body capture error: {}", err),
        }
    }
}

impl fmt::Display for BodyCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyCaptureError::StreamReadError(msg) => write!(f, "Stream read error: {}", msg),
            BodyCaptureError::MemoryAllocationError => write!(f, "Memory allocation failed"),
            BodyCaptureError::DatabaseStorageError(msg) => write!(f, "Database storage error: {}", msg),
            BodyCaptureError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            BodyCaptureError::SizeLimitExceeded(size) => write!(f, "Size limit exceeded: {} bytes", size),
            BodyCaptureError::TimeoutError => write!(f, "Response timeout exceeded"),
            BodyCaptureError::StreamTimeoutError => write!(f, "Stream read timeout exceeded"),
        }
    }
}

impl std::error::Error for ProxyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProxyError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl std::error::Error for BodyCaptureError {}

impl BodyCaptureError {
    /// Determines if the proxy should continue operating after this error
    pub fn should_continue_proxy(&self) -> bool {
        // All body capture errors should allow proxy to continue
        true
    }
    
    /// Returns a fallback empty body for when capture fails
    pub fn fallback_body(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        ProxyError::Io(err)
    }
}

impl From<BodyCaptureError> for ProxyError {
    fn from(err: BodyCaptureError) -> Self {
        ProxyError::BodyCapture(err)
    }
}
