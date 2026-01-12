//! Property-based tests for security and error handling
//! 
//! **Property 10: Security and Error Handling**
//! **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**

use crate::{
    SecurityManager, MaskingConfig, SecureString, AttackError, ErrorRecoveryStrategy,
    BackoffStrategy, CircuitBreaker, HttpRequestData, HttpResponseData, HttpHeaders
};
use proxy_common::session::{Session, Cookie, SameSite, SessionStatus};
use proptest::prelude::*;
use uuid::Uuid;

// Property test generators

/// Generate arbitrary HTTP request data with potentially sensitive information
fn arb_http_request_with_sensitive_data() -> impl Strategy<Value = HttpRequestData> {
    (
        prop::sample::select(vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "PATCH".to_string()]),
        "https://[a-z]{3,10}\\.com/[a-z]{1,20}(\\?[a-z_]+=[a-zA-Z0-9_\\-\\.]+(&[a-z_]+=[a-zA-Z0-9_\\-\\.]+)*)?",
        prop::option::of(arb_sensitive_headers()),
        prop::collection::vec(any::<u8>(), 0..1000),
    ).prop_map(|(method, url, headers, body)| {
        HttpRequestData {
            method,
            url,
            headers,
            body,
            tls: None,
        }
    })
}

/// Generate HTTP headers with sensitive data
fn arb_sensitive_headers() -> impl Strategy<Value = HttpHeaders> {
    prop::collection::hash_map(
        prop::sample::select(vec![
            "Authorization".to_string(), "Cookie".to_string(), "X-API-Key".to_string(), "X-Auth-Token".to_string(), 
            "Content-Type".to_string(), "User-Agent".to_string(), "Accept".to_string(), "X-CSRF-Token".to_string()
        ]),
        "[a-zA-Z0-9\\-\\._~\\+/=]{10,100}",
        1..8
    ).prop_map(|headers| HttpHeaders { headers })
}

/// Generate HTTP response data with potentially sensitive information
fn arb_http_response_with_sensitive_data() -> impl Strategy<Value = HttpResponseData> {
    (
        200..599i32,
        prop::option::of(arb_sensitive_headers()),
        prop::collection::vec(any::<u8>(), 0..1000),
    ).prop_map(|(status_code, headers, body)| {
        HttpResponseData {
            status_code,
            headers,
            body,
            tls: None,
        }
    })
}

/// Generate session data with sensitive information
fn arb_session_with_sensitive_data() -> impl Strategy<Value = Session> {
    (
        "[A-Za-z ]{5,20}",
        prop::collection::hash_map(
            prop::sample::select(vec![
                "Authorization".to_string(), "X-API-Key".to_string(), "X-Auth-Token".to_string(), "X-CSRF-Token".to_string()
            ]),
            "[a-zA-Z0-9\\-\\._~\\+/=]{20,100}",
            1..5
        ),
        prop::collection::vec(arb_sensitive_cookie(), 1..5),
    ).prop_map(|(name, headers, cookies)| {
        let mut session = Session::new(name, None);
        session.headers = headers;
        session.cookies = cookies;
        session.status = SessionStatus::Active;
        session
    })
}

/// Generate sensitive cookie data
fn arb_sensitive_cookie() -> impl Strategy<Value = Cookie> {
    (
        prop::sample::select(vec!["sessionid".to_string(), "auth".to_string(), "token".to_string(), "csrf".to_string(), "jsessionid".to_string()]),
        "[a-zA-Z0-9\\-\\._~\\+/=]{10,50}",
        prop::option::of("[a-z\\.]+"),
        prop::option::of("/[a-z/]*"),
    ).prop_map(|(name, value, domain, path)| {
        Cookie {
            name,
            value,
            domain,
            path,
            expires: None,
            http_only: true,
            secure: true,
            same_site: Some(SameSite::Lax),
        }
    })
}

/// Generate masking configuration
fn arb_masking_config() -> impl Strategy<Value = MaskingConfig> {
    (
        prop::collection::hash_set("[a-z\\-_]+", 1..10),
        prop::collection::hash_set("[a-z\\-_]+", 1..10),
        prop::collection::hash_set("[a-z\\-_]+", 1..10),
        prop::collection::vec("[a-zA-Z0-9\\[\\]\\(\\)\\{\\}\\*\\+\\?\\^\\$\\|\\\\\\-\\.\\s:=\"']+", 1..5),
        "\\*{3,10}",
        any::<bool>(),
        1..10usize,
        any::<bool>(),
    ).prop_map(|(sensitive_headers, sensitive_cookies, sensitive_url_params, 
                sensitive_body_patterns, mask_replacement, partial_masking, 
                partial_show_chars, masking_enabled)| {
        MaskingConfig {
            sensitive_headers,
            sensitive_cookies,
            sensitive_url_params,
            sensitive_body_patterns,
            mask_replacement,
            partial_masking,
            partial_show_chars,
            masking_enabled,
        }
    })
}

