//! Recording Engine Module
//!
//! Captures user interactions in the browser and generates flow steps.

use crate::error::{FlowEngineError, FlowResult};
use crate::flow::model::{FlowProfile, FlowStep, FlowType, SmartSelector, ProfileStatus};
use crate::flow::analyzer::{SelectorAnalyzer, ElementInfo};
use crate::flow::page::PageController;
use chromiumoxide::Page;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Recording session state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingState {
    /// Not recording
    Idle,
    /// Recording in progress
    Recording,
    /// Recording paused
    Paused,
    /// Recording completed
    Completed,
    /// Recording failed
    Failed,
}

/// Event captured during recording
#[derive(Debug, Clone)]
pub enum RecordedEvent {
    /// Page navigation
    Navigation { url: String },
    /// Element clicked
    Click { element: ElementInfo, x: f64, y: f64 },
    /// Text typed into element
    Input { element: ElementInfo, value: String, is_password: bool },
    /// Form submitted
    Submit { element: ElementInfo },
    /// Element selected (dropdown)
    Select { element: ElementInfo, value: String },
    /// Key pressed
    KeyPress { key: String, modifiers: Vec<String> },
    /// Custom event
    Custom { event_type: String, data: serde_json::Value },
}

/// Recording session configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Automatically detect password fields
    pub detect_passwords: bool,
    /// Mask sensitive input values
    pub mask_sensitive: bool,
    /// Record mouse movements (can be verbose)
    pub record_mouse_moves: bool,
    /// Record scroll events
    pub record_scroll: bool,
    /// Minimum time between events (ms)
    pub debounce_ms: u64,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            detect_passwords: true,
            mask_sensitive: true,
            record_mouse_moves: false,
            record_scroll: false,
            debounce_ms: 100,
        }
    }
}

/// Flow Recorder - captures browser interactions
pub struct FlowRecorder {
    config: RecordingConfig,
    analyzer: SelectorAnalyzer,
    state: Arc<RwLock<RecordingState>>,
    events: Arc<RwLock<Vec<RecordedEvent>>>,
    profile: Arc<RwLock<Option<FlowProfile>>>,
}

impl FlowRecorder {
    /// Create a new recorder with default config
    pub fn new() -> Self {
        Self::with_config(RecordingConfig::default())
    }

    /// Create a new recorder with custom config
    pub fn with_config(config: RecordingConfig) -> Self {
        Self {
            config,
            analyzer: SelectorAnalyzer::default(),
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            events: Arc::new(RwLock::new(Vec::new())),
            profile: Arc::new(RwLock::new(None)),
        }
    }

    /// Start a new recording session
    pub async fn start_recording(
        &self, 
        name: impl Into<String>,
        start_url: impl Into<String>,
        flow_type: FlowType,
    ) -> FlowResult<Uuid> {
        let mut state = self.state.write().await;
        if *state == RecordingState::Recording {
            return Err(FlowEngineError::Recording("Already recording".to_string()));
        }

        let name = name.into();
        let start_url = start_url.into();
        
        info!("Starting recording: {} at {}", name, start_url);

        // Create new profile
        let mut profile = FlowProfile::new(&name, &start_url);
        profile.flow_type = flow_type;
        profile.status = ProfileStatus::Recording;
        let profile_id = profile.id;

        // Add initial navigation step
        profile.add_step(FlowStep::Navigate {
            url: start_url,
            wait_for: None,
        });

        // Store profile
        let mut profile_guard = self.profile.write().await;
        *profile_guard = Some(profile);

        // Clear previous events
        let mut events = self.events.write().await;
        events.clear();

        *state = RecordingState::Recording;

        Ok(profile_id)
    }

    /// Pause recording
    pub async fn pause(&self) -> FlowResult<()> {
        let mut state = self.state.write().await;
        if *state != RecordingState::Recording {
            return Err(FlowEngineError::Recording("Not currently recording".to_string()));
        }
        *state = RecordingState::Paused;
        info!("Recording paused");
        Ok(())
    }

    /// Resume recording
    pub async fn resume(&self) -> FlowResult<()> {
        let mut state = self.state.write().await;
        if *state != RecordingState::Paused {
            return Err(FlowEngineError::Recording("Recording not paused".to_string()));
        }
        *state = RecordingState::Recording;
        info!("Recording resumed");
        Ok(())
    }

