use clap::Parser;
use proptest::prelude::*;

/// CLI Arguments structure for testing
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// gRPC server port for agent connections
    #[arg(long, default_value_t = 50051)]
    grpc_port: u16,

    /// HTTP API port for REST/GraphQL endpoints
    #[arg(long, default_value_t = 9090)]
    http_port: u16,

    /// Database connection URL
    #[arg(long, default_value = "sqlite:./proxxy.db")]
    database_url: String,

    /// Health check interval in seconds
    #[arg(long, default_value_t = 30)]
    health_check_interval: u64,

    /// Agent timeout in seconds
    #[arg(long, default_value_t = 300)]
    agent_timeout: u64,

    /// Project name to auto-load on startup
    #[arg(long, short = 'p')]
    project: Option<String>,
}

// Generator for valid project names (alphanumeric, hyphens, underscores)
// Note: Project names cannot start with hyphens to avoid confusion with CLI flags
fn arb_valid_project_name() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            prop::char::range('a', 'z'),
            prop::char::range('A', 'Z'), 
            prop::char::range('0', '9'),
            Just('-'),
            Just('_')
        ],
        1..=50
    ).prop_map(|chars: Vec<char>| chars.into_iter().collect())
    .prop_filter("Project names cannot start with hyphens", |name: &String| {
        !name.starts_with('-')
    })
}

proptest! {
    /// **Feature: cli-project-support, Property 1: CLI argument parsing**
    /// **Validates: Requirements 1.1, 1.2**
    /// For any valid project name provided via `--project` or `-p` arguments, 
    /// the orchestrator should parse and accept the project name parameter
    #[test]
    fn prop_cli_argument_parsing_long_form(project_name in arb_valid_project_name()) {
        // Test --project long form
        let args = vec!["orchestrator", "--project", &project_name];
        let parsed = Args::try_parse_from(args);
        
        prop_assert!(parsed.is_ok(), "Failed to parse --project argument with valid project name: {}", project_name);
        
        let args_struct = parsed.unwrap();
        prop_assert_eq!(args_struct.project, Some(project_name.clone()));
        
        // Verify other defaults are preserved
        prop_assert_eq!(args_struct.grpc_port, 50051);
        prop_assert_eq!(args_struct.http_port, 9090);
        prop_assert_eq!(args_struct.database_url, "sqlite:./proxxy.db");
        prop_assert_eq!(args_struct.health_check_interval, 30);
        prop_assert_eq!(args_struct.agent_timeout, 300);
    }

    /// **Feature: cli-project-support, Property 1: CLI argument parsing**
    /// **Validates: Requirements 1.1, 1.2**
    /// For any valid project name provided via `-p` short form argument,
    /// the orchestrator should parse and accept the project name parameter
    #[test]
    fn prop_cli_argument_parsing_short_form(project_name in arb_valid_project_name()) {
        // Test -p short form
        let args = vec!["orchestrator", "-p", &project_name];
        let parsed = Args::try_parse_from(args);
        
        prop_assert!(parsed.is_ok(), "Failed to parse -p argument with valid project name: {}", project_name);
        
        let args_struct = parsed.unwrap();
        prop_assert_eq!(args_struct.project, Some(project_name.clone()));
        
        // Verify other defaults are preserved
        prop_assert_eq!(args_struct.grpc_port, 50051);
        prop_assert_eq!(args_struct.http_port, 9090);
        prop_assert_eq!(args_struct.database_url, "sqlite:./proxxy.db");
        prop_assert_eq!(args_struct.health_check_interval, 30);
        prop_assert_eq!(args_struct.agent_timeout, 300);
    }

    /// **Feature: cli-project-support, Property 1: CLI argument parsing**
    /// **Validates: Requirements 1.1, 1.2**
    /// For any orchestrator startup without CLI project arguments,
    /// the system should parse successfully with project field as None
    #[test]
    fn prop_cli_argument_parsing_no_project(_dummy in 0u8..1u8) {
        // Test without project argument
        let args = vec!["orchestrator"];
        let parsed = Args::try_parse_from(args);
        
        prop_assert!(parsed.is_ok(), "Failed to parse arguments without project");
        
        let args_struct = parsed.unwrap();
        prop_assert_eq!(args_struct.project, None);
        
        // Verify defaults are preserved
        prop_assert_eq!(args_struct.grpc_port, 50051);
        prop_assert_eq!(args_struct.http_port, 9090);
        prop_assert_eq!(args_struct.database_url, "sqlite:./proxxy.db");
        prop_assert_eq!(args_struct.health_check_interval, 30);
        prop_assert_eq!(args_struct.agent_timeout, 300);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_help_includes_project_argument() {
        // Test that help text includes project argument
        let result = Args::try_parse_from(vec!["orchestrator", "--help"]);
        
        // This should fail with help text
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let help_text = error.to_string();
        
        // Verify project argument appears in help
        assert!(help_text.contains("--project"), "Help text should contain --project");
        assert!(help_text.contains("-p"), "Help text should contain -p short form");
        assert!(help_text.contains("Project name to auto-load on startup"), "Help text should contain project description");
    }

    #[test]
    fn test_specific_project_names() {
        // Test specific valid project names
        let valid_names = vec![
            "test-project",
            "my_project",
            "Project123",
            "a",
            "test_project-123",
        ];

        for name in valid_names {
            let args = vec!["orchestrator", "--project", name];
            let parsed = Args::try_parse_from(args);
            assert!(parsed.is_ok(), "Should parse valid project name: {}", name);
            assert_eq!(parsed.unwrap().project, Some(name.to_string()));
        }
    }

    #[test]
    fn test_project_with_other_args() {
        // Test project argument combined with other arguments
        let args = vec![
            "orchestrator", 
            "--project", "test-project",
            "--grpc-port", "8080",
            "--http-port", "9091"
        ];
        let parsed = Args::try_parse_from(args);
        
        assert!(parsed.is_ok(), "Should parse project with other arguments");
        let args_struct = parsed.unwrap();
        assert_eq!(args_struct.project, Some("test-project".to_string()));
        assert_eq!(args_struct.grpc_port, 8080);
        assert_eq!(args_struct.http_port, 9091);
    }
}