//! Session Integration Module
//! 
//! This module provides session integration functionality for Repeater and Intruder modules,
//! compatible with LSR (Login Sequence Recorder) session format and lifecycle management.

use attack_engine::{HttpRequestData, HttpResponseData, AttackError, AttackResult};
use proxy_common::session::{Session, SessionStatus, SessionEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Session manager for attack modules
pub struct SessionManager {
    /// Active sessions indexed by ID
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    
    /// Session event broadcaster for real-time updates
    event_sender: broadcast::Sender<SessionEvent>,
    
    /// Session validation cache to avoid repeated validation
    validation_cache: Arc<RwLock<HashMap<Uuid, ValidationCacheEntry>>>,
    
    /// Authentication failure detection configuration
    auth_failure_config: Arc<RwLock<AuthFailureDetectionConfig>>,
    
    /// Session failure tracking for reliability
    failure_tracking: Arc<RwLock<HashMap<Uuid, Vec<chrono::DateTime<chrono::Utc>>>>>,
}

/// Cached session validation result
#[derive(Debug, Clone)]
struct ValidationCacheEntry {
    is_valid: bool,
    validated_at: chrono::DateTime<chrono::Utc>,
    validation_url: Option<String>,
    error: Option<String>,
}

/// Session selection criteria for automatic session selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSelectionCriteria {
    /// Prefer sessions with specific profile IDs
    pub preferred_profile_ids: Vec<Uuid>,
    
    /// Require sessions to be validated within this duration
    pub max_validation_age_minutes: Option<u64>,
    
    /// Minimum usage count (prefer more established sessions)
    pub min_usage_count: Option<u64>,
    
    /// Exclude sessions that have failed recently
    pub exclude_recent_failures: bool,
}

/// Session application result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionApplicationResult {
    pub session_id: Uuid,
    pub session_name: String,
    pub headers_applied: usize,
    pub cookies_applied: usize,
    pub warnings: Vec<String>,
}

/// Session refresh request for expired sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRefreshRequest {
    pub session_id: Uuid,
    pub profile_id: Option<Uuid>,
    pub force_refresh: bool,
}

/// Session refresh result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRefreshResult {
    pub success: bool,
    pub new_session_id: Option<Uuid>,
    pub error: Option<String>,
    pub refresh_method: RefreshMethod,
}

/// Method used for session refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefreshMethod {
    /// Refreshed via LSR profile re-execution
    LSRProfileReExecution,
    /// Manual refresh by user
    ManualRefresh,
    /// Automatic refresh based on indicators
    AutomaticRefresh,
}

/// Authentication failure detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthFailureDetectionConfig {
    /// HTTP status codes that indicate authentication failure
    pub failure_status_codes: Vec<i32>,
    
    /// Response body patterns that indicate authentication failure
    pub failure_body_patterns: Vec<String>,
    
    /// Response header patterns that indicate authentication failure
    pub failure_header_patterns: HashMap<String, String>,
    
    /// URLs that indicate redirect to login page
    pub login_redirect_patterns: Vec<String>,
}

impl Default for AuthFailureDetectionConfig {
    fn default() -> Self {
        Self {
            failure_status_codes: vec![401, 403],
            failure_body_patterns: vec![
                "login".to_string(),
                "unauthorized".to_string(),
                "authentication required".to_string(),
                "access denied".to_string(),
                "session expired".to_string(),
            ],
            failure_header_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert("WWW-Authenticate".to_string(), ".*".to_string());
                patterns.insert("Location".to_string(), ".*/login.*".to_string());
                patterns
            },
            login_redirect_patterns: vec![
                ".*/login.*".to_string(),
                ".*/signin.*".to_string(),
                ".*/auth.*".to_string(),
            ],
        }
    }
}

