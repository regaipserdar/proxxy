//! Replay Engine Module
//!
//! Executes recorded flows and extracts session data.

use crate::error::{FlowEngineError, FlowResult};
use crate::flow::model::{FlowProfile, FlowStep, WaitCondition, ExtractType};
use crate::flow::page::PageController;
use crate::flow::browser::{BrowserManager, BrowserOptions};
use secrecy::ExposeSecret;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Replay execution result
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Whether replay completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of steps completed
    pub steps_completed: usize,
    /// Total steps in profile
    pub total_steps: usize,
    /// Extracted session cookies (JSON)
    pub session_cookies: Option<String>,
    /// Extracted data from Extract steps
    pub extracted_data: HashMap<String, String>,
    /// Final page URL
    pub final_url: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Replay options
#[derive(Debug, Clone)]
pub struct ReplayOptions {
    /// Use headed browser (visible)
    pub headed: bool,
    /// Step-by-step mode with delays
    pub step_by_step: bool,
    /// Delay between steps in ms
    pub step_delay_ms: u64,
    /// Take screenshots on failure
    pub screenshot_on_failure: bool,
    /// Variable substitutions
    pub variables: HashMap<String, String>,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            headed: false,
            step_by_step: false,
            step_delay_ms: 500,
            screenshot_on_failure: true,
            variables: HashMap::new(),
        }
    }
}

/// Flow Replayer - executes recorded flows
pub struct FlowReplayer {
    browser_manager: BrowserManager,
    options: ReplayOptions,
}

impl FlowReplayer {
    /// Create a new replayer
    pub fn new() -> Self {
        Self {
            browser_manager: BrowserManager::new(),
            options: ReplayOptions::default(),
        }
    }

    /// Create replayer with options
    pub fn with_options(options: ReplayOptions) -> Self {
        Self {
            browser_manager: BrowserManager::new(),
            options,
        }
    }

    /// Execute a flow profile
    pub async fn execute(&self, profile: &FlowProfile) -> FlowResult<ReplayResult> {
        let start_time = std::time::Instant::now();
        let total_steps = profile.steps.len();

        info!("Starting replay: {} ({} steps)", profile.name, total_steps);

        // Launch browser
        let browser_opts = BrowserOptions::default()
            .headless(!self.options.headed);
        
        let browser_arc = self.browser_manager.launch(browser_opts).await?;

        // Get browser and create page
        let browser_guard = browser_arc.read().await;
        let managed = browser_guard.as_ref()
            .ok_or_else(|| FlowEngineError::BrowserLaunch("No browser available".to_string()))?;

        let page = managed.browser()
            .new_page("about:blank")
            .await
            .map_err(|e| FlowEngineError::BrowserLaunch(format!("Failed to create page: {}", e)))?;

        let controller = PageController::new(page);
        let mut extracted_data = HashMap::new();
        let mut steps_completed = 0;

        // Execute each step
        for (i, step) in profile.steps.iter().enumerate() {
            debug!("Executing step {}/{}: {:?}", i + 1, total_steps, step);

            if let Err(e) = self.execute_step(&controller, step, &mut extracted_data).await {
                warn!("Step {} failed: {}", i + 1, e);

                let duration_ms = start_time.elapsed().as_millis() as u64;
                let final_url = controller.get_url().await.ok();

                // Close browser
                drop(browser_guard);
                self.browser_manager.close().await.ok();

                return Ok(ReplayResult {
                    success: false,
                    error: Some(e.to_string()),
                    steps_completed,
                    total_steps,
                    session_cookies: None,
                    extracted_data,
                    final_url,
                    duration_ms,
                });
            }

            steps_completed += 1;

            // Delay between steps if configured
            if self.options.step_by_step {
                tokio::time::sleep(Duration::from_millis(self.options.step_delay_ms)).await;
            }
        }

        // Extract session cookies
        let cookies = self.extract_cookies(&controller).await.ok();
        let final_url = controller.get_url().await.ok();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Replay completed: {} steps in {}ms (cookies: {})",
            steps_completed,
            duration_ms,
            cookies.is_some()
        );

        // Close browser
        drop(browser_guard);
        self.browser_manager.close().await.ok();

