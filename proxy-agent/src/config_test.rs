#[cfg(test)]
mod tests {
    use crate::{Args, load_body_capture_config};
    use std::env;
    use tempfile::NamedTempFile;
    use std::io::Write;

    // Helper function to clear all environment variables that might affect tests
    fn clear_env_vars() {
        env::remove_var("PROXXY_BODY_CAPTURE_ENABLED");
        env::remove_var("PROXXY_MAX_BODY_SIZE");
        env::remove_var("PROXXY_MEMORY_LIMIT");
        env::remove_var("PROXXY_MAX_CONCURRENT_CAPTURES");
        env::remove_var("PROXXY_RESPONSE_TIMEOUT");
        env::remove_var("PROXXY_STREAM_TIMEOUT");
        env::remove_var("PROXXY_CONTENT_TYPE_FILTERS");
        env::remove_var("PROXXY_CONTENT_TYPE_MODE");
    }

    #[test]
    fn test_load_body_capture_config_defaults() {
        clear_env_vars();

        // Test loading with no configuration sources
        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: None,
            max_body_size: None,
            response_timeout: None,
            stream_timeout: None,
        };

        let config = load_body_capture_config(&args).unwrap();
        
        // Should use default values
        assert_eq!(config.enabled, true);
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.memory_limit, 100 * 1024 * 1024);
        assert_eq!(config.response_timeout_secs, 30);
        assert_eq!(config.stream_read_timeout_secs, 5);
    }

    #[test]
    fn test_load_body_capture_config_from_cli() {
        clear_env_vars();

        // Test CLI argument override
        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: Some(false),
            max_body_size: Some(5 * 1024 * 1024), // 5MB
            response_timeout: Some(60),
            stream_timeout: Some(10),
        };

        let config = load_body_capture_config(&args).unwrap();
        
        // Should use CLI values
        assert_eq!(config.enabled, false);
        assert_eq!(config.max_body_size, 5 * 1024 * 1024);
        assert_eq!(config.response_timeout_secs, 60);
        assert_eq!(config.stream_read_timeout_secs, 10);
    }

    #[test]
    fn test_load_body_capture_config_from_env() {
        clear_env_vars();

        // Set environment variables
        env::set_var("PROXXY_BODY_CAPTURE_ENABLED", "false");
        env::set_var("PROXXY_MAX_BODY_SIZE", "2097152"); // 2MB
        env::set_var("PROXXY_RESPONSE_TIMEOUT", "45");
        env::set_var("PROXXY_STREAM_TIMEOUT", "8");

        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: None,
            max_body_size: None,
            response_timeout: None,
            stream_timeout: None,
        };

        let config = load_body_capture_config(&args).unwrap();
        
        // Should use environment values
        assert_eq!(config.enabled, false);
        assert_eq!(config.max_body_size, 2097152);
        assert_eq!(config.response_timeout_secs, 45);
        assert_eq!(config.stream_read_timeout_secs, 8);

        clear_env_vars();
    }

    #[test]
    fn test_load_body_capture_config_from_file() {
        clear_env_vars();

        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"{
            "enabled": false,
            "max_body_size": 1048576,
            "truncate_threshold": 524288,
            "memory_limit": 10485760,
            "max_concurrent_captures": 3,
            "content_type_filters": ["json", "xml"],
            "content_type_filter_mode": "Whitelist",
            "response_timeout_secs": 20,
            "stream_read_timeout_secs": 3
        }"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: Some(temp_file.path().to_path_buf()),
            enable_body_capture: None,
            max_body_size: None,
            response_timeout: None,
            stream_timeout: None,
        };

        let config = load_body_capture_config(&args).unwrap();
        
        // Should use file values
        assert_eq!(config.enabled, false);
        assert_eq!(config.max_body_size, 1048576);
        assert_eq!(config.response_timeout_secs, 20);
        assert_eq!(config.stream_read_timeout_secs, 3);
        assert_eq!(config.max_concurrent_captures, 3);
        assert_eq!(config.content_type_filters, vec!["json", "xml"]);
    }

    #[test]
    fn test_load_body_capture_config_precedence() {
        clear_env_vars();

        // Test that CLI args override environment variables
        env::set_var("PROXXY_BODY_CAPTURE_ENABLED", "false");
        env::set_var("PROXXY_MAX_BODY_SIZE", "2097152");

        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: Some(true), // CLI override
            max_body_size: Some(4194304),    // CLI override
            response_timeout: None,
            stream_timeout: None,
        };

        let config = load_body_capture_config(&args).unwrap();
        
        // CLI should override environment
        assert_eq!(config.enabled, true);  // CLI value, not env
        assert_eq!(config.max_body_size, 4194304); // CLI value, not env

        clear_env_vars();
    }

    #[test]
    fn test_load_body_capture_config_invalid_env_values() {
        clear_env_vars();

        // Test invalid environment variable values
        env::set_var("PROXXY_BODY_CAPTURE_ENABLED", "invalid");

        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: None,
            max_body_size: None,
            response_timeout: None,
            stream_timeout: None,
        };

        let result = load_body_capture_config(&args);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid PROXXY_BODY_CAPTURE_ENABLED"));
        }

        clear_env_vars();
    }

    #[test]
    fn test_load_body_capture_config_validation_failure() {
        clear_env_vars();

        // Test configuration that fails validation
        let args = Args {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 9095,
            admin_port: 9091,
            orchestrator_url: "http://127.0.0.1:50051".to_string(),
            name: None,
            ca_cert: None,
            ca_key: None,
            body_capture_config: None,
            enable_body_capture: None,
            max_body_size: None,
            response_timeout: Some(0), // Invalid - zero timeout
            stream_timeout: None,
        };

        let result = load_body_capture_config(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("validation failed"));
    }
}