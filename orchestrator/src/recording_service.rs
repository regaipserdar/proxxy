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
                    
                    // Inject recording script
                    let recording_script = self.get_recording_script();
                    if let Err(e) = page.evaluate(recording_script).await {
                        warn!("   ⚠️ Failed to inject recording script: {:?}", e);
                    } else {
                        info!("   💉 Recording script injected");
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
            if let Some(browser_arc) = self.browser_manager.get_browser().await {
                let guard = browser_arc.read().await;
                if let Some(managed_browser) = guard.as_ref() {
                    match managed_browser.browser().pages().await {
                        Ok(pages) => {
                            for page in pages {
                                // Extract events as JSON string
                                match page.evaluate("JSON.stringify(window.__proxxy_events || [])").await {
                                    Ok(val) => {
                                        match val.into_value::<String>() {
                                            Ok(json_str) => {
                                                match serde_json::from_str::<Vec<RawEvent>>(&json_str) {
                                                    Ok(events) => {
                                                        info!("   📥 Harvested {} events from page", events.len());
                                                        for event in events {
                                                            if let Some(step) = self.convert_event_to_step(event) {
                                                                recorded_steps.push(step);
                                                            }
                                                        }
                                                    },
                                                    Err(e) => warn!("   ⚠️ Failed to parse events: {:?}", e),
                                                }
                                            },
                                            Err(e) => warn!("   ⚠️ Failed to get event string: {:?}", e),
                                        }
                                    },
                                    Err(e) => warn!("   ⚠️ Failed to evaluate event script: {:?}", e),
                                }
                            }
                        },
                        Err(e) => warn!("   ⚠️ Failed to get pages: {:?}", e),
                    }
                }
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
        let xpath = event.xpath?;
        let selector = SmartSelector::xpath(xpath);

        match event.event_type.as_str() {
            "click" => Some(FlowStep::Click {
                selector,
                wait_for: None,
            }),
            "input" => {
                let is_masked = event.is_password.unwrap_or(false);
                Some(FlowStep::Type {
                    selector,
                    value: SecretString::new(Box::from("".to_string())), // Do not record actual input for security
                    is_masked,
                    clear_first: false,
                })
            },
            "submit" => Some(FlowStep::Submit {
                selector, // Submit event usually bubbles from form, but target is form. XPath handles this.
                wait_for_navigation: true,
            }),
            _ => None,
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
                    xpath: getXPath(target)
                };
                window.__proxxy_events.push(event);
                console.log('[Proxxy Recording] Click:', event);
            }, true);

            // Input listener
            document.addEventListener('input', function(e) {
                const target = e.target;
                // Only capture that input happened, not the value
                const event = {
                    type: 'input',
                    timestamp: Date.now(),
                    tagName: target.tagName,
                    id: target.id || null,
                    name: target.name || null,
                    inputType: target.type || null,
                    isPassword: target.type === 'password'
                };
                window.__proxxy_events.push(event);
            }, true);

            // Form submit listener
            document.addEventListener('submit', function(e) {
                const form = e.target;
                const event = {
                    type: 'submit',
                    timestamp: Date.now(),
                    formId: form.id || null,
                    action: form.action || null
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
    xpath: Option<String>,
    #[serde(rename = "isPassword", default)]
    is_password: Option<bool>,
    // Other fields available in JS but not strictly needed for basic replay yet:
    // timestamp, tagName, id, className, name, inputType, formId, action
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
