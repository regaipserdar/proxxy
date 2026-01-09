use std::fs;
use std::path::Path;

/// Unit tests for workspace configuration validation
///
/// Tests that validate the workspace structure, crate configurations,
/// and dependency requirements as specified in the requirements.
///
/// **Validates: Requirements 1.1, 1.2, 2.1-2.5, 3.1-3.3, 4.1-4.5, 5.1-5.4**
#[cfg(test)]
mod workspace_configuration_tests {
    use super::*;

    /// Test that root Cargo.toml contains all expected member crates
    /// **Validates: Requirements 1.1, 1.2**
    #[test]
    fn test_root_cargo_toml_contains_expected_members() {
        let root_cargo_path = "../Cargo.toml";
        assert!(
            Path::new(root_cargo_path).exists(),
            "Root Cargo.toml should exist"
        );

        let cargo_content =
            fs::read_to_string(root_cargo_path).expect("Should be able to read root Cargo.toml");

        // Expected member crates based on requirements
        let expected_members = vec![
            "proxy-core",
            "proxy-agent",
            "orchestrator",
            "UI", // Note: UI is the actual directory name for tauri-app
            "workspace-tests",
        ];

        // Verify workspace section exists
        assert!(
            cargo_content.contains("[workspace]"),
            "Root Cargo.toml should contain [workspace] section"
        );

        // Verify members section exists
        assert!(
            cargo_content.contains("members = ["),
            "Root Cargo.toml should contain members array"
        );

        // Verify each expected member is listed
        for member in expected_members {
            assert!(
                cargo_content.contains(&format!("\"{}\"", member)),
                "Root Cargo.toml should contain member: {}",
                member
            );
        }

        // Verify workspace resolver is set
        assert!(
            cargo_content.contains("resolver = \"2\""),
            "Root Cargo.toml should use resolver version 2"
        );
    }

    /// Test that each crate type is configured correctly
    /// **Validates: Requirements 2.1, 3.1, 4.1, 5.1**
    #[test]
    fn test_crate_types_configured_correctly() {
        // Test proxy-core as library crate (Requirement 2.1)
        test_library_crate_configuration("proxy-core");

        // Test proxy-agent as binary crate (Requirement 3.1)
        test_binary_crate_configuration("proxy-agent");

        // Test orchestrator as library crate with binary (Requirement 4.1)
        test_library_with_binary_crate_configuration("orchestrator");

        // Test UI (tauri-app) as Tauri application (Requirement 5.1)
        test_tauri_application_configuration("UI");
    }

    fn test_library_crate_configuration(crate_name: &str) {
        let cargo_path = format!("../{}/Cargo.toml", crate_name);
        let src_lib_path = format!("../{}/src/lib.rs", crate_name);

        assert!(
            Path::new(&cargo_path).exists(),
            "Crate {} should have Cargo.toml",
            crate_name
        );

        assert!(
            Path::new(&src_lib_path).exists(),
            "Library crate {} should have src/lib.rs",
            crate_name
        );

        let cargo_content = fs::read_to_string(&cargo_path)
            .expect(&format!("Should be able to read {}/Cargo.toml", crate_name));

        // Verify package section
        assert!(
            cargo_content.contains("[package]"),
            "Crate {} should have [package] section",
            crate_name
        );

        assert!(
            cargo_content.contains(&format!("name = \"{}\"", crate_name)),
            "Crate {} should have correct name in Cargo.toml",
            crate_name
        );
    }

    fn test_binary_crate_configuration(crate_name: &str) {
        let cargo_path = format!("../{}/Cargo.toml", crate_name);
        let src_main_path = format!("../{}/src/main.rs", crate_name);

        assert!(
            Path::new(&cargo_path).exists(),
            "Crate {} should have Cargo.toml",
            crate_name
        );

        assert!(
            Path::new(&src_main_path).exists(),
            "Binary crate {} should have src/main.rs",
            crate_name
        );

        let cargo_content = fs::read_to_string(&cargo_path)
            .expect(&format!("Should be able to read {}/Cargo.toml", crate_name));

        // Verify package section
        assert!(
            cargo_content.contains("[package]"),
            "Crate {} should have [package] section",
            crate_name
        );

        assert!(
            cargo_content.contains(&format!("name = \"{}\"", crate_name)),
            "Crate {} should have correct name in Cargo.toml",
            crate_name
        );
    }

