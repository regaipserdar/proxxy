//! Page Controller Module
//!
//! Manages page interactions, navigation, and element operations.

use crate::error::{FlowEngineError, FlowResult};
use crate::flow::model::{SmartSelector, SelectorType, WaitCondition};
use chromiumoxide::Page;
use std::time::Duration;
use tracing::{debug, info, warn, error};

/// Default timeout for operations
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Page controller for browser automation
pub struct PageController {
    page: Page,
    default_timeout: Duration,
}

impl PageController {
    /// Create a new page controller
    pub fn new(page: Page) -> Self {
        Self {
            page,
            default_timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set default timeout for operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Get the underlying page
    pub fn page(&self) -> &Page {
        &self.page
    }

    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> FlowResult<()> {
        info!("Navigating to: {}", url);
        
        self.page
            .goto(url)
            .await
            .map_err(|e| FlowEngineError::Navigation(format!("Failed to navigate to {}: {}", url, e)))?;

        Ok(())
    }

    /// Navigate and wait for a selector
    pub async fn navigate_and_wait(&self, url: &str, wait_selector: &str) -> FlowResult<()> {
        self.navigate(url).await?;
        self.wait_for_selector(wait_selector).await?;
        Ok(())
    }

    /// Wait for a selector to appear
    pub async fn wait_for_selector(&self, selector: &str) -> FlowResult<()> {
        debug!("Waiting for selector: {}", selector);
        
        self.page
            .find_element(selector)
            .await
            .map_err(|e| FlowEngineError::ElementNotFound { 
                selector: format!("{} ({})", selector, e) 
            })?;

        Ok(())
    }

    /// Wait for a condition
    pub async fn wait_for_condition(&self, condition: &WaitCondition) -> FlowResult<()> {
        match condition {
            WaitCondition::ElementVisible(selector) => {
                self.wait_for_selector(selector).await
            }
            WaitCondition::ElementHidden(selector) => {
                let start = std::time::Instant::now();
                while start.elapsed() < self.default_timeout {
                    if self.page.find_element(selector).await.is_err() {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(FlowEngineError::Timeout {
                    condition: "ElementHidden".to_string(),
                    details: format!("Element {} still visible", selector),
                })
            }
            WaitCondition::UrlMatches(pattern) => {
                let start = std::time::Instant::now();
                while start.elapsed() < self.default_timeout {
                    let url = self.get_url().await?;
                    if url.contains(pattern) {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(FlowEngineError::Timeout {
                    condition: "UrlMatches".to_string(),
                    details: format!("URL does not match pattern: {}", pattern),
                })
            }
            WaitCondition::NetworkIdle => {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            WaitCondition::PageLoaded => {
                self.page
                    .evaluate("document.readyState === 'complete'")
                    .await
                    .map_err(|e| FlowEngineError::Navigation(format!("Page load check failed: {}", e)))?;
                Ok(())
            }
            WaitCondition::TextPresent(text) => {
                let start = std::time::Instant::now();
                while start.elapsed() < self.default_timeout {
                    let content = self.get_page_content().await?;
                    if content.contains(text) {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(FlowEngineError::Timeout {
                    condition: "TextPresent".to_string(),
                    details: format!("Text '{}' not found", text),
                })
            }
        }
    }

    /// Click an element using smart selector
    pub async fn click(&self, selector: &SmartSelector) -> FlowResult<()> {
        let css = self.selector_to_css(selector).await?;
        debug!("Clicking: {}", css);

        let element = self.find_element_with_fallback(selector).await?;
        
        element
            .click()
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Click failed: {}", e)))?;

        Ok(())
    }

    /// Type text into an element
    pub async fn type_text(&self, selector: &SmartSelector, text: &str, clear_first: bool) -> FlowResult<()> {
        debug!("Typing into: {}", selector.value);

        let element = self.find_element_with_fallback(selector).await?;

        if clear_first {
            element
                .click()
                .await
                .map_err(|e| FlowEngineError::Replay(format!("Click for type failed: {}", e)))?;
            
            // Use evaluate for selectAll instead of execute
            self.page
                .evaluate("document.execCommand('selectAll', false, null)")
                .await
                .ok();
        }

        element
            .type_str(text)
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Type failed: {}", e)))?;

        Ok(())
    }

    /// Hover over an element
    pub async fn hover(&self, selector: &SmartSelector) -> FlowResult<()> {
        let element = self.find_element_with_fallback(selector).await?;
        
        element
            .scroll_into_view()
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Scroll failed: {}", e)))?;

        element
            .focus()
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Hover/focus failed: {}", e)))?;

        Ok(())
    }

    /// Get current page URL
    pub async fn get_url(&self) -> FlowResult<String> {
        let result = self.page
            .evaluate("window.location.href")
            .await
            .map_err(|e| FlowEngineError::Navigation(format!("Failed to get URL: {}", e)))?;

        result
            .into_value::<String>()
            .map_err(|e| FlowEngineError::Navigation(format!("Failed to parse URL: {}", e)))
    }

    /// Get page content (HTML)
    pub async fn get_page_content(&self) -> FlowResult<String> {
        let result = self.page
            .evaluate("document.documentElement.outerHTML")
            .await
            .map_err(|e| FlowEngineError::Navigation(format!("Failed to get content: {}", e)))?;

        result
            .into_value::<String>()
            .map_err(|e| FlowEngineError::Navigation(format!("Failed to parse content: {}", e)))
    }

    /// Execute JavaScript and return result
    pub async fn execute_script(&self, script: &str) -> FlowResult<serde_json::Value> {
        let result = self.page
            .evaluate(script)
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Script execution failed: {}", e)))?;

        // If the script returns undefined/void, return null instead of error
        Ok(result
            .into_value::<serde_json::Value>()
            .unwrap_or(serde_json::Value::Null))
    }

    /// Take a screenshot
    pub async fn screenshot(&self) -> FlowResult<Vec<u8>> {
        self.page
            .screenshot(chromiumoxide::page::ScreenshotParams::default())
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Screenshot failed: {}", e)))
    }

    /// Extract text from element
    pub async fn extract_text(&self, selector: &SmartSelector) -> FlowResult<String> {
        let element = self.find_element_with_fallback(selector).await?;
        
        element
            .inner_text()
            .await
            .map_err(|e| FlowEngineError::Replay(format!("Text extraction failed: {}", e)))?
            .ok_or_else(|| FlowEngineError::Replay("No text content".to_string()))
    }

    /// Find element with fallback to alternatives and retry logic
    async fn find_element_with_fallback(&self, selector: &SmartSelector) -> FlowResult<chromiumoxide::Element> {
        let css = self.selector_to_css(selector).await?;
        
        // Retry with exponential backoff - wait up to 15-20 seconds for element
        let max_attempts = 15;
        let mut delay = Duration::from_millis(500);
        
        for attempt in 1..=max_attempts {
            // Try primary selector
            if let Ok(element) = self.page.find_element(&css).await {
                if attempt > 1 {
                    debug!("Found element after {} attempts: {}", attempt, css);
                }
                return Ok(element);
            }

            // Try alternatives
            for alt in &selector.alternatives {
                let alt_css = match alt.selector_type {
                    SelectorType::Css => alt.value.clone(),
                    SelectorType::XPath => {
                        // Try to convert XPath alternative to CSS
                        let alt_selector = SmartSelector {
                            value: alt.value.clone(),
                            selector_type: SelectorType::XPath,
                            priority: alt.priority,
                            alternatives: Vec::new(),
                            validation_result: None,
                        };
                        match self.selector_to_css(&alt_selector).await {
                            Ok(css) => css,
                            Err(_) => continue,
                        }
                    },
                    SelectorType::Text => format!(":contains('{}')", alt.value),
                    SelectorType::AriaLabel => format!("[aria-label='{}']", alt.value),
                    SelectorType::Placeholder => format!("[placeholder='{}']", alt.value),
                };

                if let Ok(element) = self.page.find_element(&alt_css).await {
                    info!("Found element using alternative selector: {}", alt.value);
                    return Ok(element);
                }
            }

            // Wait before retry if not last attempt
            if attempt < max_attempts {
                debug!("Element not found (attempt {}/{}), waiting {:?}...", attempt, max_attempts, delay);
                tokio::time::sleep(delay).await;
                // Increase delay for next attempt (exponential backoff with cap)
                delay = std::cmp::min(delay * 2, Duration::from_secs(2));
            }
        }

        // Debug: Log what we see on the page when element not found
        error!("🔍 DEBUG: Element search failed for selector: {}", css);
        
        if let Ok(url) = self.page.evaluate("window.location.href").await {
            if let Ok(url_str) = url.into_value::<String>() {
                error!("   📍 Current page URL: {}", url_str);
            }
        }
        if let Ok(title) = self.page.evaluate("document.title").await {
            if let Ok(title_str) = title.into_value::<String>() {
                error!("   📄 Page title: {}", title_str);
            }
        }
        // Log available forms and inputs for debugging
        if let Ok(forms) = self.page.evaluate("document.querySelectorAll('form').length").await {
            if let Ok(count) = forms.into_value::<i64>() {
                error!("   📋 Page has {} form(s)", count);
            }
        }
        if let Ok(inputs) = self.page.evaluate("document.querySelectorAll('input').length").await {
            if let Ok(count) = inputs.into_value::<i64>() {
                error!("   ⌨️  Page has {} input(s)", count);
            }
        }
        // Log actual HTML structure of forms for deep debugging
        if let Ok(html) = self.page.evaluate("Array.from(document.querySelectorAll('form')).map(f => f.outerHTML.substring(0, 200)).join('\\n---\\n')").await {
            if let Ok(html_str) = html.into_value::<String>() {
                if !html_str.is_empty() {
                    error!("   🔧 Form HTML snippets:\n{}", html_str);
                }
            }
        }

        Err(FlowEngineError::ElementNotFound { selector: css })
    }

    /// Quick element check without long retry loop
    /// Used to quickly determine if an element exists (e.g., for Submit step pre-check)
    pub async fn find_element_quick(&self, selector: &SmartSelector) -> FlowResult<chromiumoxide::Element> {
        let css = self.selector_to_css(selector).await?;
        
        // Only 3 quick attempts with short delays (~1.5 seconds total)
        for attempt in 1..=3 {
            if let Ok(element) = self.page.find_element(&css).await {
                return Ok(element);
            }
            if attempt < 3 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        
        Err(FlowEngineError::ElementNotFound { selector: css })
    }

    /// Convert SmartSelector to CSS selector string
    pub async fn selector_to_css(&self, selector: &SmartSelector) -> FlowResult<String> {
        match selector.selector_type {
            SelectorType::Css => Ok(selector.value.clone()),
            SelectorType::XPath => {
                // Runtime conversion from XPath to CSS via JS
                debug!("Converting XPath to CSS at runtime: {}", selector.value);
                // Escape backslashes and double quotes for JS string
                let escaped_xpath = selector.value.replace("\\", "\\\\").replace("\"", "\\\"");
                
                let script = format!(
                    r#"(function() {{
                        try {{
                            const el = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                            if (!el) return null;
                            
                            if (el.id) return '#' + el.id;
                            let path = [];
                            let current = el;
                            while (current && current.nodeType === Node.ELEMENT_NODE) {{
                                let selector = current.nodeName.toLowerCase();
                                if (current.id) {{
                                    path.unshift('#' + current.id);
                                    break;
                                }}
                                let sib = current;
                                let nth = 1;
                                while (sib = sib.previousElementSibling) {{
                                    if (sib.nodeName.toLowerCase() == selector) nth++;
                                }}
                                if (nth != 1) selector += ':nth-of-type(' + nth + ')';
                                path.unshift(selector);
                                current = current.parentNode;
                            }}
                            return path.join(' > ') || null;
                        }} catch (e) {{
                            console.error('XPath to CSS conversion failed:', e);
                            return null;
                        }}
                    }})()"#,
                    escaped_xpath
                );
                
                let result = self.execute_script(&script).await?;
                if let Some(css) = result.as_str() {
                    Ok(css.to_string())
                } else {
                    Err(FlowEngineError::ElementNotFound { 
                        selector: format!("XPath: {}", selector.value) 
                    })
                }
            }
            SelectorType::Text => Ok(format!(":contains('{}')", selector.value)),
            SelectorType::AriaLabel => Ok(format!("[aria-label='{}']", selector.value)),
            SelectorType::Placeholder => Ok(format!("[placeholder='{}']", selector.value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::model::SmartSelector;

    #[test]
    fn test_selector_to_css() {
        let selector = SmartSelector::css("#login-btn");
        assert_eq!(selector.selector_type, SelectorType::Css);
        assert_eq!(selector.value, "#login-btn");
    }
}
