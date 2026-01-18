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

/// Source of navigation detection
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationSource {
    Browser,  // Authoritative: PerformanceObserver, History API
    Proxy,    // Fallback: proxy traffic (deprecated, causes noise)
}

/// Navigation event detected during recording
#[derive(Debug, Clone)]
pub struct NavigationEvent {
    pub url: String,
    pub timestamp: i64,
    pub nav_type: Option<String>,  // 'navigate', 'reload', 'back_forward', 'pushState', 'popstate', 'hashchange'
    pub source: NavigationSource,  // Browser (authoritative) or Proxy (deprecated)
}

/// Recording service for managing browser-based flow recording
pub struct RecordingService {
    ca: Arc<CertificateAuthority>,
    browser_manager: Arc<BrowserManager>,
    state: Arc<RwLock<RecordingState>>,
    session_info: Arc<RwLock<Option<RecordingSessionInfo>>>,
    ca_cert_path: Arc<RwLock<Option<String>>>,
    /// URLs visited during recording (from proxy traffic)
    navigation_history: Arc<RwLock<Vec<NavigationEvent>>>,
}

impl RecordingService {
    pub fn new(ca: Arc<CertificateAuthority>) -> Self {
        Self {
            ca,
            browser_manager: Arc::new(BrowserManager::new()),
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            session_info: Arc::new(RwLock::new(None)),
            ca_cert_path: Arc::new(RwLock::new(None)),
            navigation_history: Arc::new(RwLock::new(Vec::new())),
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

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        matches!(*self.state.read().await, RecordingState::Recording { .. })
    }

    /// Add a navigation event detected from proxy traffic
    /// Called by server when it receives HTTP traffic during recording
    /// Uses same-site (eTLD+1) filtering, noise domain exclusion, and static asset filtering
    /// NOTE: This is a supplementary signal - browser-side detection is the primary source
    pub async fn add_navigation_event(&self, url: &str, content_type: Option<&str>, status_code: u16) {
        // Only track during active recording
        if !self.is_recording().await {
            return;
        }
        
        // Skip non-successful responses (4xx, 5xx) - but allow redirects (3xx)
        if status_code >= 400 {
            return;
        }
        
        // === NOISE DOMAIN FILTERING ===
        // Exclude browser internal and Google/analytics service domains
        let noise_domains = [
            "googleapis.com",
            "gstatic.com",
            "google.com",
            "googleusercontent.com",
            "chrome-devtools-frontend.appspot.com",
            "chromium.org",
            "gvt1.com", "gvt2.com", "gvt3.com",
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
            "facebook.com",
            "fbcdn.net",
            "analytics.",
            "segment.",
            "hotjar.",
            "newrelic.",
            "sentry.",
        ];
        
        let is_noise_domain = noise_domains.iter().any(|d| url.contains(d));
        if is_noise_domain {
            return;
        }
        
        // === SAME-SITE (eTLD+1) FILTERING ===
        // Allow subdomains: api.example.com, auth.example.com all count as same-site
        let session = self.session_info.read().await;
        if let Some(ref info) = *session {
            let start_site = Self::extract_etld_plus_one(&info.start_url);
            let url_site = Self::extract_etld_plus_one(url);
            
            if let (Some(start), Some(current)) = (start_site, url_site) {
                if start != current {
                    return; // Skip navigation to different site
                }
            }
        }
        drop(session);
        
        // === STATIC ASSET FILTERING ===
        // Only filter definitive static assets by extension
        let static_extensions = [
            ".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico",
            ".woff", ".woff2", ".ttf", ".eot", ".map",
            ".mp3", ".mp4", ".webm", ".webp", ".pdf", ".zip"
        ];
        
        let path = url.split('?').next().unwrap_or(url);
        let is_static_asset = static_extensions.iter().any(|ext| {
            path.to_lowercase().ends_with(ext)
        });
        
        if is_static_asset {
            return;
        }
        
        // === CONTENT-TYPE CHECK ===
        // If we have Content-Type, use it as primary signal
        let is_html_content = content_type
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false);
        
        // For redirects (3xx), we don't have content-type yet but should still track
        let is_redirect = (300..400).contains(&status_code);
        
        // Fallback: URLs without extensions are likely page navigations
        let has_no_extension = !path.rsplit('/').next().unwrap_or("").contains('.');
        
        // Track if: HTML content OR redirect OR no extension (likely a page)
        let should_track = is_html_content || is_redirect || has_no_extension;
        
        if should_track {
            let mut history = self.navigation_history.write().await;
            
            // Avoid duplicate consecutive navigations to same URL
            if history.last().map(|e| e.url.as_str()) != Some(url) {
                info!("   📍 Navigation detected (traffic): {}", url);
                history.push(NavigationEvent {
                    url: url.to_string(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    nav_type: None, // Unknown from proxy
                    source: NavigationSource::Proxy, // Deprecated source
                });
            }
        }
    }
    
    /// Extract eTLD+1 (effective top-level domain + 1) from URL
    /// This allows same-site matching: api.example.com and example.com are same-site
    fn extract_etld_plus_one(url: &str) -> Option<String> {
        let domain = Self::extract_domain(url)?;
        
        // Simple eTLD+1 extraction: take last two parts
        // This handles most cases (example.com, example.co.uk needs more complex logic)
        let parts: Vec<&str> = domain.split('.').collect();
        
        if parts.len() >= 2 {
            // Handle common compound TLDs
            let compound_tlds = ["co.uk", "com.au", "co.nz", "com.br", "co.jp"];
            let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
            
            if compound_tlds.contains(&last_two.as_str()) && parts.len() >= 3 {
                // Take 3 parts for compound TLDs: example.co.uk
                Some(format!("{}.{}", parts[parts.len() - 3], last_two))
            } else {
                // Take 2 parts: example.com
                Some(last_two)
            }
        } else {
            Some(domain)
        }
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        let without_protocol = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        
        without_protocol
            .split('/')
            .next()
            .and_then(|s| s.split(':').next())
            .map(|s| s.to_lowercase())
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
                // Check if browser is actually running - if not, state is stuck
                let browser_running = if let Some(browser_arc) = self.browser_manager.get_browser().await {
                    let browser_guard = browser_arc.read().await;
                    browser_guard.is_some()
                } else {
                    false
                };
                
                if browser_running {
                    return Err("Already recording".to_string());
                }
                
                // No browser running, state is stuck - reset it
                warn!("   ⚠️ Recording state stuck (no active browser), resetting...");
                drop(state);
                *self.state.write().await = RecordingState::Idle;
                *self.session_info.write().await = None;
                self.navigation_history.write().await.clear();
            }
        }