    /// Stop recording and finalize the profile
    pub async fn stop_recording(&self) -> FlowResult<FlowProfile> {
        let mut state = self.state.write().await;
        if *state != RecordingState::Recording && *state != RecordingState::Paused {
            return Err(FlowEngineError::Recording("No active recording".to_string()));
        }

        *state = RecordingState::Completed;

        let mut profile_guard = self.profile.write().await;
        let profile = profile_guard.take()
            .ok_or_else(|| FlowEngineError::Recording("No profile found".to_string()))?;

        info!("Recording stopped: {} steps recorded", profile.step_count());

        Ok(profile)
    }

    /// Record a click event
    pub async fn record_click(&self, element: ElementInfo, x: f64, y: f64) -> FlowResult<()> {
        if !self.is_recording().await {
            return Ok(());
        }

        debug!("Recording click at ({}, {}) on {:?}", x, y, element.tag_name);

        // Generate smart selector
        let selector = self.analyzer.analyze_element(&element)?;

        // Add click step to profile
        let mut profile_guard = self.profile.write().await;
        if let Some(ref mut profile) = *profile_guard {
            profile.add_step(FlowStep::Click {
                selector,
                wait_for: None,
            });
        }

        // Store raw event
        let mut events = self.events.write().await;
        events.push(RecordedEvent::Click { element, x, y });

        Ok(())
    }

    /// Record an input event
    pub async fn record_input(
        &self,
        element: ElementInfo,
        value: String,
    ) -> FlowResult<()> {
        if !self.is_recording().await {
            return Ok(());
        }

        let is_password = self.is_password_field(&element);
        debug!("Recording input on {:?} (password: {})", element.tag_name, is_password);

        // Generate smart selector
        let selector = self.analyzer.analyze_element(&element)?;

        // Mask value if sensitive
        let stored_value = if is_password && self.config.mask_sensitive {
            // In real implementation, we'd use SecretString and encryption
            "***MASKED***".to_string()
        } else {
            value.clone()
        };

        // Add type step to profile
        let mut profile_guard = self.profile.write().await;
        if let Some(ref mut profile) = *profile_guard {
            profile.add_step(FlowStep::Type {
                selector,
                value: secrecy::SecretString::new(stored_value.clone().into()),
                is_masked: is_password,
                clear_first: true,
            });
        }

        // Store raw event
        let mut events = self.events.write().await;
        events.push(RecordedEvent::Input { element, value: stored_value, is_password });

        Ok(())
    }

    /// Record a form submission
    pub async fn record_submit(&self, element: ElementInfo) -> FlowResult<()> {
        if !self.is_recording().await {
            return Ok(());
        }

        debug!("Recording form submit");

        let selector = self.analyzer.analyze_element(&element)?;

        let mut profile_guard = self.profile.write().await;
        if let Some(ref mut profile) = *profile_guard {
            profile.add_step(FlowStep::Submit {
                selector,
                wait_for_navigation: true,
            });
        }

        let mut events = self.events.write().await;
        events.push(RecordedEvent::Submit { element });

        Ok(())
    }

    /// Record a navigation event
    pub async fn record_navigation(&self, url: String) -> FlowResult<()> {
        if !self.is_recording().await {
            return Ok(());
        }

        debug!("Recording navigation to: {}", url);

        let mut profile_guard = self.profile.write().await;
        if let Some(ref mut profile) = *profile_guard {
            // Only add if different from last navigation
            if let Some(FlowStep::Navigate { url: last_url, .. }) = profile.steps.last() {
                if last_url == &url {
                    return Ok(()); // Skip duplicate
                }
            }
            
            profile.add_step(FlowStep::Navigate {
                url: url.clone(),
                wait_for: None,
            });
        }

        let mut events = self.events.write().await;
        events.push(RecordedEvent::Navigation { url });

        Ok(())
    }

    /// Add a wait step
    pub async fn add_wait(&self, duration_ms: u64) -> FlowResult<()> {
        let mut profile_guard = self.profile.write().await;
        if let Some(ref mut profile) = *profile_guard {
            profile.add_step(FlowStep::Wait {
                duration_ms,
                condition: None,
            });
        }
        Ok(())
    }

    /// Get current recording state
    pub async fn get_state(&self) -> RecordingState {
        self.state.read().await.clone()
    }

    /// Check if currently recording
    async fn is_recording(&self) -> bool {
        *self.state.read().await == RecordingState::Recording
    }