    fn test_library_with_binary_crate_configuration(crate_name: &str) {
        let cargo_path = format!("../{}/Cargo.toml", crate_name);
        let src_lib_path = format!("../{}/src/lib.rs", crate_name);
        let src_main_path = format!("../{}/src/main.rs", crate_name);

        assert!(
            Path::new(&cargo_path).exists(),
            "Crate {} should have Cargo.toml",
            crate_name
        );

        assert!(
            Path::new(&src_lib_path).exists(),
            "Library crate {} should have src/lib.rs",
            crate_name
        );

        assert!(
            Path::new(&src_main_path).exists(),
            "Binary crate {} should have src/main.rs",
            crate_name
        );

        let cargo_content = fs::read_to_string(&cargo_path)
            .expect(&format!("Should be able to read {}/Cargo.toml", crate_name));

        // Verify both [lib] and [[bin]] sections exist
        assert!(
            cargo_content.contains("[lib]"),
            "Crate {} should have [lib] section",
            crate_name
        );

        assert!(
            cargo_content.contains("[[bin]]"),
            "Crate {} should have [[bin]] section",
            crate_name
        );
    }

    fn test_tauri_application_configuration(crate_name: &str) {
        let cargo_path = format!("../{}/Cargo.toml", crate_name);
        let src_main_path = format!("../{}/src/main.rs", crate_name);

        assert!(
            Path::new(&cargo_path).exists(),
            "Crate {} should have Cargo.toml",
            crate_name
        );

        assert!(
            Path::new(&src_main_path).exists(),
            "Tauri application {} should have src/main.rs",
            crate_name
        );

        let cargo_content = fs::read_to_string(&cargo_path)
            .expect(&format!("Should be able to read {}/Cargo.toml", crate_name));

        // Verify it has tauri dependency
        assert!(
            cargo_content.contains("tauri = {"),
            "Tauri application {} should have tauri dependency",
            crate_name
        );

        // Verify it has build-dependencies section with tauri-build
        assert!(
            cargo_content.contains("[build-dependencies]"),
            "Tauri application {} should have [build-dependencies] section",
            crate_name
        );

        assert!(
            cargo_content.contains("tauri-build = {"),
            "Tauri application {} should have tauri-build build dependency",
            crate_name
        );
    }

    /// Test that all required dependencies are present in each crate
    /// **Validates: Requirements 2.2-2.5, 3.2-3.3, 4.2-4.5, 5.2-5.4**
    #[test]
    fn test_required_dependencies_present() {
        test_proxy_core_dependencies();
        test_proxy_agent_dependencies();
        test_orchestrator_dependencies();
        test_tauri_app_dependencies();
    }

    fn test_proxy_core_dependencies() {
        let cargo_content = fs::read_to_string("../proxy-core/Cargo.toml")
            .expect("Should be able to read proxy-core/Cargo.toml");

        // Required dependencies for proxy-core (Requirements 2.2-2.5)
        let required_deps = vec![
            "hudsucker", // Requirement 2.2
            "hyper",     // Requirement 2.3
            "tower",     // Requirement 2.3
            "tokio",     // Requirement 2.4
            "rcgen",     // Requirement 2.5
        ];

        for dep in required_deps {
            assert!(
                cargo_content.contains(&format!("{} = {{", dep)),
                "proxy-core should have {} dependency",
                dep
            );

            // Verify workspace inheritance
            assert!(
                cargo_content.contains(&format!("{} = {{ workspace = true", dep)),
                "proxy-core should inherit {} from workspace",
                dep
            );
        }

        // Verify tokio has full features (Requirement 2.4)
        assert!(
            cargo_content.contains("tokio = { workspace = true, features = [\"full\"]")
                || cargo_content.contains("tokio = { workspace = true }"), // Features can be inherited from workspace
            "proxy-core should have tokio with full features"
        );
    }

    fn test_proxy_agent_dependencies() {
        let cargo_content = fs::read_to_string("../proxy-agent/Cargo.toml")
            .expect("Should be able to read proxy-agent/Cargo.toml");

        // Required dependencies for proxy-agent (Requirements 3.2-3.3)
        assert!(
            cargo_content.contains("proxy-core = { path = \"../proxy-core\" }"),
            "proxy-agent should depend on proxy-core library"
        );

        assert!(
            cargo_content.contains("tokio = { workspace = true }"),
            "proxy-agent should have tokio dependency for async execution"
        );
    }

    fn test_orchestrator_dependencies() {
        let cargo_content = fs::read_to_string("../orchestrator/Cargo.toml")
            .expect("Should be able to read orchestrator/Cargo.toml");

        // Required dependencies for orchestrator (Requirements 4.2-4.5)
        let required_deps = vec![
            "tonic", // Requirement 4.2 - gRPC server functionality
            "prost", // Requirement 4.3 - Protocol Buffers
            "sqlx",  // Requirement 4.4 - SQLite database operations
            "tokio", // Requirement 4.5 - async operations
        ];

        for dep in required_deps {
            assert!(
                cargo_content.contains(&format!("{} = {{ workspace = true", dep)),
                "orchestrator should have {} dependency with workspace inheritance",
                dep
            );
        }

        // Verify sqlx has SQLite features (Requirement 4.4)
        assert!(
            cargo_content.contains(
                "sqlx = { workspace = true, features = [\"runtime-tokio-rustls\", \"sqlite\"]"
            ) || cargo_content.contains("sqlx = { workspace = true }"), // Features can be inherited from workspace
            "orchestrator should have sqlx with SQLite features"
        );
    }

