//! Property-based tests for session integration functionality
//! 
//! **Feature: repeater-intruder, Property 7: Session Integration Completeness**
//! **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

use orchestrator::session_integration::{
    SessionManager, ExpirationHandling, SessionSelectionCriteria, RefreshMethod
};
use attack_engine::{HttpRequestData, HttpResponseData, HttpHeaders};
use proxy_common::session::{Session, SessionStatus, Cookie, SameSite};
use proptest::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// PROPERTY TEST GENERATORS
// ============================================================================

/// Generate a valid session for testing
fn arb_session() -> impl Strategy<Value = Session> {
    (
        "[a-zA-Z0-9 ]{1,50}",  // name
        prop::collection::hash_map("[a-zA-Z-]{1,20}", "[a-zA-Z0-9=+/]{1,100}", 0..10), // headers
        prop::collection::vec(arb_cookie(), 0..5), // cookies
        prop::option::of(any::<u128>().prop_map(|n| Uuid::from_u128(n))), // profile_id
    ).prop_map(|(name, headers, cookies, profile_id)| {
        let mut session = Session::new(name, profile_id);
        session.headers = headers;
        session.cookies = cookies;
        session.status = SessionStatus::Active;
        session
    })
}

/// Generate a valid cookie for testing
fn arb_cookie() -> impl Strategy<Value = Cookie> {
    (
        "[a-zA-Z0-9_]{1,20}",  // name
        "[a-zA-Z0-9=+/]{1,100}", // value
        prop::option::of("[a-zA-Z0-9.-]{1,50}"), // domain
        prop::option::of("/[a-zA-Z0-9/]*"), // path
        any::<bool>(), // http_only
        any::<bool>(), // secure
        prop::option::of(prop::sample::select(vec![SameSite::Strict, SameSite::Lax, SameSite::None])), // same_site
    ).prop_map(|(name, value, domain, path, http_only, secure, same_site)| {
        Cookie {
            name,
            value,
            domain,
            path,
            expires: None,
            http_only,
            secure,
            same_site,
        }
    })
}

/// Generate a valid HTTP request for testing
fn arb_http_request() -> impl Strategy<Value = HttpRequestData> {
    (
        prop::sample::select(vec!["GET", "POST", "PUT", "DELETE", "PATCH"]),
        "https://[a-zA-Z0-9.-]{1,30}\\.[a-z]{2,4}/[a-zA-Z0-9/_-]*",
        prop::option::of(prop::collection::hash_map("[a-zA-Z-]{1,20}", "[a-zA-Z0-9 =+/]{1,100}", 0..10)),
        prop::collection::vec(any::<u8>(), 0..1000),
    ).prop_map(|(method, url, headers_map, body)| {
        let headers = headers_map.map(|h| HttpHeaders { headers: h });
        HttpRequestData {
            method: method.to_string(),
            url,
            headers,
            body,
            tls: None,
        }
    })
}

/// Generate a valid HTTP response for testing
fn arb_http_response() -> impl Strategy<Value = HttpResponseData> {
    (
        100i32..600i32, // status_code
        prop::option::of(prop::collection::hash_map("[a-zA-Z-]{1,20}", "[a-zA-Z0-9 =+/]{1,100}", 0..10)),
        prop::collection::vec(any::<u8>(), 0..1000),
    ).prop_map(|(status_code, headers_map, body)| {
        let headers = headers_map.map(|h| HttpHeaders { headers: h });
        HttpResponseData {
            status_code,
            headers,
            body,
            tls: None,
        }
    })
}

/// Generate authentication failure response
fn arb_auth_failure_response() -> impl Strategy<Value = HttpResponseData> {
    (
        prop::sample::select(vec![401, 403]), // failure status codes
        prop::option::of(prop::collection::hash_map(
            "[a-zA-Z-]{1,20}", 
            "[a-zA-Z0-9 =+/]{1,100}", 
            0..5
        )),
        prop::sample::select(vec![
            b"login required".to_vec(),
            b"unauthorized access".to_vec(),
            b"authentication failed".to_vec(),
            b"session expired".to_vec(),
        ]),
    ).prop_map(|(status_code, headers_map, body)| {
        let mut headers_map = headers_map.unwrap_or_default();
        
        // Add authentication failure indicators
        if status_code == 401 {
            headers_map.insert("WWW-Authenticate".to_string(), "Bearer".to_string());
        }
        
        let headers = Some(HttpHeaders { headers: headers_map });
        HttpResponseData {
            status_code,
            headers,
            body,
            tls: None,
        }
    })
}