/// Generate attack errors
fn arb_attack_error() -> impl Strategy<Value = AttackError> {
    prop_oneof![
        "[a-z0-9\\-]+".prop_map(|agent_id| AttackError::AgentUnavailable { agent_id }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|reason| AttackError::InvalidPayloadConfig { reason }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|error| AttackError::ExecutionFailed { error }),
        any::<[u8; 16]>().prop_map(|bytes| AttackError::SessionExpired { session_id: Uuid::from_bytes(bytes) }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|reason| AttackError::PayloadGenerationFailed { reason }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|operation| AttackError::DatabaseError { operation }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|details| AttackError::NetworkError { details }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|reason| AttackError::ResourceAllocationFailed { reason }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|reason| AttackError::InvalidAttackConfig { reason }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|error| AttackError::SerializationError { error }),
        ("[a-zA-Z0-9_]+", "[a-zA-Z0-9 \\-_]+").prop_map(|(field, reason)| 
            AttackError::ValidationError { field, reason }),
        ("[a-zA-Z0-9_]+", "[a-zA-Z0-9 \\-_]+").prop_map(|(resource_type, details)| 
            AttackError::ResourceExhaustion { resource_type, details }),
        "[a-zA-Z0-9 \\-_]+".prop_map(|reason| AttackError::AuthenticationFailure { reason }),
        ("[a-zA-Z0-9_]+", "[a-zA-Z0-9 \\-_]+").prop_map(|(operation, reason)| 
            AttackError::PermissionDenied { operation, reason }),
        ("[a-zA-Z0-9_]+", 1..60000u64).prop_map(|(operation, duration_ms)| 
            AttackError::Timeout { operation, duration_ms }),
        ("[a-zA-Z0-9_]+", prop::option::of(1..300000u64)).prop_map(|(limit_type, retry_after_ms)| 
            AttackError::RateLimitExceeded { limit_type, retry_after_ms }),
        ("[a-zA-Z0-9_]+", "[a-zA-Z0-9 \\-_]+").prop_map(|(component, reason)| 
            AttackError::ConfigurationError { component, reason }),
        ("[a-zA-Z0-9_]+", "[a-zA-Z0-9 \\-_]+").prop_map(|(violation_type, details)| 
            AttackError::SecurityViolation { violation_type, details }),
    ]
}

/// Generate error recovery strategy
fn arb_error_recovery_strategy() -> impl Strategy<Value = ErrorRecoveryStrategy> {
    (
        1..10u32,
        arb_backoff_strategy(),
        prop::collection::vec("[a-z0-9\\-]+", 0..5),
        any::<bool>(),
        1..20u32,
        1000..300000u64,
    ).prop_map(|(max_retries, backoff_strategy, fallback_agents, 
                quick_failure_detection, circuit_breaker_threshold, circuit_breaker_timeout_ms)| {
        ErrorRecoveryStrategy {
            max_retries,
            backoff_strategy,
            fallback_agents,
            quick_failure_detection,
            circuit_breaker_threshold,
            circuit_breaker_timeout_ms,
        }
    })
}

/// Generate backoff strategy
fn arb_backoff_strategy() -> impl Strategy<Value = BackoffStrategy> {
    prop_oneof![
        (100..10000u64).prop_map(|delay_ms| BackoffStrategy::Fixed { delay_ms }),
        (100..5000u64, 1.1..3.0f64, 1000..60000u64).prop_map(|(initial_delay_ms, multiplier, max_delay_ms)| 
            BackoffStrategy::Exponential { initial_delay_ms, multiplier, max_delay_ms }),
        (100..5000u64, 100..2000u64).prop_map(|(initial_delay_ms, increment_ms)| 
            BackoffStrategy::Linear { initial_delay_ms, increment_ms }),
    ]
}