/// Session expiration handling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpirationHandling {
    /// Fail the request immediately
    Fail,
    /// Continue without session data
    ContinueWithoutSession,
    /// Attempt to refresh the session
    AttemptRefresh { profile_id: Option<Uuid> },
    /// Use fallback session if available
    UseFallback { fallback_session_id: Uuid },
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
            auth_failure_config: Arc::new(RwLock::new(AuthFailureDetectionConfig::default())),
            failure_tracking: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a session (compatible with LSR session format)
    pub async fn add_session(&self, session: Session) -> AttackResult<()> {
        info!("🔐 Adding session: {} ({})", session.name, session.id);
        
        // Validate session structure
        self.validate_session_structure(&session)?;
        
        // Store session
        self.sessions.write().await.insert(session.id, session.clone());
        
        // Clear validation cache for this session
        self.validation_cache.write().await.remove(&session.id);
        
        // Broadcast session creation event
        let _ = self.event_sender.send(SessionEvent::Created { 
            session_id: session.id 
        });
        
        info!("   ✓ Session added successfully");
        Ok(())
    }

    /// Get all available sessions
    pub async fn get_sessions(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Get active sessions (non-expired, valid status)
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|session| {
                !session.is_expired() && 
                matches!(session.status, SessionStatus::Active)
            })
            .cloned()
            .collect()
    }

    /// Get a specific session by ID
    pub async fn get_session(&self, session_id: &Uuid) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &Uuid) -> AttackResult<()> {
        info!("🗑️ Removing session: {}", session_id);
        
        if self.sessions.write().await.remove(session_id).is_some() {
            // Clear validation cache
            self.validation_cache.write().await.remove(session_id);
            
            info!("   ✓ Session removed successfully");
            Ok(())
        } else {
            warn!("   ⚠ Session not found: {}", session_id);
            Err(AttackError::SessionExpired { 
                session_id: *session_id 
            })
        }
    }

    /// Apply session data to an HTTP request (LSR compatible)
    pub async fn apply_session_to_request(
        &self,
        mut request: HttpRequestData,
        session_id: &Uuid,
        expiration_handling: ExpirationHandling,
    ) -> AttackResult<(HttpRequestData, SessionApplicationResult)> {
        debug!("🔐 Applying session {} to request", session_id);
        
        // Get session
        let session = match self.get_session(session_id).await {
            Some(session) => session,
            None => {
                warn!("   ⚠ Session {} not found", session_id);
                return Err(AttackError::SessionExpired { 
                    session_id: *session_id 
                });
            }
        };

        // Check session expiration
        if session.is_expired() {
            warn!("   ⚠ Session {} has expired", session_id);
            return self.handle_expired_session(request, &session, expiration_handling).await;
        }

        // Check session status
        match session.status {
            SessionStatus::Active => {
                // Proceed with session application
            }
            SessionStatus::Expired => {
                warn!("   ⚠ Session {} is marked as expired", session_id);
                return self.handle_expired_session(request, &session, expiration_handling).await;
            }
            SessionStatus::Invalid => {
                warn!("   ⚠ Session {} is marked as invalid", session_id);
                return Err(AttackError::SessionExpired { 
                    session_id: *session_id 
                });
            }
            SessionStatus::Validating => {
                warn!("   ⚠ Session {} is still being validated", session_id);
                // Continue with application but add warning
            }
        }

        // Apply session headers and cookies
        let session_headers = session.get_http_headers();
        let header_count = session_headers.len();
        let cookie_count = session.cookies.len();
        
        // Initialize request headers if not present
        if request.headers.is_none() {
            request.headers = Some(attack_engine::HttpHeaders {
                headers: HashMap::new(),
            });
        }
        
        // Apply session headers (LSR format compatible)
        let mut warnings = Vec::new();
        if let Some(ref mut headers) = request.headers {
            for (key, value) in session_headers {
                // Check for header conflicts
                if let Some(existing_value) = headers.headers.get(&key) {
                    if existing_value != &value {
                        warnings.push(format!(
                            "Header '{}' overridden: '{}' -> '{}'", 
                            key, existing_value, value
                        ));
                    }
                }
                headers.headers.insert(key, value);
            }
        }
        
        // Increment session usage counter
        self.increment_session_usage(session_id).await;
        
        // Broadcast session usage event
        let _ = self.event_sender.send(SessionEvent::Used { 
            session_id: *session_id,
            target_url: request.url.clone(),
        });
        
        let result = SessionApplicationResult {
            session_id: *session_id,
            session_name: session.name.clone(),
            headers_applied: header_count,
            cookies_applied: cookie_count,
            warnings,
        };
        
        info!("   ✓ Applied session '{}': {} headers, {} cookies", 
              session.name, header_count, cookie_count);
        
        Ok((request, result))
    }

    /// Select best session based on criteria
    pub async fn select_session(&self, criteria: &SessionSelectionCriteria) -> Option<Session> {
        let sessions = self.get_active_sessions().await;
        
        if sessions.is_empty() {
            return None;
        }
        
        // Filter sessions based on criteria
        let mut candidates: Vec<Session> = sessions.into_iter()
            .filter(|session| self.matches_criteria(session, criteria))
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        // Sort by preference (most recently validated, highest usage count)
        candidates.sort_by(|a, b| {
            // Prefer sessions with recent validation
            let a_validation_age = a.metadata.last_validated
                .map(|t| chrono::Utc::now().signed_duration_since(t).num_minutes())
                .unwrap_or(i64::MAX);
            let b_validation_age = b.metadata.last_validated
                .map(|t| chrono::Utc::now().signed_duration_since(t).num_minutes())
                .unwrap_or(i64::MAX);
            
            match a_validation_age.cmp(&b_validation_age) {
                std::cmp::Ordering::Equal => {
                    // If validation age is equal, prefer higher usage count
                    b.metadata.usage_count.cmp(&a.metadata.usage_count)
                }
                other => other,
            }
        });
        
        candidates.into_iter().next()
    }

    /// Validate session against target URL (LSR compatible validation)
    pub async fn validate_session(
        &self,
        session_id: &Uuid,
        validation_url: &str,
    ) -> AttackResult<bool> {
        info!("🔍 Validating session {} against {}", session_id, validation_url);
        
        // Check validation cache first
        if let Some(cached) = self.get_cached_validation(session_id).await {
            if cached.validated_at > chrono::Utc::now() - chrono::Duration::minutes(5) {
                info!("   ✓ Using cached validation result: {}", cached.is_valid);
                return Ok(cached.is_valid);
            }
        }
        
        // Get session
        let mut session = match self.get_session(session_id).await {
            Some(session) => session,
            None => {
                return Err(AttackError::SessionExpired { 
                    session_id: *session_id 
                });
            }
        };
        
        // TODO: Implement actual validation by making a test request
        // For now, assume validation based on session status and expiration
        let is_valid = !session.is_expired() && 
                      matches!(session.status, SessionStatus::Active);
        
        // Update validation cache
        let cache_entry = ValidationCacheEntry {
            is_valid,
            validated_at: chrono::Utc::now(),
            validation_url: Some(validation_url.to_string()),
            error: if is_valid { None } else { Some("Session expired or invalid".to_string()) },
        };
        self.validation_cache.write().await.insert(*session_id, cache_entry);
        
        // Update session metadata
        if is_valid {
            session.mark_validated(validation_url.to_string());
            self.sessions.write().await.insert(*session_id, session);
            
            // Broadcast validation success
            let _ = self.event_sender.send(SessionEvent::Validated { 
                session_id: *session_id,
                validation_url: validation_url.to_string(),
            });
        } else {
            // Broadcast validation failure
            let _ = self.event_sender.send(SessionEvent::ValidationFailed { 
                session_id: *session_id,
                error: "Session expired or invalid".to_string(),
            });
        }
        
        info!("   ✓ Session validation result: {}", is_valid);
        Ok(is_valid)
    }

    /// Detect authentication failure from HTTP response (LSR compatible indicators)
    pub async fn detect_authentication_failure(
        &self,
        response: &HttpResponseData,
        request_url: &str,
    ) -> bool {
        let config = self.auth_failure_config.read().await;
        
        // Check status codes
        if config.failure_status_codes.contains(&response.status_code) {
            debug!("   🔍 Auth failure detected: status code {}", response.status_code);
            return true;
        }
        
        // Check response body patterns
        if let Ok(body_str) = String::from_utf8(response.body.clone()) {
            let body_lower = body_str.to_lowercase();
            for pattern in &config.failure_body_patterns {
                if body_lower.contains(&pattern.to_lowercase()) {
                    debug!("   🔍 Auth failure detected: body contains '{}'", pattern);
                    return true;
                }
            }
        }
        
        // Check response headers
        if let Some(headers) = &response.headers {
            for (header_name, pattern) in &config.failure_header_patterns {
                if let Some(header_value) = headers.headers.get(header_name) {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if regex.is_match(header_value) {
                            debug!("   🔍 Auth failure detected: header '{}' matches pattern '{}'", 
                                   header_name, pattern);
                            return true;
                        }
                    }
                }
            }
            
            // Check for login redirects
            if let Some(location) = headers.headers.get("Location") {
                for pattern in &config.login_redirect_patterns {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if regex.is_match(location) {
                            debug!("   🔍 Auth failure detected: redirect to login page '{}'", location);
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }

    /// Handle authentication failure for a session
    pub async fn handle_authentication_failure(
        &self,
        session_id: &Uuid,
        failure_url: &str,
        response: &HttpResponseData,
    ) -> AttackResult<SessionRefreshResult> {
        warn!("🚨 Authentication failure detected for session {}", session_id);
        
        // Record failure
        self.record_session_failure(session_id).await;
        
        // Mark session as invalid
        if let Some(mut session) = self.get_session(session_id).await {
            session.status = SessionStatus::Invalid;
            self.sessions.write().await.insert(*session_id, session.clone());
            
            // Broadcast failure event
            let _ = self.event_sender.send(SessionEvent::ValidationFailed {
                session_id: *session_id,
                error: format!("Authentication failure at {}", failure_url),
            });
            
            // Attempt refresh if profile is available
            if let Some(profile_id) = session.profile_id {
                info!("   🔄 Attempting session refresh via LSR profile {}", profile_id);
                return self.refresh_session_via_lsr(session_id, &profile_id).await;
            }
        }
        
        // No refresh possible
        Ok(SessionRefreshResult {
            success: false,
            new_session_id: None,
            error: Some("Session authentication failed and no refresh method available".to_string()),
            refresh_method: RefreshMethod::ManualRefresh,
        })
    }

    /// Refresh session via LSR profile re-execution
    pub async fn refresh_session_via_lsr(
        &self,
        session_id: &Uuid,
        profile_id: &Uuid,
    ) -> AttackResult<SessionRefreshResult> {
        info!("🔄 Refreshing session {} via LSR profile {}", session_id, profile_id);
        
        // TODO: Implement actual LSR integration
        // This would involve:
        // 1. Calling LSR to re-execute the login profile
        // 2. Getting new session data from LSR
        // 3. Updating the session with new data
        // 4. Validating the new session
        
        // For now, return a placeholder result
        warn!("   ⚠ LSR integration not yet implemented");
        
        Ok(SessionRefreshResult {
            success: false,
            new_session_id: None,
            error: Some("LSR integration not yet implemented".to_string()),
            refresh_method: RefreshMethod::LSRProfileReExecution,
        })
    }

    /// Refresh session manually (user-initiated)
    pub async fn refresh_session_manually(
        &self,
        session_id: &Uuid,
        new_session_data: Session,
    ) -> AttackResult<SessionRefreshResult> {
        info!("🔄 Manual session refresh for {}", session_id);
        
        // Validate new session data
        self.validate_session_structure(&new_session_data)?;
        
        // Replace old session with new data
        self.sessions.write().await.insert(*session_id, new_session_data.clone());
        
        // Clear validation cache
        self.validation_cache.write().await.remove(session_id);
        
        // Clear failure tracking
        self.failure_tracking.write().await.remove(session_id);
        
        // Broadcast refresh event
        let _ = self.event_sender.send(SessionEvent::Created {
            session_id: *session_id,
        });
        
        info!("   ✓ Session manually refreshed");
        
        Ok(SessionRefreshResult {
            success: true,
            new_session_id: Some(*session_id),
            error: None,
            refresh_method: RefreshMethod::ManualRefresh,
        })
    }

    /// Check if session should be automatically refreshed
    pub async fn should_auto_refresh_session(&self, session_id: &Uuid) -> bool {
        // Check failure count
        if let Some(failures) = self.failure_tracking.read().await.get(session_id) {
            let recent_failures = failures.iter()
                .filter(|&failure_time| {
                    chrono::Utc::now().signed_duration_since(*failure_time).num_minutes() < 30
                })
                .count();
            
            // Don't auto-refresh if there have been multiple recent failures
            if recent_failures >= 3 {
                debug!("   ⚠ Too many recent failures for session {}, skipping auto-refresh", session_id);
                return false;
            }
        }
        
        // Check if session has profile for refresh
        if let Some(session) = self.get_session(session_id).await {
            return session.profile_id.is_some();
        }
        
        false
    }

    /// Record session failure for tracking
    async fn record_session_failure(&self, session_id: &Uuid) {
        let mut tracking = self.failure_tracking.write().await;
        let failures = tracking.entry(*session_id).or_insert_with(Vec::new);
        failures.push(chrono::Utc::now());
        
        // Keep only recent failures (last 24 hours)
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        failures.retain(|&failure_time| failure_time > cutoff);
    }

    /// Get session failure history
    pub async fn get_session_failure_history(&self, session_id: &Uuid) -> Vec<chrono::DateTime<chrono::Utc>> {
        self.failure_tracking.read().await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Update authentication failure detection configuration
    pub async fn update_auth_failure_config(&self, config: AuthFailureDetectionConfig) {
        *self.auth_failure_config.write().await = config;
        info!("🔧 Updated authentication failure detection configuration");
    }

    /// Get current authentication failure detection configuration
    pub async fn get_auth_failure_config(&self) -> AuthFailureDetectionConfig {
        self.auth_failure_config.read().await.clone()
    }

    /// Handle expired session based on expiration handling strategy
    async fn handle_expired_session(
        &self,
        request: HttpRequestData,
        session: &Session,
        handling: ExpirationHandling,
    ) -> AttackResult<(HttpRequestData, SessionApplicationResult)> {
        match handling {
            ExpirationHandling::Fail => {
                error!("   ✗ Session expired, failing request");
                Err(AttackError::SessionExpired { 
                    session_id: session.id 
                })
            }
            ExpirationHandling::ContinueWithoutSession => {
                warn!("   ⚠ Session expired, continuing without session data");
                let result = SessionApplicationResult {
                    session_id: session.id,
                    session_name: session.name.clone(),
                    headers_applied: 0,
                    cookies_applied: 0,
                    warnings: vec!["Session expired, request sent without authentication".to_string()],
                };
                Ok((request, result))
            }
            ExpirationHandling::AttemptRefresh { profile_id: _ } => {
                warn!("   ⚠ Session expired, attempting refresh");
                // TODO: Implement session refresh via LSR integration
                // For now, fall back to continuing without session
                let result = SessionApplicationResult {
                    session_id: session.id,
                    session_name: session.name.clone(),
                    headers_applied: 0,
                    cookies_applied: 0,
                    warnings: vec![
                        "Session expired".to_string(),
                        "Session refresh not yet implemented".to_string(),
                        "Request sent without authentication".to_string(),
                    ],
                };
                Ok((request, result))
            }
            ExpirationHandling::UseFallback { fallback_session_id } => {
                warn!("   ⚠ Session expired, using fallback session");
                // Use Box::pin to avoid recursion issues
                Box::pin(self.apply_session_to_request(
                    request, 
                    &fallback_session_id, 
                    ExpirationHandling::Fail
                )).await
            }
        }
    }

    /// Increment session usage counter
    async fn increment_session_usage(&self, session_id: &Uuid) {
        if let Some(mut session) = self.sessions.read().await.get(session_id).cloned() {
            session.increment_usage();
            self.sessions.write().await.insert(*session_id, session);
        }
    }

    /// Check if session matches selection criteria
    fn matches_criteria(&self, session: &Session, criteria: &SessionSelectionCriteria) -> bool {
        // Check preferred profile IDs
        if !criteria.preferred_profile_ids.is_empty() {
            if let Some(profile_id) = session.profile_id {
                if !criteria.preferred_profile_ids.contains(&profile_id) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check validation age
        if let Some(max_age_minutes) = criteria.max_validation_age_minutes {
            if let Some(last_validated) = session.metadata.last_validated {
                let age_minutes = chrono::Utc::now()
                    .signed_duration_since(last_validated)
                    .num_minutes();
                if age_minutes > max_age_minutes as i64 {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check minimum usage count
        if let Some(min_usage) = criteria.min_usage_count {
            if session.metadata.usage_count < min_usage {
                return false;
            }
        }
        
        // TODO: Implement recent failure exclusion
        // This would require tracking session failures
        
        true
    }

    /// Get cached validation result
    async fn get_cached_validation(&self, session_id: &Uuid) -> Option<ValidationCacheEntry> {
        self.validation_cache.read().await.get(session_id).cloned()
    }

    /// Validate session structure (LSR compatibility check)
    fn validate_session_structure(&self, session: &Session) -> AttackResult<()> {
        // Check required fields
        if session.name.trim().is_empty() {
            return Err(AttackError::InvalidPayloadConfig {
                reason: "Session name cannot be empty".to_string(),
            });
        }
        
        // Validate headers format
        for (key, value) in &session.headers {
            if key.trim().is_empty() {
                return Err(AttackError::InvalidPayloadConfig {
                    reason: "Session header key cannot be empty".to_string(),
                });
            }
            if value.trim().is_empty() {
                return Err(AttackError::InvalidPayloadConfig {
                    reason: format!("Session header '{}' value cannot be empty", key),
                });
            }
        }
        
        // Validate cookies format
        for cookie in &session.cookies {
            if cookie.name.trim().is_empty() {
                return Err(AttackError::InvalidPayloadConfig {
                    reason: "Session cookie name cannot be empty".to_string(),
                });
            }
            if cookie.value.trim().is_empty() {
                return Err(AttackError::InvalidPayloadConfig {
                    reason: format!("Session cookie '{}' value cannot be empty", cookie.name),
                });
            }
        }
        
        Ok(())
    }

    /// Subscribe to session events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_sender.subscribe()
    }

    /// Get session statistics
    pub async fn get_session_statistics(&self) -> SessionStatistics {
        let sessions = self.sessions.read().await;
        let total_sessions = sessions.len();
        let active_sessions = sessions.values()
            .filter(|s| matches!(s.status, SessionStatus::Active) && !s.is_expired())
            .count();
        let expired_sessions = sessions.values()
            .filter(|s| s.is_expired())
            .count();
        let invalid_sessions = sessions.values()
            .filter(|s| matches!(s.status, SessionStatus::Invalid))
            .count();
        
        SessionStatistics {
            total_sessions,
            active_sessions,
            expired_sessions,
            invalid_sessions,
            validating_sessions: total_sessions - active_sessions - expired_sessions - invalid_sessions,
        }
    }
}

/// Session statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub invalid_sessions: usize,
    pub validating_sessions: usize,
}

impl Default for SessionSelectionCriteria {
    fn default() -> Self {
        Self {
            preferred_profile_ids: Vec::new(),
            max_validation_age_minutes: Some(60), // 1 hour
            min_usage_count: None,
            exclude_recent_failures: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proxy_common::session::{Cookie, SameSite, SessionMetadata};

    fn create_test_session() -> Session {
        let mut session = Session::new("Test Session".to_string(), None);
        session.headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        session.headers.insert("X-CSRF-Token".to_string(), "csrf123".to_string());
        session.cookies.push(Cookie {
            name: "sessionid".to_string(),
            value: "abc123".to_string(),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            expires: None,
            http_only: true,
            secure: true,
            same_site: Some(SameSite::Lax),
        });
        session.status = SessionStatus::Active;
        session
    }

    fn create_test_request() -> HttpRequestData {
        HttpRequestData {
            method: "GET".to_string(),
            url: "https://example.com/api/test".to_string(),
            headers: None,
            body: Vec::new(),
            tls: None,
        }
    }

    #[tokio::test]
    async fn test_session_management() {
        let manager = SessionManager::new();
        let session = create_test_session();
        let session_id = session.id;

        // Add session
        assert!(manager.add_session(session).await.is_ok());

        // Get session
        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Session");

        // Get all sessions
        let sessions = manager.get_sessions().await;
        assert_eq!(sessions.len(), 1);

        // Remove session
        assert!(manager.remove_session(&session_id).await.is_ok());
        assert!(manager.get_session(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_apply_session_to_request() {
        let manager = SessionManager::new();
        let session = create_test_session();
        let session_id = session.id;
        let request = create_test_request();

        // Add session
        manager.add_session(session).await.unwrap();

        // Apply session
        let result = manager.apply_session_to_request(
            request,
            &session_id,
            ExpirationHandling::Fail,
        ).await;

        assert!(result.is_ok());
        let (modified_request, application_result) = result.unwrap();

        // Check headers were applied
        assert!(modified_request.headers.is_some());
        let headers = modified_request.headers.unwrap();
        assert_eq!(headers.headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(headers.headers.get("X-CSRF-Token"), Some(&"csrf123".to_string()));
        assert!(headers.headers.contains_key("Cookie"));

        // Check application result
        assert_eq!(application_result.session_name, "Test Session");
        assert!(application_result.headers_applied > 0);
        assert_eq!(application_result.cookies_applied, 1);
    }

    #[tokio::test]
    async fn test_expired_session_handling() {
        let manager = SessionManager::new();
        let mut session = create_test_session();
        session.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        let session_id = session.id;
        let request = create_test_request();

        manager.add_session(session).await.unwrap();

        // Test fail handling
        let result = manager.apply_session_to_request(
            request.clone(),
            &session_id,
            ExpirationHandling::Fail,
        ).await;
        assert!(result.is_err());

        // Test continue without session handling
        let result = manager.apply_session_to_request(
            request,
            &session_id,
            ExpirationHandling::ContinueWithoutSession,
        ).await;
        assert!(result.is_ok());
        let (_, application_result) = result.unwrap();
        assert_eq!(application_result.headers_applied, 0);
        assert!(!application_result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_session_selection() {
        let manager = SessionManager::new();
        
        // Create multiple sessions
        let mut session1 = create_test_session();
        session1.name = "Session 1".to_string();
        session1.metadata.usage_count = 10;
        
        let mut session2 = create_test_session();
        session2.name = "Session 2".to_string();
        session2.metadata.usage_count = 5;
        session2.metadata.last_validated = Some(chrono::Utc::now());
        
        manager.add_session(session1).await.unwrap();
        manager.add_session(session2).await.unwrap();

        // Test selection with default criteria
        let criteria = SessionSelectionCriteria::default();
        let selected = manager.select_session(&criteria).await;
        assert!(selected.is_some());
        
        // Should prefer the more recently validated session
        assert_eq!(selected.unwrap().name, "Session 2");
    }

    #[tokio::test]
    async fn test_session_validation() {
        let manager = SessionManager::new();
        let session = create_test_session();
        let session_id = session.id;

        manager.add_session(session).await.unwrap();

        // Test validation
        let is_valid = manager.validate_session(&session_id, "https://example.com/test").await;
        assert!(is_valid.is_ok());
        assert!(is_valid.unwrap());

        // Test cached validation
        let is_valid_cached = manager.validate_session(&session_id, "https://example.com/test").await;
        assert!(is_valid_cached.is_ok());
        assert!(is_valid_cached.unwrap());
    }
}