    /// Get recorded events count
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }

    /// Check if element is a password field
    fn is_password_field(&self, element: &ElementInfo) -> bool {
        if let Some(ref input_type) = element.input_type {
            return input_type.eq_ignore_ascii_case("password");
        }
        // Also check common naming patterns
        if let Some(ref name) = element.name {
            let lower = name.to_lowercase();
            return lower.contains("password") || lower.contains("passwd") || lower.contains("pwd");
        }
        if let Some(ref id) = element.id {
            let lower = id.to_lowercase();
            return lower.contains("password") || lower.contains("passwd") || lower.contains("pwd");
        }
        false
    }

    /// JavaScript to inject for event capture
    pub fn get_capture_script() -> &'static str {
        r#"
        (function() {
            if (window.__flowRecorderInjected) return;
            window.__flowRecorderInjected = true;

            const getElementInfo = (el) => {
                if (!el || !el.tagName) return null;
                return {
                    tagName: el.tagName,
                    id: el.id || null,
                    classList: Array.from(el.classList || []),
                    name: el.getAttribute('name'),
                    inputType: el.getAttribute('type'),
                    placeholder: el.getAttribute('placeholder'),
                    ariaLabel: el.getAttribute('aria-label'),
                    dataTestid: el.getAttribute('data-testid'),
                    dataCy: el.getAttribute('data-cy'),
                    textContent: el.textContent?.substring(0, 100),
                    href: el.getAttribute('href')
                };
            };

            // Click listener
            document.addEventListener('click', (e) => {
                const info = getElementInfo(e.target);
                if (info) {
                    window.__flowEvents = window.__flowEvents || [];
                    window.__flowEvents.push({
                        type: 'click',
                        element: info,
                        x: e.clientX,
                        y: e.clientY,
                        timestamp: Date.now()
                    });
                }
            }, true);

            // Input listener
            document.addEventListener('input', (e) => {
                if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
                    const info = getElementInfo(e.target);
                    if (info) {
                        window.__flowEvents = window.__flowEvents || [];
                        window.__flowEvents.push({
                            type: 'input',
                            element: info,
                            value: e.target.value,
                            timestamp: Date.now()
                        });
                    }
                }
            }, true);

            // Submit listener
            document.addEventListener('submit', (e) => {
                const info = getElementInfo(e.target);
                if (info) {
                    window.__flowEvents = window.__flowEvents || [];
                    window.__flowEvents.push({
                        type: 'submit',
                        element: info,
                        timestamp: Date.now()
                    });
                }
            }, true);

            console.log('[FlowRecorder] Event capture initialized');
        })();
        "#
    }

    /// Get and clear captured events from page
    pub fn get_drain_events_script() -> &'static str {
        r#"
        (function() {
            const events = window.__flowEvents || [];
            window.__flowEvents = [];
            return events;
        })();
        "#
    }
}

impl Default for FlowRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recorder_lifecycle() {
        let recorder = FlowRecorder::new();
        
        // Start recording
        let profile_id = recorder.start_recording(
            "Test Flow",
            "https://example.com",
            FlowType::Login
        ).await.unwrap();
        
        assert!(profile_id != Uuid::nil());
        assert_eq!(recorder.get_state().await, RecordingState::Recording);

        // Pause
        recorder.pause().await.unwrap();
        assert_eq!(recorder.get_state().await, RecordingState::Paused);

        // Resume
        recorder.resume().await.unwrap();
        assert_eq!(recorder.get_state().await, RecordingState::Recording);

        // Stop
        let profile = recorder.stop_recording().await.unwrap();
        assert_eq!(profile.name, "Test Flow");
        assert!(profile.step_count() >= 1); // At least initial navigation
    }

    #[tokio::test]
    async fn test_record_events() {
        let recorder = FlowRecorder::new();
        
        recorder.start_recording("Test", "https://example.com", FlowType::Login).await.unwrap();

        // Record a click
        let element = ElementInfo {
            tag_name: "BUTTON".to_string(),
            id: Some("submit-btn".to_string()),
            ..Default::default()
        };
        recorder.record_click(element, 100.0, 200.0).await.unwrap();

        assert_eq!(recorder.event_count().await, 1);

        let profile = recorder.stop_recording().await.unwrap();
        assert_eq!(profile.step_count(), 2); // navigation + click
    }

    #[test]
    fn test_password_detection() {
        let recorder = FlowRecorder::new();
        
        let password_field = ElementInfo {
            tag_name: "INPUT".to_string(),
            input_type: Some("password".to_string()),
            ..Default::default()
        };
        assert!(recorder.is_password_field(&password_field));

        let text_field = ElementInfo {
            tag_name: "INPUT".to_string(),
            input_type: Some("text".to_string()),
            ..Default::default()
        };
        assert!(!recorder.is_password_field(&text_field));

        let named_password = ElementInfo {
            tag_name: "INPUT".to_string(),
            name: Some("user_password".to_string()),
            ..Default::default()
        };
        assert!(recorder.is_password_field(&named_password));
    }
}
