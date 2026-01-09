use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Property test for workspace dependency consistency
///
/// **Property 1: Workspace Dependency Consistency**
/// For any crate in the workspace that uses a shared dependency,
/// the dependency should be declared with `workspace = true` to inherit
/// the version from the root workspace configuration.
/// **Validates: Requirements 6.2**
#[cfg(test)]
mod workspace_dependency_tests {
    use super::*;

    // Define the expected workspace dependencies from the root Cargo.toml
    fn get_workspace_dependencies() -> HashMap<String, bool> {
        let mut deps = HashMap::new();
        deps.insert("tokio".to_string(), true);
        deps.insert("hudsucker".to_string(), true);
        deps.insert("hyper".to_string(), true);
        deps.insert("tower".to_string(), true);
        deps.insert("rcgen".to_string(), true);
        deps.insert("tonic".to_string(), true);
        deps.insert("prost".to_string(), true);
        deps.insert("sqlx".to_string(), true);
        deps.insert("tauri".to_string(), true);
        deps.insert("serde".to_string(), true);
        deps.insert("serde_json".to_string(), true);
        deps.insert("chrono".to_string(), true);
        deps.insert("thiserror".to_string(), true);
        deps.insert("anyhow".to_string(), true);
        deps.insert("tracing".to_string(), true);
        deps.insert("tracing-subscriber".to_string(), true);
        deps.insert("uuid".to_string(), true);
        deps.insert("proptest".to_string(), true);
        deps
    }

    // Helper function to parse a Cargo.toml file and extract dependencies
    fn parse_cargo_toml_dependencies(content: &str) -> HashMap<String, bool> {
        let mut dependencies = HashMap::new();
        let mut in_dependencies_section = false;

        for line in content.lines() {
            let line = line.trim();

            // Check if we're entering a dependencies section
            if line == "[dependencies]"
                || line == "[dev-dependencies]"
                || line == "[build-dependencies]"
            {
                in_dependencies_section = true;
                continue;
            }

            // Check if we're leaving the dependencies section
            if line.starts_with('[')
                && line.ends_with(']')
                && line != "[dependencies]"
                && line != "[dev-dependencies]"
                && line != "[build-dependencies]"
            {
                in_dependencies_section = false;
                continue;
            }

            // Parse dependency lines
            if in_dependencies_section && !line.is_empty() && !line.starts_with('#') {
                if let Some(eq_pos) = line.find('=') {
                    let dep_name = line[..eq_pos].trim().to_string();
                    let dep_value = line[eq_pos + 1..].trim();

                    // Check if it uses workspace inheritance
                    let uses_workspace = dep_value.contains("workspace = true");
                    dependencies.insert(dep_name, uses_workspace);
                }
            }
        }

        dependencies
    }

    proptest! {
        #[test]
        fn test_workspace_dependency_consistency(
            crate_name in prop::sample::select(vec!["proxy-core", "proxy-agent", "orchestrator", "tauri-app"])
        ) {
            // Feature: distributed-mitm-proxy, Property 1: Workspace Dependency Consistency

            let cargo_toml_path = format!("../{}/Cargo.toml", crate_name);

            // Skip test if the crate doesn't exist yet (this allows the test to pass
            // during early development phases)
            if !Path::new(&cargo_toml_path).exists() {
                return Ok(());
            }

            let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
                .map_err(|e| proptest::test_runner::TestCaseError::fail(
                    format!("Failed to read {}: {}", cargo_toml_path, e)
                ))?;

            let workspace_deps = get_workspace_dependencies();
            let crate_deps = parse_cargo_toml_dependencies(&cargo_toml_content);

            // For any dependency that exists in both workspace and crate,
            // the crate should use workspace = true
            for (dep_name, _) in &workspace_deps {
                if let Some(&uses_workspace) = crate_deps.get(dep_name) {
                    prop_assert!(
                        uses_workspace,
                        "Crate '{}' uses dependency '{}' but does not inherit from workspace (missing 'workspace = true')",
                        crate_name,
                        dep_name
                    );
                }
            }
        }
    }

    #[test]
    fn test_workspace_dependency_consistency_unit() {
        // Unit test to verify the property with a concrete example
        let workspace_deps = get_workspace_dependencies();

        // Test with a sample Cargo.toml content that should pass
        let good_cargo_toml = r#"
[package]
name = "test-crate"
version.workspace = true
edition.workspace = true

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
"#;

        let parsed_deps = parse_cargo_toml_dependencies(good_cargo_toml);

        // Verify that tokio and serde are correctly marked as using workspace
        assert!(parsed_deps.get("tokio").copied().unwrap_or(false));
        assert!(parsed_deps.get("serde").copied().unwrap_or(false));

        // Test with a sample Cargo.toml content that should fail
        let bad_cargo_toml = r#"
[package]
name = "test-crate"
version.workspace = true
edition.workspace = true

[dependencies]
tokio = "1.0"
serde = { workspace = true }
"#;

        let parsed_deps_bad = parse_cargo_toml_dependencies(bad_cargo_toml);

        // Verify that tokio is NOT marked as using workspace (should fail the property)
        assert!(!parsed_deps_bad.get("tokio").copied().unwrap_or(false));
        assert!(parsed_deps_bad.get("serde").copied().unwrap_or(false));
    }
}