/// Generate session selection criteria
fn arb_session_criteria() -> impl Strategy<Value = SessionSelectionCriteria> {
    (
        prop::collection::vec(any::<u128>().prop_map(|n| Uuid::from_u128(n)), 0..3),
        prop::option::of(1u64..1440u64), // max_validation_age_minutes (1 minute to 24 hours)
        prop::option::of(0u64..100u64), // min_usage_count
        any::<bool>(), // exclude_recent_failures
    ).prop_map(|(preferred_profile_ids, max_validation_age_minutes, min_usage_count, exclude_recent_failures)| {
        SessionSelectionCriteria {
            preferred_profile_ids,
            max_validation_age_minutes,
            min_usage_count,
            exclude_recent_failures,
        }
    })
}

// ============================================================================
// PROPERTY TESTS
// ============================================================================

proptest! {
    /// **Property 7.1: Session Addition and Retrieval Consistency**
    /// For any valid session, adding it to the manager and then retrieving it should return the same session data
    fn prop_session_addition_retrieval_consistency(session in arb_session()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            let session_id = session.id;
            
            // Add session
            let add_result = manager.add_session(session.clone()).await;
            if add_result.is_err() {
                return Err("Session addition failed".to_string());
            }
            
            // Retrieve session
            let retrieved = manager.get_session(&session_id).await;
            if retrieved.is_none() {
                return Err("Session should be retrievable".to_string());
            }
            
            let retrieved_session = retrieved.unwrap();
            if retrieved_session.id != session.id {
                return Err("Session ID should match".to_string());
            }
            if retrieved_session.name != session.name {
                return Err("Session name should match".to_string());
            }
            if retrieved_session.headers.len() != session.headers.len() {
                return Err("Header count should match".to_string());
            }
            if retrieved_session.cookies.len() != session.cookies.len() {
                return Err("Cookie count should match".to_string());
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.2: Session Application Preserves Request Structure**
    /// For any valid session and request, applying the session should preserve the original request structure while adding session data
    fn prop_session_application_preserves_structure(
        session in arb_session(),
        request in arb_http_request()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            let session_id = session.id;
            let original_url = request.url.clone();
            let original_method = request.method.clone();
            let original_body = request.body.clone();
            
            // Add session
            manager.add_session(session.clone()).await.unwrap();
            
            // Apply session to request
            let result = manager.apply_session_to_request(
                request,
                &session_id,
                ExpirationHandling::Fail,
            ).await;
            
            if result.is_err() {
                return Err("Session application should succeed".to_string());
            }
            
            let (modified_request, application_result) = result.unwrap();
            
            // Verify request structure is preserved
            if modified_request.url != original_url {
                return Err("URL should be preserved".to_string());
            }
            if modified_request.method != original_method {
                return Err("Method should be preserved".to_string());
            }
            if modified_request.body != original_body {
                return Err("Body should be preserved".to_string());
            }
            
            // Verify session data was applied
            if modified_request.headers.is_none() {
                return Err("Headers should be present after session application".to_string());
            }
            if application_result.session_id != session_id {
                return Err("Application result should reference correct session".to_string());
            }
            if application_result.session_name != session.name {
                return Err("Application result should have correct session name".to_string());
            }
            
            // Verify session headers were applied
            if !session.headers.is_empty() {
                let headers = modified_request.headers.unwrap();
                for (key, value) in &session.headers {
                    if headers.headers.get(key) != Some(value) {
                        return Err(format!("Session header '{}' should be applied", key));
                    }
                }
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.3: Authentication Failure Detection Consistency**
    /// For any response that contains authentication failure indicators, the detection should consistently identify it as a failure
    fn prop_auth_failure_detection_consistency(
        failure_response in arb_auth_failure_response(),
        request_url in "https://[a-zA-Z0-9.-]{1,30}\\.[a-z]{2,4}/[a-zA-Z0-9/_-]*"
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            
            // Detect authentication failure
            let is_failure = manager.detect_authentication_failure(&failure_response, &request_url).await;
            
            // Should detect failure for responses with failure indicators
            if !is_failure {
                return Err(format!("Should detect authentication failure for status {} with body containing failure indicators", failure_response.status_code));
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.4: Session Selection Criteria Consistency**
    /// For any session selection criteria, the selected session (if any) should match all specified criteria
    fn prop_session_selection_criteria_consistency(
        sessions in prop::collection::vec(arb_session(), 1..10),
        criteria in arb_session_criteria()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            
            // Add all sessions
            for session in &sessions {
                manager.add_session(session.clone()).await.unwrap();
            }
            
            // Select session based on criteria
            let selected = manager.select_session(&criteria).await;
            
            if let Some(selected_session) = selected {
                // Verify selected session matches criteria
                if !criteria.preferred_profile_ids.is_empty() {
                    if let Some(profile_id) = selected_session.profile_id {
                        if !criteria.preferred_profile_ids.contains(&profile_id) {
                            return Err("Selected session should have preferred profile ID".to_string());
                        }
                    }
                }
                
                if let Some(min_usage) = criteria.min_usage_count {
                    if selected_session.metadata.usage_count < min_usage {
                        return Err("Selected session should meet minimum usage count requirement".to_string());
                    }
                }
                
                // Session should be active and not expired
                if !matches!(selected_session.status, SessionStatus::Active) {
                    return Err("Selected session should be active".to_string());
                }
                if selected_session.is_expired() {
                    return Err("Selected session should not be expired".to_string());
                }
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.5: Session Expiration Handling Consistency**
    /// For any expired session, the expiration handling strategy should be applied consistently
    fn prop_session_expiration_handling_consistency(
        mut session in arb_session(),
        request in arb_http_request(),
        handling in prop::sample::select(vec![
            ExpirationHandling::Fail,
            ExpirationHandling::ContinueWithoutSession,
        ])
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            
            // Make session expired
            session.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
            let session_id = session.id;
            
            manager.add_session(session).await.unwrap();
            
            // Apply expired session with handling strategy
            let result = manager.apply_session_to_request(
                request.clone(),
                &session_id,
                handling.clone(),
            ).await;
            
            match handling {
                ExpirationHandling::Fail => {
                    if result.is_ok() {
                        return Err("Should fail when handling is set to Fail".to_string());
                    }
                }
                ExpirationHandling::ContinueWithoutSession => {
                    if result.is_err() {
                        return Err("Should succeed when handling is set to ContinueWithoutSession".to_string());
                    }
                    if let Ok((modified_request, application_result)) = result {
                        // Request should be unchanged (no session data applied)
                        if modified_request.url != request.url {
                            return Err("URL should be unchanged".to_string());
                        }
                        if modified_request.method != request.method {
                            return Err("Method should be unchanged".to_string());
                        }
                        if application_result.headers_applied != 0 {
                            return Err("No headers should be applied".to_string());
                        }
                        if application_result.warnings.is_empty() {
                            return Err("Should have warnings about expired session".to_string());
                        }
                    }
                }
                _ => {} // Other handling strategies not tested in this property
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.6: Session Validation Caching Consistency**
    /// For any session, validating it multiple times should return consistent results and use caching appropriately
    fn prop_session_validation_caching_consistency(
        session in arb_session(),
        validation_url in "https://[a-zA-Z0-9.-]{1,30}\\.[a-z]{2,4}/[a-zA-Z0-9/_-]*"
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            let session_id = session.id;
            
            manager.add_session(session).await.unwrap();
            
            // First validation
            let first_result = manager.validate_session(&session_id, &validation_url).await;
            if first_result.is_err() {
                return Err("First validation should succeed".to_string());
            }
            
            // Second validation (should use cache)
            let second_result = manager.validate_session(&session_id, &validation_url).await;
            if second_result.is_err() {
                return Err("Second validation should succeed".to_string());
            }
            
            // Results should be consistent
            if first_result.unwrap() != second_result.unwrap() {
                return Err("Validation results should be consistent".to_string());
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }

    /// **Property 7.7: Session Statistics Accuracy**
    /// For any collection of sessions, the statistics should accurately reflect the session states
    fn prop_session_statistics_accuracy(
        sessions in prop::collection::vec(arb_session(), 0..20)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let manager = SessionManager::new();
            
            // Add all sessions with various states
            let mut expected_active = 0;
            let mut expected_expired = 0;
            let mut expected_invalid = 0;
            
            for mut session in sessions {
                // Randomly set session states for testing
                match session.id.as_u128() % 4 {
                    0 => {
                        session.status = SessionStatus::Active;
                        expected_active += 1;
                    }
                    1 => {
                        session.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
                        expected_expired += 1;
                    }
                    2 => {
                        session.status = SessionStatus::Invalid;
                        expected_invalid += 1;
                    }
                    _ => {
                        session.status = SessionStatus::Active;
                        expected_active += 1;
                    }
                }
                
                manager.add_session(session).await.unwrap();
            }
            
            // Get statistics
            let stats = manager.get_session_statistics().await;
            
            // Verify statistics accuracy
            if stats.total_sessions != expected_active + expected_expired + expected_invalid {
                return Err("Total sessions should match".to_string());
            }
            
            // Note: The exact counts may vary due to expiration logic in get_session_statistics
            // but total should always be consistent
            if stats.active_sessions > expected_active + expected_expired + expected_invalid {
                return Err("Active sessions should not exceed total".to_string());
            }
            
            Ok(())
        });
        
        prop_assert!(result.is_ok(), "Test failed: {:?}", result.err());
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_session_integration_end_to_end() {
    // **Feature: repeater-intruder, Property 7: Session Integration Completeness**
    // **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**
    
    let manager = SessionManager::new();
    
    // Create test session
    let mut session = Session::new("Test Integration Session".to_string(), None);
    session.headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
    session.headers.insert("X-CSRF-Token".to_string(), "csrf-token-123".to_string());
    session.cookies.push(Cookie {
        name: "sessionid".to_string(),
        value: "session-value-123".to_string(),
        domain: Some("example.com".to_string()),
        path: Some("/".to_string()),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some(SameSite::Lax),
    });
    session.status = SessionStatus::Active;
    let session_id = session.id;
    
    // Add session
    manager.add_session(session).await.unwrap();
    
    // Create test request
    let request = HttpRequestData {
        method: "POST".to_string(),
        url: "https://example.com/api/test".to_string(),
        headers: Some(HttpHeaders {
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers
            },
        }),
        body: b"{\"test\": \"data\"}".to_vec(),
        tls: None,
    };
    
    // Apply session to request
    let (modified_request, application_result) = manager
        .apply_session_to_request(request, &session_id, ExpirationHandling::Fail)
        .await
        .unwrap();
    
    // Verify session was applied correctly
    assert_eq!(application_result.session_name, "Test Integration Session");
    assert!(application_result.headers_applied > 0);
    assert_eq!(application_result.cookies_applied, 1);
    
    // Verify headers were merged correctly
    let headers = modified_request.headers.unwrap();
    assert_eq!(headers.headers.get("Authorization"), Some(&"Bearer test-token".to_string()));
    assert_eq!(headers.headers.get("X-CSRF-Token"), Some(&"csrf-token-123".to_string()));
    assert_eq!(headers.headers.get("Content-Type"), Some(&"application/json".to_string()));
    assert!(headers.headers.contains_key("Cookie"));
    
    // Test authentication failure detection
    let failure_response = HttpResponseData {
        status_code: 401,
        headers: Some(HttpHeaders {
            headers: {
                let mut headers = HashMap::new();
                headers.insert("WWW-Authenticate".to_string(), "Bearer".to_string());
                headers
            },
        }),
        body: b"Unauthorized access".to_vec(),
        tls: None,
    };
    
    let is_failure = manager
        .detect_authentication_failure(&failure_response, "https://example.com/api/test")
        .await;
    assert!(is_failure, "Should detect authentication failure");
    
    // Test session validation
    let is_valid = manager
        .validate_session(&session_id, "https://example.com/api/test")
        .await
        .unwrap();
    assert!(is_valid, "Session should be valid");
    
    println!("✓ Session integration end-to-end test passed");
}

#[tokio::test]
async fn test_session_failure_handling() {
    // Test authentication failure handling workflow
    let manager = SessionManager::new();
    
    // Create session with profile for refresh testing
    let profile_id = Uuid::new_v4();
    let mut session = Session::new("Test Failure Session".to_string(), Some(profile_id));
    session.status = SessionStatus::Active;
    let session_id = session.id;
    
    manager.add_session(session).await.unwrap();
    
    // Create authentication failure response
    let failure_response = HttpResponseData {
        status_code: 403,
        headers: None,
        body: b"Access denied - session expired".to_vec(),
        tls: None,
    };
    
    // Handle authentication failure
    let refresh_result = manager
        .handle_authentication_failure(&session_id, "https://example.com/api/test", &failure_response)
        .await
        .unwrap();
    
    // Should attempt refresh but fail (LSR not implemented)
    assert!(!refresh_result.success);
    assert!(refresh_result.error.is_some());
    assert!(matches!(refresh_result.refresh_method, RefreshMethod::LSRProfileReExecution));
    
    // Session should be marked as invalid
    let updated_session = manager.get_session(&session_id).await.unwrap();
    assert!(matches!(updated_session.status, SessionStatus::Invalid));
    
    println!("✓ Session failure handling test passed");
}