        Ok(ReplayResult {
            success: true,
            error: None,
            steps_completed,
            total_steps,
            session_cookies: cookies,
            extracted_data,
            final_url,
            duration_ms,
        })
    }

    /// Execute a single step
    async fn execute_step(
        &self,
        controller: &PageController,
        step: &FlowStep,
        extracted_data: &mut HashMap<String, String>,
    ) -> FlowResult<()> {
        match step {
            FlowStep::Navigate { url, wait_for } => {
                let url = self.substitute_variables(url);
                controller.navigate(&url).await?;
                if let Some(selector) = wait_for {
                    controller.wait_for_selector(selector).await?;
                }
            }

            FlowStep::Click { selector, wait_for } => {
                controller.click(selector).await?;
                if let Some(sel) = wait_for {
                    controller.wait_for_selector(sel).await?;
                }
            }

            FlowStep::Type { selector, value, clear_first, .. } => {
                let text = self.substitute_variables(value.expose_secret());
                controller.type_text(selector, &text, *clear_first).await?;
            }

            FlowStep::Wait { duration_ms, condition } => {
                if let Some(cond) = condition {
                    controller.wait_for_condition(cond).await?;
                } else {
                    tokio::time::sleep(Duration::from_millis(*duration_ms)).await;
                }
            }

            FlowStep::CheckSession { validation_url, success_indicators } => {
                controller.navigate(validation_url).await?;
                let content = controller.get_page_content().await?;
                
                let is_valid = success_indicators.iter().any(|indicator| {
                    content.contains(indicator)
                });

                if !is_valid {
                    return Err(FlowEngineError::SessionValidation(
                        "Session validation failed: no success indicators found".to_string()
                    ));
                }
            }

            FlowStep::Submit { selector, wait_for_navigation } => {
                controller.click(selector).await?;
                if *wait_for_navigation {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    controller.wait_for_condition(&WaitCondition::PageLoaded).await?;
                }
            }

            FlowStep::Select { selector, value } => {
                // Select uses JavaScript to set the value
                let css = &selector.value;
                let script = format!(
                    r#"document.querySelector('{}').value = '{}'"#,
                    css, value
                );
                controller.execute_script(&script).await?;
            }

            FlowStep::Hover { selector } => {
                controller.hover(selector).await?;
            }

            FlowStep::KeyPress { key, modifiers } => {
                // Key press via JavaScript
                let mods = modifiers.join("+");
                let script = format!(
                    r#"
                    document.dispatchEvent(new KeyboardEvent('keydown', {{
                        key: '{}',
                        {}
                    }}))
                    "#,
                    key,
                    if !mods.is_empty() {
                        format!("ctrlKey: {}, altKey: {}, shiftKey: {}, metaKey: {}",
                            modifiers.contains(&"ctrl".to_string()),
                            modifiers.contains(&"alt".to_string()),
                            modifiers.contains(&"shift".to_string()),
                            modifiers.contains(&"meta".to_string())
                        )
                    } else {
                        String::new()
                    }
                );
                controller.execute_script(&script).await?;
            }

            FlowStep::Screenshot { filename } => {
                let data = controller.screenshot().await?;
                if let Some(name) = filename {
                    // In real impl, we'd save to file
                    debug!("Screenshot captured: {} ({} bytes)", name, data.len());
                }
            }

            FlowStep::Extract { selector, extract_type, variable_name } => {
                let value = match extract_type {
                    ExtractType::Text => controller.extract_text(selector).await?,
                    ExtractType::Value => {
                        let script = format!(
                            "document.querySelector('{}').value",
                            selector.value
                        );
                        controller.execute_script(&script).await?
                            .as_str()
                            .unwrap_or("")
                            .to_string()
                    }
                    ExtractType::Attribute(attr) => {
                        let script = format!(
                            "document.querySelector('{}').getAttribute('{}')",
                            selector.value, attr
                        );
                        controller.execute_script(&script).await?
                            .as_str()
                            .unwrap_or("")
                            .to_string()
                    }
                    ExtractType::InnerHtml | ExtractType::OuterHtml => {
                        let method = if matches!(extract_type, ExtractType::InnerHtml) {
                            "innerHTML"
                        } else {
                            "outerHTML"
                        };
                        let script = format!(
                            "document.querySelector('{}').{}",
                            selector.value, method
                        );
                        controller.execute_script(&script).await?
                            .as_str()
                            .unwrap_or("")
                            .to_string()
                    }
                };
                extracted_data.insert(variable_name.clone(), value);
            }

            FlowStep::ExecuteScript { script, result_variable } => {
                let result = controller.execute_script(script).await?;
                if let Some(var_name) = result_variable {
                    extracted_data.insert(var_name.clone(), result.to_string());
                }
            }

            FlowStep::Custom { action_type, parameters } => {
                debug!("Custom action: {} with {:?}", action_type, parameters);
                // Custom actions would be handled by extension points
            }
        }

        Ok(())
    }

    /// Extract cookies from the page
    async fn extract_cookies(&self, controller: &PageController) -> FlowResult<String> {
        let script = r#"
            document.cookie.split(';').map(c => {
                const [name, value] = c.trim().split('=');
                return { name, value };
            })
        "#;

        let result = controller.execute_script(script).await?;
        Ok(result.to_string())
    }

    /// Substitute variables in a string
    fn substitute_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in &self.options.variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

impl Default for FlowReplayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let mut options = ReplayOptions::default();
        options.variables.insert("username".to_string(), "testuser".to_string());
        options.variables.insert("password".to_string(), "secret123".to_string());

        let replayer = FlowReplayer::with_options(options);
        
        let result = replayer.substitute_variables("User: {{username}}, Pass: {{password}}");
        assert_eq!(result, "User: testuser, Pass: secret123");
    }

    #[test]
    fn test_replay_options() {
        let opts = ReplayOptions::default();
        assert!(!opts.headed);
        assert!(!opts.step_by_step);
        assert!(opts.screenshot_on_failure);
    }
}
