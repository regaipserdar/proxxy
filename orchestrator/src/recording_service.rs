use flow_engine::{
    BrowserManager, BrowserOptions, ProxyConfig,
    FlowStep, SmartSelector, SecretString,
};
use proxy_core::CertificateAuthority;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use std::io::Write;
use serde::Deserialize;

/// Recording session state
#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Starting,
    Recording { profile_id: String },
    Stopping,
    Failed { error: String },
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Active recording session info
#[derive(Debug, Clone)]
pub struct RecordingSessionInfo {
    pub profile_id: String,
    pub start_url: String,
    pub event_count: usize,
    pub started_at: i64,
}

/// Recording service for managing browser-based flow recording
pub struct RecordingService {
    ca: Arc<CertificateAuthority>,
    browser_manager: Arc<BrowserManager>,
    state: Arc<RwLock<RecordingState>>,
    session_info: Arc<RwLock<Option<RecordingSessionInfo>>>,
    ca_cert_path: Arc<RwLock<Option<String>>>,
}

impl RecordingService {
    pub fn new(ca: Arc<CertificateAuthority>) -> Self {
        Self {
            ca,
            browser_manager: Arc::new(BrowserManager::new()),
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            session_info: Arc::new(RwLock::new(None)),
            ca_cert_path: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current recording state
    pub async fn get_state(&self) -> RecordingState {
        self.state.read().await.clone()
    }

    /// Get current session info
    pub async fn get_session_info(&self) -> Option<RecordingSessionInfo> {
        self.session_info.read().await.clone()
    }

    /// Start a recording session
    pub async fn start_recording(
        &self,
        profile_id: String,
        start_url: String,
        proxy_port: Option<u16>,
    ) -> Result<(), String> {
        // Check if already recording
        {
            let state = self.state.read().await;
            if matches!(*state, RecordingState::Recording { .. }) {
                return Err("Already recording".to_string());
            }
        }

        // Set state to starting
        *self.state.write().await = RecordingState::Starting;

        // Write CA cert to temp file for browser
        let ca_cert_pem = self.ca.get_ca_cert_pem()
            .map_err(|e| format!("CA certificate not available: {:?}", e))?;
        
        let ca_cert_path = self.write_ca_cert_to_temp(&ca_cert_pem)?;
        *self.ca_cert_path.write().await = Some(ca_cert_path.clone());

        info!("🎥 Starting recording for profile: {}", profile_id);
        info!("   📜 CA cert written to: {}", ca_cert_path);

        // Configure browser options
        let mut options = BrowserOptions::headed()
            .headless(false);
        
        options.ignore_ssl_errors = true;
        options.ca_cert_path = Some(ca_cert_path);

        // Add proxy if port specified
        if let Some(port) = proxy_port {
            options.proxy = Some(ProxyConfig::new("127.0.0.1", port));
            info!("   🔗 Proxy configured: 127.0.0.1:{}", port);
        }

        // Launch browser
        match self.browser_manager.launch(options).await {
            Ok(browser_arc) => {
                info!("   ✅ Browser launched successfully");

                // Navigate to start URL using existing page (avoids extra empty tab)
                if let Some(browser_guard) = browser_arc.read().await.as_ref() {
                    // Get existing page or create one
                    let pages = browser_guard.browser().pages().await
                        .map_err(|e| format!("Failed to get pages: {:?}", e))?;
                    
                    let page = if let Some(existing_page) = pages.into_iter().next() {
                        // Use existing blank page
                        existing_page.goto(&start_url).await
                            .map_err(|e| format!("Failed to navigate: {:?}", e))?;
                        existing_page
                    } else {
                        // No existing page, create new one
                        browser_guard.browser().new_page(&start_url).await
                            .map_err(|e| format!("Failed to create page: {:?}", e))?
                    };
                    
                    info!("   🌐 Navigated to: {}", start_url);
                    
                    // Inject recording script ON EVERY NEW DOCUMENT (persists through navigation)
                    let recording_script = self.get_recording_script();
                    if let Err(e) = page.evaluate_on_new_document(recording_script).await {
                        warn!("   ⚠️ Failed to register recording script: {:?}", e);
                    } else {
                        info!("   💉 Recording script registered (will persist through navigation)");
                    }
                    
                    // Also inject immediately for current page
                    if let Err(e) = page.evaluate(recording_script).await {
                        warn!("   ⚠️ Failed to inject initial recording script: {:?}", e);
                    }
                }

                // Update session info
                let now = chrono::Utc::now().timestamp();
                *self.session_info.write().await = Some(RecordingSessionInfo {
                    profile_id: profile_id.clone(),
                    start_url: start_url.clone(),
                    event_count: 0,
                    started_at: now,
                });

                // Set state to recording
                *self.state.write().await = RecordingState::Recording { profile_id };

                Ok(())
            }
            Err(e) => {
                error!("   ❌ Failed to launch browser: {:?}", e);
                *self.state.write().await = RecordingState::Failed { 
                    error: format!("Browser launch failed: {:?}", e) 
                };
                Err(format!("Failed to launch browser: {:?}", e))
            }
        }
    }

    /// Stop recording and return captured events
    pub async fn stop_recording(&self, save: bool) -> Result<Option<(String, Vec<FlowStep>)>, String> {
        let profile_id = {
            let state = self.state.read().await;
            match &*state {
                RecordingState::Recording { profile_id } => profile_id.clone(),
                _ => return Err("Not currently recording".to_string()),
            }
        };

        *self.state.write().await = RecordingState::Stopping;
        info!("🛑 Stopping recording for profile: {}", profile_id);

        let mut recorded_steps = Vec::new();

        if save {
            // Harvest events before closing
            info!("   🔍 Attempting to harvest events...");
            if let Some(browser_arc) = self.browser_manager.get_browser().await {
                info!("   🔍 Got browser_arc");
                let guard = browser_arc.read().await;
                if let Some(managed_browser) = guard.as_ref() {
                    info!("   🔍 Got managed_browser");
                    match managed_browser.browser().pages().await {
                        Ok(pages) => {
                            info!("   🔍 Got {} pages", pages.len());
                            for (page_idx, page) in pages.iter().enumerate() {
                                info!("   🔍 Processing page {}", page_idx);
                                // Extract events as JSON string
                                match page.evaluate("JSON.stringify(window.__proxxy_events || [])").await {
                                    Ok(val) => {
                                        match val.into_value::<String>() {
                                            Ok(json_str) => {
                                                info!("   🔍 Got JSON: {} chars", json_str.len());
                                                match serde_json::from_str::<Vec<RawEvent>>(&json_str) {
                                                    Ok(events) => {
                                                        info!("   📥 Harvested {} raw events from page {}", events.len(), page_idx);
                                                        for (i, event) in events.into_iter().enumerate() {
                                                            info!("      📋 Event {}: type={}, xpath={:?}", i, event.event_type, event.xpath);
                                                            if let Some(step) = self.convert_event_to_step(event) {
                                                                info!("      ✅ Converted to step: {:?}", step);
                                                                recorded_steps.push(step);
                                                            } else {
                                                                info!("      ❌ Could not convert (missing xpath or unknown type)");
                                                            }
                                                        }
                                                    },
                                                    Err(e) => warn!("   ⚠️ Failed to parse events: {:?}", e),
                                                }
                                            },
                                            Err(e) => warn!("   ⚠️ Failed to get event string: {:?}", e),
                                        }
                                    },
                                    Err(e) => warn!("   ⚠️ Failed to evaluate event script on page {}: {:?}", page_idx, e),
                                }
                            }
                        },
                        Err(e) => warn!("   ⚠️ Failed to get pages: {:?}", e),
                    }
                } else {
                    warn!("   ⚠️ managed_browser is None!");
                }
            } else {
                warn!("   ⚠️ browser_arc is None!");
            }
        }

        // Close browser
        if let Err(e) = self.browser_manager.close().await {
            warn!("   ⚠️ Error closing browser: {:?}", e);
        } else {
            info!("   ✅ Browser closed");
        }

        // Cleanup temp CA cert file
        if let Some(path) = self.ca_cert_path.write().await.take() {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!("   ⚠️ Failed to cleanup CA cert file: {:?}", e);
            }
        }

        // Clear session info
        *self.session_info.write().await = None;
        *self.state.write().await = RecordingState::Idle;

        if save {
            info!("   💾 Recording saved for profile: {} ({} steps)", profile_id, recorded_steps.len());
            Ok(Some((profile_id, recorded_steps)))
        } else {
            info!("   🗑️ Recording discarded");
            Ok(None)
        }
    }

    fn convert_event_to_step(&self, event: RawEvent) -> Option<FlowStep> {
        let xpath = event.xpath.clone();
        let selector_val = event.selector.clone().or(xpath);
        
        let selector = match selector_val {
            Some(s) => {
                // Determine if it's XPath or CSS
                if s.starts_with('/') || s.starts_with('(') {
                    SmartSelector::xpath(s)
                } else {
                    SmartSelector::css(s)
                }
            },
            None => {
                warn!("⚠️ RawEvent missing selector and xpath: {:?}", event);
                return None;
            }
        };

        match event.event_type.as_str() {
            "click" => {
                info!("   👉 Converting 'click' event at {}", selector.value);
                Some(FlowStep::Click {
                    selector,
                    wait_for: None,
                })
            },
            "input" => {
                let is_masked = event.is_password.unwrap_or(false);
                let actual_value = event.value.unwrap_or_default();
                info!("   ⌨️  Converting 'input' event (value len: {}) at {}", actual_value.len(), selector.value);
                Some(FlowStep::Type {
                    selector,
                    value: SecretString::new(Box::from(actual_value)),
                    is_masked,
                    clear_first: false,
                })
            },
            "submit" => {
                info!("   📤 Converting 'submit' event at {}", selector.value);
                Some(FlowStep::Submit {
                    selector,
                    wait_for_navigation: true,
                })
            },
            _ => {
                warn!("   ❓ Unknown event type: {}", event.event_type);
                None
            },
        }
    }

    /// Debug: Launch browser with proxy for manual testing
    /// Does NOT start recording, just opens browser window
    pub async fn debug_launch_browser(
        &self,
        start_url: String,
        proxy_port: Option<u16>,
    ) -> Result<(), String> {
        info!("🔧 DEBUG: Launching browser for manual testing");
        
        // Write CA cert to temp file
        let ca_cert_pem = self.ca.get_ca_cert_pem()
            .map_err(|e| format!("CA certificate not available: {:?}", e))?;
        
        let ca_cert_path = self.write_ca_cert_to_temp(&ca_cert_pem)?;
        info!("   📜 CA cert: {}", ca_cert_path);

        // Configure browser options
        let mut options = BrowserOptions::headed()
            .headless(false);
        
        options.ignore_ssl_errors = true;
        options.ca_cert_path = Some(ca_cert_path);

        // Add proxy if port specified
        if let Some(port) = proxy_port {
            options.proxy = Some(ProxyConfig::new("127.0.0.1", port));
            info!("   🔗 Proxy: 127.0.0.1:{}", port);
        }

        // Launch browser
        match self.browser_manager.launch(options).await {
            Ok(browser_arc) => {
                info!("   ✅ Browser launched");

                // Navigate to start URL using existing page
                if let Some(browser_guard) = browser_arc.read().await.as_ref() {
                    let pages = browser_guard.browser().pages().await
                        .map_err(|e| format!("Failed to get pages: {:?}", e))?;
                    
                    if let Some(page) = pages.into_iter().next() {
                        page.goto(&start_url).await
                            .map_err(|e| format!("Failed to navigate: {:?}", e))?;
                        info!("   🌐 Navigated to: {}", start_url);
                    } else {
                        browser_guard.browser().new_page(&start_url).await
                            .map_err(|e| format!("Failed to create page: {:?}", e))?;
                        info!("   🌐 Navigated to: {}", start_url);
                    }
                }

                Ok(())
            }
            Err(e) => {
                Err(format!("Browser launch failed: {:?}", e))
            }
        }
    }

    /// Close debug browser
    pub async fn debug_close_browser(&self) -> Result<(), String> {
        info!("🔧 DEBUG: Closing browser");
        self.browser_manager.close().await
            .map_err(|e| format!("Failed to close browser: {:?}", e))?;
        
        // Cleanup temp CA cert
        if let Some(path) = self.ca_cert_path.write().await.take() {
            let _ = std::fs::remove_file(&path);
        }
        
        Ok(())
    }

    /// Open HTML/JSON content in the managed browser
    /// If browser is not open, launches one first
    /// If browser is open, opens a new tab with the content
    pub async fn open_in_browser(
        &self,
        content: String,
        content_type: String,
        base_url: Option<String>,
        proxy_port: Option<u16>,
    ) -> Result<(), String> {
        info!("🌐 Opening content in browser (type: {}, base: {:?})", content_type, base_url);
        
        // Check if browser is already open
        let browser_exists = self.browser_manager.get_browser().await.is_some();
        
        if !browser_exists {
            // Need to launch browser first
            info!("   📱 No browser open, launching...");
            
            // Write CA cert to temp file
            let ca_cert_pem = self.ca.get_ca_cert_pem()
                .map_err(|e| format!("CA certificate not available: {:?}", e))?;
            
            let ca_cert_path = self.write_ca_cert_to_temp(&ca_cert_pem)?;
            *self.ca_cert_path.write().await = Some(ca_cert_path.clone());

            // Configure browser options
            let mut options = BrowserOptions::headed()
                .headless(false);
            
            options.ignore_ssl_errors = true;
            options.ca_cert_path = Some(ca_cert_path);

            // Add proxy if port specified
            if let Some(port) = proxy_port {
                options.proxy = Some(ProxyConfig::new("127.0.0.1", port));
                info!("   🔗 Proxy: 127.0.0.1:{}", port);
            }

            // Launch browser
            self.browser_manager.launch(options).await
                .map_err(|e| format!("Browser launch failed: {:?}", e))?;
            info!("   ✅ Browser launched");
        }
        
        // Open new tab and inject content
        if let Some(browser_arc) = self.browser_manager.get_browser().await {
            if let Some(browser_guard) = browser_arc.read().await.as_ref() {
                // For HTML content with base URL, inject <base> tag and navigate
                if content_type.contains("html") && base_url.is_some() {
                    let base = base_url.unwrap();
                    
                    // Inject <base> tag into HTML head for proper relative URL resolution
                    let modified_content = if content.to_lowercase().contains("<head>") {
                        content.replacen("<head>", &format!("<head><base href=\"{}\">", base), 1)
                            .replacen("<HEAD>", &format!("<HEAD><base href=\"{}\">", base), 1)
                    } else if content.to_lowercase().contains("<html>") {
                        content.replacen("<html>", &format!("<html><head><base href=\"{}\"></head>", base), 1)
                            .replacen("<HTML>", &format!("<HTML><head><base href=\"{}\"></head>", base), 1)
                    } else {
                        format!("<head><base href=\"{}\"></head>{}", base, content)
                    };
                    
                    // Create new page and inject HTML using data URL with base tag
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    let base64_content = STANDARD.encode(modified_content.as_bytes());
                    let data_url = format!("data:text/html;base64,{}", base64_content);
                    
                    browser_guard.browser().new_page(&data_url).await
                        .map_err(|e| format!("Failed to open new tab: {:?}", e))?;
                } else if content_type.contains("html") {
                    // HTML without base URL - use data URL directly
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    let base64_content = STANDARD.encode(content.as_bytes());
                    let data_url = format!("data:text/html;base64,{}", base64_content);
                    browser_guard.browser().new_page(&data_url).await
                        .map_err(|e| format!("Failed to open new tab: {:?}", e))?;
                } else {
                    // For JSON/text, use data URL (no relative resources to worry about)
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    let mime_type = if content_type.contains("json") {
                        "application/json"
                    } else {
                        "text/plain"
                    };
                    let base64_content = STANDARD.encode(content.as_bytes());
                    let data_url = format!("data:{};base64,{}", mime_type, base64_content);
                    browser_guard.browser().new_page(&data_url).await
                        .map_err(|e| format!("Failed to open new tab: {:?}", e))?;
                }
                
                info!("   ✅ Opened content in new tab");
            }
        }
        
        Ok(())
    }

    /// Write CA certificate to a temporary file
    fn write_ca_cert_to_temp(&self, pem: &str) -> Result<String, String> {
        let temp_dir = std::env::temp_dir();
        let cert_path = temp_dir.join("proxxy_ca.crt");
        
        let mut file = std::fs::File::create(&cert_path)
            .map_err(|e| format!("Failed to create temp CA file: {:?}", e))?;
        
        file.write_all(pem.as_bytes())
            .map_err(|e| format!("Failed to write CA cert: {:?}", e))?;
        
        Ok(cert_path.to_string_lossy().to_string())
    }

    /// Get JavaScript recording script to inject
    fn get_recording_script(&self) -> &'static str {
        r#"
        (function() {
            window.__proxxy_events = [];
            
            // Click listener
            document.addEventListener('click', function(e) {
                const target = e.target;
                const event = {
                    type: 'click',
                    timestamp: Date.now(),
                    tagName: target.tagName,
                    id: target.id || null,
                    className: target.className || null,
                    name: target.name || null,
                    textContent: target.textContent?.substring(0, 100) || null,
                    xpath: getXPath(target),
                    selector: getSelector(target)
                };
                window.__proxxy_events.push(event);
                console.log('[Proxxy Recording] Click:', event);
            }, true);

            function getSelector(el) {
                if (!el) return null;
                if (el.id) return '#' + el.id;
                
                let path = [];
                let current = el;
                while (current && current.nodeType === Node.ELEMENT_NODE) {
                    let selector = current.nodeName.toLowerCase();
                    if (current.id) {
                        path.unshift('#' + current.id);
                        break;
                    }
                    
                    let sib = current;
                    let nth = 1;
                    while (sib = sib.previousElementSibling) {
                        if (sib.nodeName.toLowerCase() == selector) nth++;
                    }
                    if (nth != 1) selector += ':nth-of-type(' + nth + ')';
                    
                    path.unshift(selector);
                    current = current.parentNode;
                }
                return path.join(' > ');
            }

            // Debounced input capture - captures FINAL value instead of per-keystroke
            const inputTimers = new Map();
            const capturedInputs = new Map();
            
            function captureInputValue(target) {
                const xpath = getXPath(target);
                const inputId = xpath;
                const currentValue = target.value || '';
                
                // Store current value state
                capturedInputs.set(inputId, {
                    value: currentValue,
                    xpath: xpath,
                    selector: getSelector(target),
                    isPassword: target.type === 'password',
                    tagName: target.tagName,
                    id: target.id || null,
                    name: target.name || null,
                });
            }
            
            function flushInputEvent(target) {
                const xpath = getXPath(target);
                const inputId = xpath;
                const state = capturedInputs.get(inputId);
                
                if (state && state.value) {
                    const event = {
                        type: 'input',
                        timestamp: Date.now(),
                        tagName: state.tagName,
                        id: state.id,
                        name: state.name,
                        value: state.value, // Actual value captured!
                        isPassword: state.isPassword,
                        xpath: state.xpath,
                        selector: state.selector
                    };
                    window.__proxxy_events.push(event);
                    console.log('[Proxxy Recording] Input (debounced):', event.xpath, 'value:', state.isPassword ? '***' : state.value);
                    capturedInputs.delete(inputId);
                }
            }
            
            // On each input, capture value but debounce the event
            document.addEventListener('input', function(e) {
                const target = e.target;
                const xpath = getXPath(target);
                
                captureInputValue(target);
                
                // Clear existing timer
                if (inputTimers.has(xpath)) {
                    clearTimeout(inputTimers.get(xpath));
                }
                
                // Set new debounce timer (500ms)
                inputTimers.set(xpath, setTimeout(() => {
                    flushInputEvent(target);
                    inputTimers.delete(xpath);
                }, 500));
            }, true);
            
            // On blur, immediately flush the input
            document.addEventListener('blur', function(e) {
                const target = e.target;
                if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
                    const xpath = getXPath(target);
                    if (inputTimers.has(xpath)) {
                        clearTimeout(inputTimers.get(xpath));
                        inputTimers.delete(xpath);
                    }
                    flushInputEvent(target);
                }
            }, true);

            // Form submit listener
            document.addEventListener('submit', function(e) {
                const form = e.target;
                const event = {
                    type: 'submit',
                    timestamp: Date.now(),
                    formId: form.id || null,
                    action: form.action || null,
                    xpath: getXPath(form),
                    selector: getSelector(form)
                };
                window.__proxxy_events.push(event);
                console.log('[Proxxy Recording] Form submit:', event);
            }, true);

            // Navigation listener
            window.addEventListener('beforeunload', function() {
                console.log('[Proxxy Recording] Navigation detected');
            });

            function getXPath(element) {
                if (!element) return '';
                if (element.id) return '//*[@id="' + element.id + '"]';
                if (element === document.body) return '/html/body';
                
                let ix = 0;
                const siblings = element.parentNode?.childNodes || [];
                for (let i = 0; i < siblings.length; i++) {
                    const sibling = siblings[i];
                    if (sibling === element) {
                        const parentPath = getXPath(element.parentNode);
                        return parentPath + '/' + element.tagName.toLowerCase() + '[' + (ix + 1) + ']';
                    }
                    if (sibling.nodeType === 1 && sibling.tagName === element.tagName) {
                        ix++;
                    }
                }
                return '';
            }

            console.log('[Proxxy Recording] 🎥 Recording started');
        })();
        "#
    }
}

impl Default for RecordingService {
    fn default() -> Self {
        panic!("RecordingService requires CertificateAuthority")
    }
}

#[derive(Debug, Deserialize)]
struct RawEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    timestamp: Option<i64>, // Milliseconds since epoch from Date.now()
    #[serde(default)]
    xpath: Option<String>,
    #[serde(default)]
    selector: Option<String>,
    #[serde(rename = "isPassword", default)]
    is_password: Option<bool>,
    #[serde(default)]
    value: Option<String>, // Captured input value for Type events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_default() {
        let state = RecordingState::default();
        assert_eq!(state, RecordingState::Idle);
    }
}