    fn test_tauri_app_dependencies() {
        let cargo_content =
            fs::read_to_string("../UI/Cargo.toml").expect("Should be able to read UI/Cargo.toml");

        // Required dependencies for tauri-app (Requirements 5.2-5.4)
        assert!(
            cargo_content.contains("tauri = { workspace = true }"),
            "tauri-app should have tauri dependency with appropriate features"
        );

        assert!(
            cargo_content.contains("orchestrator = { path = \"../orchestrator\" }"),
            "tauri-app should depend on orchestrator library"
        );

        assert!(
            cargo_content.contains("tokio = { workspace = true }"),
            "tauri-app should have tokio dependency for async operations"
        );

        // Verify build dependencies
        assert!(
            cargo_content.contains("tauri-build = { workspace = true }"),
            "tauri-app should have tauri-build build dependency"
        );
    }

    /// Test workspace-level dependency definitions
    /// **Validates: Requirements 6.1, 6.2**
    #[test]
    fn test_workspace_dependency_definitions() {
        let root_cargo_content =
            fs::read_to_string("../Cargo.toml").expect("Should be able to read root Cargo.toml");

        // Verify workspace.dependencies section exists
        assert!(
            root_cargo_content.contains("[workspace.dependencies]"),
            "Root Cargo.toml should have [workspace.dependencies] section"
        );

        // Required workspace dependencies
        let required_workspace_deps = vec![
            "tokio",
            "hudsucker",
            "hyper",
            "tower",
            "rcgen",
            "tonic",
            "prost",
            "sqlx",
            "tauri",
            "tauri-build",
            "serde",
            "serde_json",
            "chrono",
            "thiserror",
            "anyhow",
            "tracing",
            "tracing-subscriber",
            "uuid",
            "proptest",
        ];

        for dep in required_workspace_deps {
            assert!(
                root_cargo_content.contains(&format!("{} = ", dep)),
                "Workspace should define {} dependency",
                dep
            );
        }

        // Verify tokio has full features
        assert!(
            root_cargo_content.contains("tokio = { version = \"1.0\", features = [\"full\"] }"),
            "Workspace should define tokio with full features"
        );

        // Verify sqlx has SQLite features
        assert!(
            root_cargo_content.contains(
                "sqlx = { version = \"0.7\", features = [\"runtime-tokio-rustls\", \"sqlite\"] }"
            ),
            "Workspace should define sqlx with SQLite features"
        );
    }

    /// Test workspace package configuration
    /// **Validates: Requirements 1.4**
    #[test]
    fn test_workspace_package_configuration() {
        let root_cargo_content =
            fs::read_to_string("../Cargo.toml").expect("Should be able to read root Cargo.toml");

        // Verify workspace.package section exists
        assert!(
            root_cargo_content.contains("[workspace.package]"),
            "Root Cargo.toml should have [workspace.package] section"
        );

        // Verify consistent version and edition
        assert!(
            root_cargo_content.contains("version = \"0.1.0\""),
            "Workspace should define consistent version"
        );

        assert!(
            root_cargo_content.contains("edition = \"2021\""),
            "Workspace should use consistent Rust edition 2021"
        );
    }

    /// Test that crates use workspace inheritance for version and edition
    /// **Validates: Requirements 1.4**
    #[test]
    fn test_crate_workspace_inheritance() {
        let crates_to_check = vec!["proxy-agent", "UI"];

        for crate_name in crates_to_check {
            let cargo_path = format!("../{}/Cargo.toml", crate_name);
            let cargo_content = fs::read_to_string(&cargo_path)
                .expect(&format!("Should be able to read {}/Cargo.toml", crate_name));

            // Check for workspace inheritance of version and edition
            assert!(
                cargo_content.contains("version.workspace = true")
                    || cargo_content.contains("version = \"0.1.0\""), // Direct version is also acceptable
                "Crate {} should inherit version from workspace or define it directly",
                crate_name
            );

            assert!(
                cargo_content.contains("edition.workspace = true")
                    || cargo_content.contains("edition = \"2021\""), // Direct edition is also acceptable
                "Crate {} should inherit edition from workspace or define it directly",
                crate_name
            );
        }
    }
}
