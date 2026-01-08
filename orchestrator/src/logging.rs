use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use crate::OrchestratorError;

/// Logging configuration for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Whether to enable JSON formatted logs
    pub json_format: bool,
    
    /// Whether to include timestamps in logs
    pub include_timestamp: bool,
    
    /// Whether to include thread names in logs
    pub include_thread_names: bool,
    
    /// Whether to include file and line number information
    pub include_file_info: bool,
    
    /// Whether to enable span events (enter/exit)
    pub enable_span_events: bool,
    
    /// Whether to enable colored output (only for non-JSON format)
    pub enable_colors: bool,
    
    /// Log file path (optional, if None logs only to stdout)
    pub log_file: Option<String>,
    
    /// Maximum log file size in MB before rotation
    pub max_file_size_mb: u64,
    
    /// Number of rotated log files to keep
    pub max_files: u32,
    
    /// Module-specific log levels
    pub module_levels: std::collections::HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut module_levels = std::collections::HashMap::new();
        
        // Set default levels for common modules
        module_levels.insert("orchestrator".to_string(), "info".to_string());
        module_levels.insert("sqlx".to_string(), "warn".to_string());
        module_levels.insert("tonic".to_string(), "info".to_string());
        module_levels.insert("hyper".to_string(), "warn".to_string());
        module_levels.insert("tokio".to_string(), "warn".to_string());
        
        Self {
            level: "info".to_string(),
            json_format: false,
            include_timestamp: true,
            include_thread_names: true,
            include_file_info: false,
            enable_span_events: false,
            enable_colors: true,
            log_file: None,
            max_file_size_mb: 100,
            max_files: 5,
            module_levels,
        }
    }
}

/// Initialize logging based on the provided configuration
pub fn init_logging(config: &LoggingConfig) -> Result<(), OrchestratorError> {
    // Build the environment filter
    let mut filter = EnvFilter::new(&config.level);
    
    // Add module-specific filters
    for (module, level) in &config.module_levels {
        let directive = format!("{}={}", module, level);
        filter = filter.add_directive(directive.parse()
            .map_err(|e| OrchestratorError::Logging(format!("Invalid log directive: {}", e)))?);
    }
    
    // Try to initialize logging, ignore if already initialized
    let result = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_names(config.include_thread_names)
                .with_file(config.include_file_info)
                .with_line_number(config.include_file_info)
                .with_ansi(config.enable_colors)
        )
        .try_init();
    
    match result {
        Ok(_) => {
            tracing::info!("Logging initialized with config level: {}", config.level);
        }
        Err(_) => {
            // Logging already initialized, that's fine
            tracing::debug!("Logging already initialized, skipping");
        }
    }
    
    Ok(())
}

/// Create a file appender for log rotation
fn create_file_appender(
    log_file: &str,
    _config: &LoggingConfig,
) -> Result<tracing_appender::rolling::RollingFileAppender, OrchestratorError> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use std::path::Path;
    
    let log_path = Path::new(log_file);
    let directory = log_path.parent()
        .ok_or_else(|| OrchestratorError::Logging("Invalid log file path".to_string()))?;
    let filename = log_path.file_name()
        .ok_or_else(|| OrchestratorError::Logging("Invalid log file name".to_string()))?
        .to_string_lossy();
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(directory)
        .map_err(|e| OrchestratorError::Logging(format!("Failed to create log directory: {}", e)))?;
    
    // For simplicity, we'll use daily rotation
    // In a production system, you might want size-based rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        directory,
        filename.as_ref(),
    );
    
    Ok(file_appender)
}

/// Log level utilities
pub mod levels {
    /// Check if a log level string is valid
    pub fn is_valid_level(level: &str) -> bool {
        matches!(level.to_lowercase().as_str(), "trace" | "debug" | "info" | "warn" | "error")
    }
    
    /// Get all valid log levels
    pub fn valid_levels() -> Vec<&'static str> {
        vec!["trace", "debug", "info", "warn", "error"]
    }
    
    /// Convert log level to numeric value for comparison
    pub fn level_to_numeric(level: &str) -> Option<u8> {
        match level.to_lowercase().as_str() {
            "trace" => Some(0),
            "debug" => Some(1),
            "info" => Some(2),
            "warn" => Some(3),
            "error" => Some(4),
            _ => None,
        }
    }
}

/// Structured logging macros for common orchestrator events
#[macro_export]
macro_rules! log_agent_event {
    ($level:ident, $agent_id:expr, $event:expr, $($field:ident = $value:expr),*) => {
        tracing::$level!(
            agent_id = $agent_id,
            event = $event,
            $($field = $value,)*
            "Agent event"
        );
    };
}

#[macro_export]
macro_rules! log_traffic_event {
    ($level:ident, $agent_id:expr, $method:expr, $url:expr, $status:expr, $($field:ident = $value:expr),*) => {
        tracing::$level!(
            agent_id = $agent_id,
            method = $method,
            url = $url,
            status = $status,
            $($field = $value,)*
            "Traffic event"
        );
    };
}

#[macro_export]
macro_rules! log_database_event {
    ($level:ident, $operation:expr, $table:expr, $($field:ident = $value:expr),*) => {
        tracing::$level!(
            operation = $operation,
            table = $table,
            $($field = $value,)*
            "Database event"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.json_format);
        assert!(config.include_timestamp);
        assert!(config.include_thread_names);
        assert!(!config.include_file_info);
        assert!(!config.enable_span_events);
        assert!(config.enable_colors);
        assert!(config.log_file.is_none());
        assert_eq!(config.max_file_size_mb, 100);
        assert_eq!(config.max_files, 5);
        assert!(!config.module_levels.is_empty());
    }
    
    #[test]
    fn test_log_level_validation() {
        assert!(levels::is_valid_level("info"));
        assert!(levels::is_valid_level("DEBUG"));
        assert!(levels::is_valid_level("Error"));
        assert!(!levels::is_valid_level("invalid"));
        assert!(!levels::is_valid_level(""));
    }
    
    #[test]
    fn test_log_level_numeric_conversion() {
        assert_eq!(levels::level_to_numeric("trace"), Some(0));
        assert_eq!(levels::level_to_numeric("debug"), Some(1));
        assert_eq!(levels::level_to_numeric("info"), Some(2));
        assert_eq!(levels::level_to_numeric("warn"), Some(3));
        assert_eq!(levels::level_to_numeric("error"), Some(4));
        assert_eq!(levels::level_to_numeric("invalid"), None);
    }
    
    #[test]
    fn test_valid_levels_list() {
        let levels = levels::valid_levels();
        assert_eq!(levels.len(), 5);
        assert!(levels.contains(&"info"));
        assert!(levels.contains(&"debug"));
        assert!(levels.contains(&"error"));
    }
}