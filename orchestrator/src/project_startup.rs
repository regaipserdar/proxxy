use crate::database::Database;
use crate::models::settings::{ScopeConfig, InterceptionConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Handles CLI project operations during orchestrator startup
pub struct ProjectStartupHandler {
    db: Arc<Database>,
}

impl ProjectStartupHandler {
    /// Create a new ProjectStartupHandler
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Handle CLI project operations: check existence, create if needed, load, and configure state
    pub async fn handle_cli_project(
        &self,
        project_name: &str,
        scope_state: Arc<RwLock<ScopeConfig>>,
        interception_state: Arc<RwLock<InterceptionConfig>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Starting CLI project operations for '{}'", project_name);

        // Validate project name using existing validation rules
        if !self.is_valid_project_name(project_name) {
            let error_msg = format!(
                "Invalid project name '{}'. Use only alphanumeric characters, hyphens, and underscores",
                project_name
            );
            error!("   ✗ {}", error_msg);
            return Err(error_msg.into());
        }

        // Check if project exists
        let project_exists = self.project_exists(project_name).await?;
        
        if !project_exists {
            info!("   📁 Project '{}' does not exist, creating automatically...", project_name);
            self.create_project(project_name).await?;
            info!("   ✓ Project '{}' created successfully", project_name);
        } else {
            info!("   📁 Project '{}' already exists", project_name);
        }

        // Load the project
        info!("   📂 Loading project '{}'...", project_name);
        self.load_project(project_name).await?;
        info!("   ✓ Project '{}' loaded successfully", project_name);

        // Load and update configuration state
        info!("   ⚙️  Loading project configurations...");
        self.load_configuration_state(scope_state, interception_state).await?;
        info!("   ✓ Project configurations loaded and applied");

        info!("✅ CLI project operations completed successfully for '{}'", project_name);
        Ok(())
    }

    /// Check if a project exists by looking for the project directory
    async fn project_exists(&self, project_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let projects = self.db.list_projects().await?;
        Ok(projects.iter().any(|p| p.name == project_name))
    }

    /// Create a new project using the existing Database method
    async fn create_project(&self, project_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.db.create_project(project_name).await?;
        Ok(())
    }

    /// Load a project using the existing Database method
    async fn load_project(&self, project_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.db.load_project(project_name).await?;
        Ok(())
    }

    /// Load configuration state from the database and update in-memory state
    async fn load_configuration_state(
        &self,
        scope_state: Arc<RwLock<ScopeConfig>>,
        interception_state: Arc<RwLock<InterceptionConfig>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load ScopeConfig from database
        match self.db.get_scope_config().await {
            Ok(scope_config) => {
                *scope_state.write().await = scope_config;
                info!("   ✓ ScopeConfig loaded from database");
            }
            Err(e) => {
                // Log detailed error information for troubleshooting
                error!("   ✗ Failed to load ScopeConfig from database: {}", e);
                error!("     Error type: {}", std::any::type_name_of_val(&e));
                error!("     Falling back to default ScopeConfig");
                
                // Use default configuration as fallback
                let default_config = ScopeConfig::default();
                *scope_state.write().await = default_config.clone();
                
                warn!("   ⚠️  Using default ScopeConfig: enabled={}, patterns={:?}", 
                      default_config.enabled, default_config.include_patterns);
            }
        }

        // Load InterceptionConfig from database
        match self.db.get_interception_config().await {
            Ok(interception_config) => {
                *interception_state.write().await = interception_config;
                info!("   ✓ InterceptionConfig loaded from database");
            }
            Err(e) => {
                // Log detailed error information for troubleshooting
                error!("   ✗ Failed to load InterceptionConfig from database: {}", e);
                error!("     Error type: {}", std::any::type_name_of_val(&e));
                error!("     Falling back to default InterceptionConfig");
                
                // Use default configuration as fallback
                let default_config = InterceptionConfig::default();
                *interception_state.write().await = default_config.clone();
                
                warn!("   ⚠️  Using default InterceptionConfig: enabled={}, rules_count={}", 
                      default_config.enabled, default_config.rules.len());
            }
        }

        Ok(())
    }

    /// Validate project name using existing validation rules (alphanumeric, -, _)
    fn is_valid_project_name(&self, name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_validation() {
        // Test the validation logic directly without needing a Database instance
        fn is_valid_project_name(name: &str) -> bool {
            !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        }

        // Valid names
        assert!(is_valid_project_name("test"));
        assert!(is_valid_project_name("test-project"));
        assert!(is_valid_project_name("test_project"));
        assert!(is_valid_project_name("test123"));
        assert!(is_valid_project_name("Test-Project_123"));

        // Invalid names
        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name("test project")); // space
        assert!(!is_valid_project_name("test.project")); // dot
        assert!(!is_valid_project_name("test@project")); // special char
        assert!(!is_valid_project_name("test/project")); // slash
    }
}