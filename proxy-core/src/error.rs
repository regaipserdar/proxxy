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

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        ProxyError::Io(err)
    }
}