        // Set state to starting
        *self.state.write().await = RecordingState::Starting;
        
        // Clear navigation history from previous session
        self.navigation_history.write().await.clear();

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
                                // Extract events and navigations from sessionStorage (persisted across navigations)
                                let harvest_script = r#"
                                    try {
                                        const storageKey = '__proxxy_recording_events';
                                        const navStorageKey = '__proxxy_navigations';
                                        
                                        const fromStorage = sessionStorage.getItem(storageKey);
                                        const fromWindow = window.__proxxy_events || [];
                                        const events = fromStorage ? JSON.parse(fromStorage) : fromWindow;
                                        
                                        const navFromStorage = sessionStorage.getItem(navStorageKey);
                                        const navFromWindow = window.__proxxy_navigations || [];
                                        const navigations = navFromStorage ? JSON.parse(navFromStorage) : navFromWindow;
                                        
                                        // Clear storage after reading
                                        sessionStorage.removeItem(storageKey);
                                        sessionStorage.removeItem(navStorageKey);
                                        
                                        // Return both events and navigations
                                        JSON.stringify({ events: events, navigations: navigations });
                                    } catch (e) {
                                        JSON.stringify({ events: window.__proxxy_events || [], navigations: window.__proxxy_navigations || [] });
                                    }
                                "#;
                                match page.evaluate(harvest_script).await {
                                    Ok(val) => {
                                        match val.into_value::<String>() {
                                            Ok(json_str) => {
                                                info!("   🔍 Got JSON: {} chars", json_str.len());
                                                
                                                // Parse the new format: {events, navigations}
                                                #[derive(serde::Deserialize)]
                                                struct HarvestResult {
                                                    events: Vec<RawEvent>,
                                                    #[serde(default)]
                                                    navigations: Vec<BrowserNavigation>,
                                                }
                                                
                                                #[derive(serde::Deserialize, Debug)]
                                                struct BrowserNavigation {
                                                    url: String,
                                                    #[serde(default)]
                                                    timestamp: i64,
                                                    #[serde(rename = "type", default)]
                                                    nav_type: Option<String>,
                                                    #[serde(default)]
                                                    source: Option<String>, // 'browser'
                                                }
                                                
                                                match serde_json::from_str::<HarvestResult>(&json_str) {
                                                    Ok(result) => {
                                                        info!("   📥 Harvested {} events, {} navigations from page {}", 
                                                              result.events.len(), result.navigations.len(), page_idx);
                                                        
                                                        // Process events
                                                        for (i, event) in result.events.into_iter().enumerate() {
                                                            info!("      📋 Event {}: type={}, xpath={:?}", i, event.event_type, event.xpath);
                                                            if let Some(step) = self.convert_event_to_step(event) {
                                                                info!("      ✅ Converted to step: {:?}", step);
                                                                recorded_steps.push(step);
                                                            } else {
                                                                info!("      ❌ Could not convert (missing xpath or unknown type)");
                                                            }
                                                        }
                                                        
                                                        // Store browser navigations (will use these instead of proxy-based)
                                                        if !result.navigations.is_empty() {
                                                            info!("   📍 Browser-detected navigations: {:?}", 
                                                                  result.navigations.iter().map(|n| &n.url).collect::<Vec<_>>());
                                                            // Add to navigation history (browser-based, authoritative)
                                                            let mut history = self.navigation_history.write().await;
                                                            for nav in result.navigations {
                                                                // Use browser navigations as they're more accurate
                                                                if !history.iter().any(|h| h.url == nav.url) {
                                                                    history.push(NavigationEvent {
                                                                        url: nav.url,
                                                                        timestamp: nav.timestamp,
                                                                        nav_type: nav.nav_type,
                                                                        source: NavigationSource::Browser, // Authoritative
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    },
                                                    Err(e) => {
                                                        // Fallback: try parsing as just events array (old format)
                                                        match serde_json::from_str::<Vec<RawEvent>>(&json_str) {
                                                            Ok(events) => {
                                                                info!("   📥 Harvested {} raw events (legacy format) from page {}", events.len(), page_idx);
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
                                                            Err(e2) => warn!("   ⚠️ Failed to parse events (new: {:?}, legacy: {:?})", e, e2),
                                                        }
                                                    },
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

        // Get navigation history before clearing state
        let nav_history = self.navigation_history.read().await.clone();
        info!("   📍 Navigation history: {} page(s) visited", nav_history.len());

        // Clear session info
        *self.session_info.write().await = None;
        *self.state.write().await = RecordingState::Idle;

        if save {
            // Optimize steps: merge consecutive TYPE events, remove noise
            let optimized_steps = self.optimize_steps(recorded_steps);
            
            // Merge navigation history with DOM steps
            let session = self.session_info.read().await;
            let start_url = session.as_ref().map(|s| s.start_url.clone());
            drop(session);
            
            let final_steps = self.merge_navigation_with_steps(nav_history, optimized_steps, start_url);
            
            info!("   🧹 Final {} steps (with navigation)", final_steps.len());
            info!("   💾 Recording saved for profile: {} ({} steps)", profile_id, final_steps.len());
            Ok(Some((profile_id, final_steps)))
        } else {
            info!("   🗑️ Recording discarded");
            Ok(None)
        }
    }

    fn convert_event_to_step(&self, event: RawEvent) -> Option<FlowStep> {
        // Build smart selector using priority-based selection
        let selector = match self.build_smart_selector(&event) {
            Some(s) => s,
            None => {
                warn!("⚠️ Could not build selector for event: {:?}", event.event_type);
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

    /// Build smart selector using priority-based selection
    /// Priority: ID (100) > data-testid (95) > name (90) > placeholder (70) > 
    ///           aria-label (65) > action (60) > unique class (50) > path (10)
    fn build_smart_selector(&self, event: &RawEvent) -> Option<SmartSelector> {
        let mut candidates: Vec<(u8, String)> = vec![];
        let tag = event.tag_name.as_deref().unwrap_or("*");
        
        // Priority 100: ID (if not dynamic)
        if let Some(id) = &event.id {
            if !id.is_empty() && !Self::is_dynamic_id(id) {
                candidates.push((100, format!("#{}", id)));
            }
        }
        
        // Priority 95: data-testid
        if let Some(testid) = &event.data_testid {
            if !testid.is_empty() {
                candidates.push((95, format!("[data-testid=\"{}\"]", testid)));
            }
        }
        
        // Priority 90: name attribute
        if let Some(name) = &event.name {
            if !name.is_empty() {
                candidates.push((90, format!("{}[name=\"{}\"]", tag, name)));
            }
        }
        
        // Priority 70: type + placeholder combo (for inputs)
        if let (Some(input_type), Some(placeholder)) = (&event.input_type, &event.placeholder) {
            if !placeholder.is_empty() {
                candidates.push((70, format!("{}[type=\"{}\"][placeholder=\"{}\"]", tag, input_type, placeholder)));
            }
        } else if let Some(placeholder) = &event.placeholder {
            if !placeholder.is_empty() {
                candidates.push((70, format!("{}[placeholder=\"{}\"]", tag, placeholder)));
            }
        }
        
        // Priority 65: aria-label
        if let Some(aria_label) = &event.aria_label {
            if !aria_label.is_empty() {
                candidates.push((65, format!("{}[aria-label=\"{}\"]", tag, aria_label)));
            }
        }
        
        // Priority 60: action attribute (for forms)
        if let Some(action) = &event.action {
            if !action.is_empty() && tag == "form" {
                // Extract path from full URL
                let action_path = if action.starts_with("http") {
                    action.split('/').skip(3).collect::<Vec<_>>().join("/")
                } else {
                    action.clone()
                };
                if !action_path.is_empty() {
                    candidates.push((60, format!("form[action*=\"{}\"]", action_path)));
                }
            }
        }
        
        // Priority 50: role attribute
        if let Some(role) = &event.role {
            if !role.is_empty() {
                candidates.push((50, format!("{}[role=\"{}\"]", tag, role)));
            }
        }
        
        // Priority 10: CSS path (last resort)
        if let Some(css_path) = &event.css_path {
            if !css_path.is_empty() {
                candidates.push((10, css_path.clone()));
            }
        }
        
        // Fallback to xpath converted to css
        if candidates.is_empty() {
            if let Some(xpath) = &event.xpath {
                let css = Self::xpath_to_css(xpath);
                candidates.push((5, css));
            }
        }
        
        if candidates.is_empty() {
            return None;
        }
        
        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        
        let (priority, value) = candidates.remove(0);
        
        // Build alternatives from remaining candidates
        use flow_engine::flow::model::AlternativeSelector;
        let alternatives: Vec<AlternativeSelector> = candidates
            .into_iter()
            .take(3)  // Max 3 alternatives
            .map(|(p, v)| AlternativeSelector {
                value: v,
                selector_type: flow_engine::flow::model::SelectorType::Css,
                priority: p,
            })
            .collect();
        
        info!("   🎯 Smart selector: {} (priority: {}, {} alternatives)", value, priority, alternatives.len());
        
        Some(SmartSelector {
            value,
            selector_type: flow_engine::flow::model::SelectorType::Css,
            priority,
            alternatives,
            validation_result: None,
        })
    }
    
    /// Detect dynamic IDs that should be avoided (e.g., __next, ember-123, react-UID)
    fn is_dynamic_id(id: &str) -> bool {
        // Framework-generated IDs
        if id.starts_with("__") || id.starts_with("ember") || id.starts_with("react-") {
            return true;
        }
        // IDs that are mostly numbers or UUIDs
        let digit_count = id.chars().filter(|c| c.is_numeric()).count();
        if digit_count > id.len() / 2 {
            return true;
        }
        // Very short single-letter IDs might be dynamic
        if id.len() <= 2 && id.chars().all(|c| c.is_alphanumeric()) {
            return true;
        }
        false
    }

    /// Convert XPath to CSS selector (best effort)
    fn xpath_to_css(xpath: &str) -> String {
        // Handle ID shortcut: //*[@id="foo"] -> #foo
        if xpath.starts_with("//*[@id=\"") && xpath.ends_with("\"]") {
            let id = &xpath[9..xpath.len()-2];
            return format!("#{}", id);
        }
        if xpath.starts_with("//*[@id='") && xpath.ends_with("']") {
            let id = &xpath[9..xpath.len()-2];
            return format!("#{}", id);
        }
        
        // Convert path-based XPath to CSS
        let mut css_parts = Vec::new();
        for part in xpath.split('/') {
            if part.is_empty() || part == "*" {
                continue;
            }
            
            // Handle element[n] -> element:nth-of-type(n)
            if let Some(bracket_pos) = part.find('[') {
                let element = &part[..bracket_pos];
                let index_str = &part[bracket_pos+1..part.len()-1];
                if let Ok(index) = index_str.parse::<u32>() {
                    if index == 1 {
                        css_parts.push(element.to_string());
                    } else {
                        css_parts.push(format!("{}:nth-of-type({})", element, index));
                    }
                } else {
                    css_parts.push(element.to_string());
                }
            } else {
                css_parts.push(part.to_string());
            }
        }
        
        css_parts.join(" > ")
    }

    /// Optimize recorded steps to reduce noise
    /// - Merges consecutive TYPE events on same selector (keeps only last value)
    /// - Removes duplicate consecutive CLICK events on same selector
    /// - Removes redundant CLICK before TYPE on same selector
    fn optimize_steps(&self, steps: Vec<FlowStep>) -> Vec<FlowStep> {
        if steps.is_empty() {
            return steps;
        }
        
        let input_count = steps.len();
        
        // PHASE 1: Find the LAST TYPE value for each selector
        // This is the user's final intended value for each field
        use std::collections::HashMap;
        let mut last_type_value: HashMap<String, (SecretString, bool)> = HashMap::new();
        
        for step in steps.iter() {
            if let FlowStep::Type { selector, value, is_masked, .. } = step {
                // Store the last value for this selector
                last_type_value.insert(
                    selector.value.clone(), 
                    (value.clone(), *is_masked)
                );
            }
        }
        
        // PHASE 2: Process steps - only keep the LAST TYPE for each selector
        let mut seen_type_selectors: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut optimized: Vec<FlowStep> = Vec::new();
        let mut skip_next_identical_action = false;
        
        // Process in REVERSE to find last occurrences
        let mut reversed_steps: Vec<FlowStep> = steps.into_iter().rev().collect();
        
        for step in reversed_steps.drain(..) {
            match &step {
                // For TYPE: only keep the LAST one for each selector
                FlowStep::Type { selector, .. } => {
                    if seen_type_selectors.contains(&selector.value) {
                        // Already saw a later TYPE for this selector, skip this earlier one
                        continue;
                    }
                    // This is the last TYPE for this selector - use the final value
                    if let Some((final_value, is_masked)) = last_type_value.get(&selector.value) {
                        seen_type_selectors.insert(selector.value.clone());
                        optimized.push(FlowStep::Type {
                            selector: selector.clone(),
                            value: final_value.clone(),
                            is_masked: *is_masked,
                            clear_first: true, // Always clear when replaying to avoid appending
                        });
                    }
                },
                
                // For CLICK: skip if followed by TYPE on same selector
                FlowStep::Click { selector, .. } => {
                    // Check if we already have a TYPE for this selector (which implies the click)
                    if seen_type_selectors.contains(&selector.value) {
                        // Skip this click since the TYPE will handle it
                        continue;
                    }
                    // Check for duplicate consecutive clicks (in reverse, so check last optimized)
                    if let Some(FlowStep::Click { selector: last_sel, .. }) = optimized.last() {
                        if last_sel.value == selector.value {
                            continue; // Skip duplicate
                        }
                    }
                    optimized.push(step);
                },
                
                // For SUBMIT: skip duplicates
                FlowStep::Submit { selector, .. } => {
                    if let Some(FlowStep::Submit { selector: last_sel, .. }) = optimized.last() {
                        if last_sel.value == selector.value {
                            continue; // Skip duplicate
                        }
                    }
                    optimized.push(step);
                },
                
                // Keep all other steps as-is
                _ => {
                    optimized.push(step);
                }
            }
        }
        
        // Reverse back to correct order
        optimized.reverse();
        
        info!(
            "   📊 Step optimization: {} -> {} steps",
            input_count,
            optimized.len()
        );
        
        optimized
    }

    /// Merge navigation history from browser with DOM-based steps
    /// Only uses browser-detected navigations (authoritative), ignores proxy signals
    fn merge_navigation_with_steps(
        &self, 
        nav_history: Vec<NavigationEvent>, 
        mut steps: Vec<FlowStep>,
        start_url: Option<String>
    ) -> Vec<FlowStep> {
        // Filter to only browser-sourced navigations (authoritative)
        let browser_navs: Vec<&NavigationEvent> = nav_history
            .iter()
            .filter(|nav| nav.source == NavigationSource::Browser)
            .collect();
        
        // Skip if no browser navigations or only start_url
        if browser_navs.len() <= 1 {
            info!("   📍 No additional browser navigations to merge (found: {})", browser_navs.len());
            return steps;
        }
        
        // Build Navigate steps for each navigation after the initial page load
        let navigate_steps: Vec<FlowStep> = browser_navs
            .iter()
            .skip(1) // Skip first (initial page load, handled by replayer start_url)
            .filter(|nav| {
                // Skip if same as start_url or if it's just an initial_load duplicate
                let is_start = start_url.as_ref().map(|s| nav.url.starts_with(s)).unwrap_or(false);
                let is_initial = nav.nav_type.as_deref() == Some("initial_load");
                !is_start && !is_initial
            })
            .map(|nav| {
                info!("   🚀 Adding Navigate step for: {} (type: {:?})", nav.url, nav.nav_type);
                FlowStep::Navigate {
                    url: nav.url.clone(),
                    wait_for: None,
                }
            })
            .collect();
        
        if navigate_steps.is_empty() {
            info!("   📍 No Navigate steps to add after filtering");
            return steps;
        }
        
        // Prepend navigation steps to DOM steps
        // This ensures the replayer navigates to the right page before trying to interact with elements
        let nav_count = navigate_steps.len();
        let mut final_steps = navigate_steps;
        final_steps.append(&mut steps);
        
        info!("   📍 Added {} Navigate step(s) from browser", nav_count);
        
        final_steps
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
            // Load existing events from sessionStorage (survives navigation)
            const storageKey = '__proxxy_recording_events';
            let existingEvents = [];
            try {
                const stored = sessionStorage.getItem(storageKey);
                if (stored) {
                    existingEvents = JSON.parse(stored);
                }
            } catch (e) {
                console.log('[Proxxy Recording] Could not load existing events:', e);
            }
            
            window.__proxxy_events = existingEvents;
            console.log('[Proxxy Recording] 🎥 Recording started. Existing events:', existingEvents.length);
            
            // Helper: save events to sessionStorage
            function persistEvents() {
                try {
                    sessionStorage.setItem(storageKey, JSON.stringify(window.__proxxy_events));
                } catch (e) {
                    console.log('[Proxxy Recording] Could not persist events:', e);
                }
            }
            
            // === BROWSER-SIDE NAVIGATION DETECTION (GOLDEN SIGNAL) ===
            // This is the authoritative source for navigation events
            // Uses PerformanceObserver + performance.getEntriesByType('navigation')
            try {
                const navStorageKey = '__proxxy_navigations';
                let navigations = [];
                
                // Load existing navigations from sessionStorage
                try {
                    const stored = sessionStorage.getItem(navStorageKey);
                    if (stored) navigations = JSON.parse(stored);
                } catch (e) {}
                
                // Expose for harvesting
                window.__proxxy_navigations = navigations;
                
                function addNavigation(url, navType) {
                    // Avoid duplicates for same URL
                    if (navigations.some(n => n.url === url && n.type === navType)) return;
                    
                    const nav = {
                        url: url,
                        timestamp: Date.now(),
                        type: navType, // 'navigate', 'reload', 'back_forward', 'pushState', 'popstate'
                        source: 'browser' // Always browser-detected
                    };
                    navigations.push(nav);
                    window.__proxxy_navigations = navigations;
                    sessionStorage.setItem(navStorageKey, JSON.stringify(navigations));
                    console.log('[Proxxy Recording] 📍 Navigation:', navType, url);
                }
                
                // 1. PerformanceObserver for navigation timing (GOLDEN SIGNAL)
                // This catches full page loads, redirects, and document navigations
                if (window.PerformanceObserver) {
                    try {
                        const navObserver = new PerformanceObserver((list) => {
                            for (const entry of list.getEntries()) {
                                if (entry.entryType === 'navigation') {
                                    // entry.type: 'navigate', 'reload', 'back_forward', 'prerender'
                                    addNavigation(entry.name || window.location.href, entry.type || 'navigate');
                                }
                            }
                        });
                        navObserver.observe({ type: 'navigation', buffered: true });
                        console.log('[Proxxy Recording] ✅ PerformanceObserver (navigation) enabled');
                    } catch (e) {
                        console.log('[Proxxy Recording] PerformanceObserver navigation failed:', e);
                    }
                }
                
                // 2. Get current navigation entry (for initial page load)
                // This ensures we capture the page that was loaded before script injection
                if (performance.getEntriesByType) {
                    const navEntries = performance.getEntriesByType('navigation');
                    if (navEntries.length > 0) {
                        const entry = navEntries[0];
                        addNavigation(entry.name || window.location.href, entry.type || 'navigate');
                    } else {
                        // Fallback: record current page
                        addNavigation(window.location.href, 'initial_load');
                    }
                } else {
                    // Fallback for older browsers
                    addNavigation(window.location.href, 'initial_load');
                }
                
                // 3. History API hooks for SPA navigations
                let lastUrl = window.location.href;
                const originalPushState = history.pushState;
                const originalReplaceState = history.replaceState;
                
                history.pushState = function(...args) {
                    const result = originalPushState.apply(this, args);
                    if (window.location.href !== lastUrl) {
                        addNavigation(window.location.href, 'pushState');
                        lastUrl = window.location.href;
                    }
                    return result;
                };
                
                history.replaceState = function(...args) {
                    const result = originalReplaceState.apply(this, args);
                    if (window.location.href !== lastUrl) {
                        addNavigation(window.location.href, 'replaceState');
                        lastUrl = window.location.href;
                    }
                    return result;
                };
                
                window.addEventListener('popstate', () => {
                    if (window.location.href !== lastUrl) {
                        addNavigation(window.location.href, 'popstate');
                        lastUrl = window.location.href;
                    }
                });
                
                // 4. Hashchange for hash-based routing
                window.addEventListener('hashchange', () => {
                    addNavigation(window.location.href, 'hashchange');
                });
                
                console.log('[Proxxy Recording] ✅ Browser navigation detection fully enabled');
            } catch (e) {
                console.error('[Proxxy Recording] Navigation detection error:', e);
            }
            
            // Enhanced element info collector for smart selectors
            function getElementInfo(el) {
                if (!el) return {};
                return {
                    tagName: el.tagName?.toLowerCase() || null,
                    id: el.id || null,
                    name: el.name || null,
                    className: typeof el.className === 'string' ? el.className : null,
                    inputType: el.type || null,
                    placeholder: el.placeholder || null,
                    dataTestid: el.dataset?.testid || el.getAttribute('data-testid') || el.getAttribute('data-test-id') || null,
                    ariaLabel: el.getAttribute('aria-label') || null,
                    role: el.getAttribute('role') || null,
                    href: el.href || null,
                    action: el.action || null,
                    value: el.value || null,
                    textContent: el.textContent?.trim().substring(0, 50) || null,
                    // Path-based selectors as fallback
                    xpath: getXPath(el),
                    cssPath: getCssPath(el)
                };
            }
            
            // Click listener with enhanced attributes AND smart filtering
            document.addEventListener('click', function(e) {
                const target = e.target;
                const info = getElementInfo(target);
                
                // ALWAYS capture clicks that cause navigation
                const closestLink = target.closest('a');
                const hasHref = closestLink && closestLink.getAttribute('href');
                const isNavigation = hasHref || info.href;
                
                // ALWAYS capture these (never filter)
                if (isNavigation) {
                    console.log('[Proxxy Recording] Navigation click captured:', info.href || closestLink?.href);
                    const event = {
                        type: 'click',
                        timestamp: Date.now(),
                        ...info,
                        // Add href from ancestor if missing
                        href: info.href || (closestLink ? closestLink.href : null)
                    };
                    window.__proxxy_events.push(event);
                    persistEvents();
                    return;
                }
                
                // FILTER: Skip useless clicks on non-interactive elements
                const interactiveTags = ['INPUT', 'BUTTON', 'SELECT', 'TEXTAREA', 'LABEL'];
                const isInteractive = interactiveTags.includes(target.tagName) ||
                    target.getAttribute('role') === 'button' ||
                    target.getAttribute('role') === 'link' ||
                    target.getAttribute('role') === 'menuitem' ||
                    target.getAttribute('role') === 'tab' ||
                    target.getAttribute('onclick') ||
                    target.closest('button') ||
                    target.closest('[role="button"]');
                
                // Also allow clicks on elements with meaningful identifiers
                const hasMeaningfulId = info.id && !info.id.startsWith('__');
                const hasName = info.name;
                const hasTestId = info.dataTestid;
                const hasAriaLabel = info.ariaLabel;
                
                // Skip clicks on generic container divs
                if (!isInteractive && !hasMeaningfulId && !hasName && !hasTestId && !hasAriaLabel) {
                    console.log('[Proxxy Recording] Skip noise click on:', target.tagName);
                    return;
                }
                
                const event = {
                    type: 'click',
                    timestamp: Date.now(),
                    ...info
                };
                window.__proxxy_events.push(event);
                persistEvents();
                console.log('[Proxxy Recording] Click:', info.id || info.cssPath);
            }, true);

            // CSS path generator (with ID shortcut)
            function getCssPath(el) {
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
                const info = getElementInfo(target);
                
                // Store current value state with enhanced info
                capturedInputs.set(inputId, {
                    ...info,
                    isPassword: target.type === 'password'
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
                        ...state
                    };
                    window.__proxxy_events.push(event);
                    persistEvents();
                    console.log('[Proxxy Recording] Input (debounced):', state.id || state.cssPath, 'value:', state.isPassword ? '***' : state.value);
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

            // Form submit listener with enhanced attributes
            document.addEventListener('submit', function(e) {
                const form = e.target;
                const info = getElementInfo(form);
                const event = {
                    type: 'submit',
                    timestamp: Date.now(),
                    ...info
                };
                window.__proxxy_events.push(event);
                persistEvents();
                console.log('[Proxxy Recording] Form submit:', info.id || info.action || info.cssPath);
            }, true);

            // Save events before navigation (including clicks on links)
            window.addEventListener('beforeunload', function() {
                // Flush any pending inputs
                capturedInputs.forEach((state, inputId) => {
                    if (state && state.value) {
                        const event = {
                            type: 'input',
                            timestamp: Date.now(),
                            ...state
                        };
                        window.__proxxy_events.push(event);
                    }
                });
                persistEvents();
                console.log('[Proxxy Recording] Navigation - saved', window.__proxxy_events.length, 'events');
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
    timestamp: Option<i64>,
    
    // Enhanced element attributes for smart selectors
    #[serde(rename = "tagName", default)]
    tag_name: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "className", default)]
    class_name: Option<String>,
    #[serde(rename = "inputType", default)]  // JavaScript getElementInfo uses 'type' but we'll adjust JS to use 'inputType'
    input_type: Option<String>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(rename = "dataTestid", default)]
    data_testid: Option<String>,
    #[serde(rename = "ariaLabel", default)]
    aria_label: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    href: Option<String>,
    #[serde(default)]
    action: Option<String>,
    #[serde(rename = "textContent", default)]
    text_content: Option<String>,
    
    // Path-based selectors (fallback)
    #[serde(default)]
    xpath: Option<String>,
    #[serde(rename = "cssPath", default)]
    css_path: Option<String>,
    
    // For input events
    #[serde(rename = "isPassword", default)]
    is_password: Option<bool>,
    #[serde(default)]
    value: Option<String>,
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