// Property tests

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 10.1: Sensitive Data Masking Consistency**
    /// For any HTTP request/response with sensitive data, masking should consistently
    /// hide sensitive information while preserving non-sensitive data structure
    #[test]
    fn prop_sensitive_data_masking_consistency(
        request in arb_http_request_with_sensitive_data(),
        response in arb_http_response_with_sensitive_data(),
        session in arb_session_with_sensitive_data(),
        config in arb_masking_config()
    ) {
        let security_manager = SecurityManager::with_config(config.clone());
        
        if config.masking_enabled {
            // Test request masking
            let masked_request = security_manager.mask_request(&request);
            
            // Sensitive headers should be masked
            if let Some(headers) = &masked_request.headers {
                for (key, value) in &headers.headers {
                    if config.sensitive_headers.contains(&key.to_lowercase()) {
                        prop_assert_ne!(value, request.headers.as_ref().unwrap().headers.get(key).unwrap());
                        prop_assert!(value.contains(&config.mask_replacement) || 
                                   (config.partial_masking && value.contains("***")));
                    }
                }
            }
            
            // Test response masking
            let masked_response = security_manager.mask_response(&response);
            
            // Sensitive headers should be masked
            if let Some(headers) = &masked_response.headers {
                for (key, value) in &headers.headers {
                    if config.sensitive_headers.contains(&key.to_lowercase()) {
                        if let Some(original_value) = response.headers.as_ref().and_then(|h| h.headers.get(key)) {
                            prop_assert_ne!(value, original_value);
                        }
                    }
                }
            }
            
            // Test session masking
            let masked_session = security_manager.mask_session(&session);
            
            // Sensitive session headers should be masked
            for (key, value) in &masked_session.headers {
                if config.sensitive_headers.contains(&key.to_lowercase()) {
                    prop_assert_ne!(value, session.headers.get(key).unwrap());
                    prop_assert!(value.contains(&config.mask_replacement) || 
                               (config.partial_masking && value.contains("***")));
                }
            }
            
            // Sensitive cookies should be masked
            for (masked_cookie, original_cookie) in masked_session.cookies.iter().zip(session.cookies.iter()) {
                if config.sensitive_cookies.iter().any(|sensitive| 
                    masked_cookie.name.to_lowercase().contains(sensitive)) {
                    prop_assert_ne!(&masked_cookie.value, &original_cookie.value);
                    prop_assert!(masked_cookie.value.contains(&config.mask_replacement) || 
                               (config.partial_masking && masked_cookie.value.contains("***")));
                }
            }
        } else {
            // When masking is disabled, data should be unchanged
            let masked_request = security_manager.mask_request(&request);
            prop_assert_eq!(masked_request.url, request.url);
            prop_assert_eq!(masked_request.body, request.body);
        }
    }

    /// **Property 10.2: Error Classification and Recovery Consistency**
    /// For any attack error, classification (severity, category, recoverability) should be
    /// consistent and recovery strategies should be appropriate for the error type
    #[test]
    fn prop_error_classification_consistency(
        error in arb_attack_error(),
        strategy in arb_error_recovery_strategy()
    ) {
        // Test error classification consistency
        let severity = error.severity();
        let category = error.category();
        let _is_recoverable = error.is_recoverable();
        let remediation = error.remediation();
        
        // Severity should be consistent with error type
        match &error {
            AttackError::SecurityViolation { .. } | AttackError::PermissionDenied { .. } => {
                prop_assert_eq!(severity, crate::ErrorSeverity::Critical);
            }
            AttackError::AuthenticationFailure { .. } | 
            AttackError::ConfigurationError { .. } |
            AttackError::InvalidAttackConfig { .. } |
            AttackError::ResourceExhaustion { .. } => {
                prop_assert_eq!(severity, crate::ErrorSeverity::High);
            }
            _ => {
                // Other errors can have various severities
            }
        }
        
        // Category should be consistent with error type
        match &error {
            AttackError::AgentUnavailable { .. } |
            AttackError::NetworkError { .. } |
            AttackError::DatabaseError { .. } |
            AttackError::ResourceAllocationFailed { .. } |
            AttackError::ResourceExhaustion { .. } |
            AttackError::Timeout { .. } => {
                prop_assert_eq!(category, crate::ErrorCategory::Infrastructure);
            }
            AttackError::InvalidPayloadConfig { .. } |
            AttackError::InvalidAttackConfig { .. } |
            AttackError::ConfigurationError { .. } |
            AttackError::ValidationError { .. } => {
                prop_assert_eq!(category, crate::ErrorCategory::Configuration);
            }
            AttackError::SessionExpired { .. } |
            AttackError::AuthenticationFailure { .. } |
            AttackError::PermissionDenied { .. } => {
                prop_assert_eq!(category, crate::ErrorCategory::Authentication);
            }
            AttackError::SecurityViolation { .. } => {
                prop_assert_eq!(category, crate::ErrorCategory::Security);
            }
            _ => {
                prop_assert_eq!(category, crate::ErrorCategory::Runtime);
            }
        }
        
        // Remediation should not be empty
        prop_assert!(!remediation.is_empty());
        prop_assert!(remediation.len() > 10); // Should be descriptive
        
        // Test backoff strategy calculation
        for attempt in 0..strategy.max_retries {
            let delay = strategy.backoff_strategy.calculate_delay(attempt);
            prop_assert!(delay > 0);
            
            // Exponential backoff should increase
            if let BackoffStrategy::Exponential { initial_delay_ms: _, multiplier: _, max_delay_ms } = &strategy.backoff_strategy {
                if attempt > 0 {
                    let prev_delay = strategy.backoff_strategy.calculate_delay(attempt - 1);
                    if delay < *max_delay_ms {
                        prop_assert!(delay >= prev_delay);
                    }
                }
            }
        }
    }

    /// **Property 10.3: Circuit Breaker State Consistency**
    /// For any circuit breaker, state transitions should be consistent and
    /// failure/success recording should maintain correct state
    #[test]
    fn prop_circuit_breaker_state_consistency(
        threshold in 1..20u32,
        timeout_ms in 1000..60000u64,
        failure_count in 0..50u32,
        success_count in 0..20u32
    ) {
        let mut circuit_breaker = CircuitBreaker::new(threshold, timeout_ms);
        
        // Initially should be closed and allow execution
        prop_assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
        prop_assert!(circuit_breaker.can_execute());
        
        // Record failures up to threshold
        for i in 0..failure_count {
            circuit_breaker.record_failure();
            
            if i + 1 >= threshold {
                // Should be open after reaching threshold
                prop_assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Open);
                prop_assert!(!circuit_breaker.can_execute());
            } else {
                // Should still be closed before threshold
                prop_assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
                prop_assert!(circuit_breaker.can_execute());
            }
        }
        
        // Record successes should reset to closed state
        for _ in 0..success_count {
            circuit_breaker.record_success();
            prop_assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
            prop_assert!(circuit_breaker.can_execute());
            prop_assert_eq!(circuit_breaker.failure_count(), 0);
        }
    }

    /// **Property 10.4: Secure String Masking Invariant**
    /// For any secure string, debug and display output should never expose
    /// the actual value when masking is enabled
    #[test]
    fn prop_secure_string_masking_invariant(
        value in "[a-zA-Z0-9\\-\\._~\\+/=]{10,100}",
        masked in any::<bool>()
    ) {
        let secure_string = if masked {
            SecureString::new(value.clone())
        } else {
            SecureString::unmasked(value.clone())
        };
        
        let debug_output = format!("{:?}", secure_string);
        let display_output = format!("{}", secure_string);
        
        if masked {
            // When masked, output should not contain the actual value
            prop_assert!(!debug_output.contains(&value));
            prop_assert!(!display_output.contains(&value));
            prop_assert!(debug_output.contains("***MASKED***"));
            prop_assert_eq!(display_output, "***MASKED***");
        } else {
            // When not masked, output should contain the actual value
            prop_assert!(debug_output.contains(&value));
            prop_assert_eq!(display_output, value.clone());
        }
        
        // Expose should always return the actual value
        prop_assert_eq!(secure_string.expose(), &value);
        prop_assert_eq!(secure_string.len(), value.len());
        prop_assert_eq!(secure_string.is_empty(), value.is_empty());
        prop_assert_eq!(secure_string.is_masked(), masked);
    }

    /// **Property 10.5: Input Validation Completeness**
    /// For any input data, validation should provide clear error messages
    /// and appropriate remediation suggestions for invalid inputs
    #[test]
    fn prop_input_validation_completeness(
        field_name in "[a-zA-Z_][a-zA-Z0-9_]*",
        error_message in "[a-zA-Z0-9 \\-_\\.]+",
        error_code in "[A-Z_]+",
        suggested_fix in "[a-zA-Z0-9 \\-_\\.,]+",
    ) {
        let validation_error = crate::ValidationError {
            field: field_name.clone(),
            message: error_message.clone(),
            error_code: error_code.clone(),
            suggested_fix: suggested_fix.clone(),
        };
        
        // All fields should be non-empty
        prop_assert!(!validation_error.field.is_empty());
        prop_assert!(!validation_error.message.is_empty());
        prop_assert!(!validation_error.error_code.is_empty());
        prop_assert!(!validation_error.suggested_fix.is_empty());
        
        // Field name should be valid identifier
        prop_assert!(validation_error.field.chars().next().unwrap().is_alphabetic() || 
                    validation_error.field.chars().next().unwrap() == '_');
        
        // Error code should be uppercase
        prop_assert_eq!(&validation_error.error_code, &validation_error.error_code.to_uppercase());
        
        // Message and suggested fix should be descriptive
        prop_assert!(validation_error.message.len() >= 5);
        prop_assert!(validation_error.suggested_fix.len() >= 5);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_security_manager_basic_functionality() {
        let security_manager = SecurityManager::new();
        
        // Test basic masking functionality
        let mut request = HttpRequestData::new("POST".to_string(), "https://example.com/login?token=secret123".to_string());
        request.set_header("Authorization".to_string(), "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string());
        request.set_body_string(r#"{"password":"secret123","api_key":"sk-abcdef123456"}"#.to_string());
        
        let masked_request = security_manager.mask_request(&request);
        
        // Check that sensitive data is masked
        assert!(masked_request.url.contains("token=***MASKED***"));
        assert_eq!(masked_request.headers.as_ref().unwrap().headers.get("Authorization").unwrap(), "***MASKED***");
        
        let body_str = String::from_utf8(masked_request.body).unwrap();
        assert!(body_str.contains("\"password\": \"***MASKED***\""));
        assert!(body_str.contains("\"api_key\": \"***MASKED***\""));
    }

    #[test]
    fn test_error_classification() {
        let errors = vec![
            AttackError::SecurityViolation { violation_type: "test".to_string(), details: "test".to_string() },
            AttackError::AgentUnavailable { agent_id: "test".to_string() },
            AttackError::NetworkError { details: "test".to_string() },
            AttackError::ValidationError { field: "test".to_string(), reason: "test".to_string() },
        ];
        
        for error in errors {
            // All errors should have valid classification
            let severity = error.severity();
            let category = error.category();
            let _is_recoverable = error.is_recoverable();
            let remediation = error.remediation();
            
            // Remediation should be descriptive
            assert!(!remediation.is_empty());
            assert!(remediation.len() > 10);
            
            // Severity should be valid
            assert!(matches!(severity, crate::ErrorSeverity::Low | crate::ErrorSeverity::Medium | 
                           crate::ErrorSeverity::High | crate::ErrorSeverity::Critical));
            
            // Category should be valid
            assert!(matches!(category, crate::ErrorCategory::Infrastructure | crate::ErrorCategory::Configuration |
                           crate::ErrorCategory::Authentication | crate::ErrorCategory::Security | 
                           crate::ErrorCategory::Runtime));
        }
    }

    #[test]
    fn test_circuit_breaker_functionality() {
        let mut circuit_breaker = CircuitBreaker::new(3, 60000);
        
        // Initially closed
        assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
        assert!(circuit_breaker.can_execute());
        
        // Record failures
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
        
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
        
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Open);
        assert!(!circuit_breaker.can_execute());
        
        // Record success should reset
        circuit_breaker.record_success();
        assert_eq!(circuit_breaker.state(), &crate::CircuitBreakerState::Closed);
        assert!(circuit_breaker.can_execute());
        assert_eq!(circuit_breaker.failure_count(), 0);
    }

    #[test]
    fn test_secure_string_behavior() {
        let secret = "very_secret_password_123";
        
        // Masked secure string
        let masked_secure = SecureString::new(secret.to_string());
        assert!(masked_secure.is_masked());
        assert_eq!(masked_secure.expose(), secret);
        assert_eq!(masked_secure.len(), secret.len());
        assert!(!masked_secure.is_empty());
        
        let debug_output = format!("{:?}", masked_secure);
        let display_output = format!("{}", masked_secure);
        
        assert!(!debug_output.contains(secret));
        assert!(!display_output.contains(secret));
        assert!(debug_output.contains("***MASKED***"));
        assert_eq!(display_output, "***MASKED***");
        
        // Unmasked secure string
        let unmasked_secure = SecureString::unmasked(secret.to_string());
        assert!(!unmasked_secure.is_masked());
        assert_eq!(unmasked_secure.expose(), secret);
        
        let debug_output = format!("{:?}", unmasked_secure);
        let display_output = format!("{}", unmasked_secure);
        
        assert!(debug_output.contains(secret));
        assert_eq!(display_output, secret);
    }
}