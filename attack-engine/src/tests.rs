//! Property-based tests for attack engine core types

pub mod security_error_handling_test;

#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::payload::*;
    use crate::parser::*;
    use crate::attack_modes::{AttackMode, AttackModeExecutor, AttackModeFactory, AttackRequest as AttackModeRequest};
    use proptest::prelude::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    use proxy_common::Session;

    // Property test generators
    prop_compose! {
        fn arb_http_headers()
            (headers in prop::collection::hash_map(".*", ".*", 0..10))
        -> HttpHeaders {
            HttpHeaders { headers }
        }
    }

    prop_compose! {
        fn arb_tls_details()
            (version in ".*", cipher in ".*")
        -> TlsDetails {
            TlsDetails { version, cipher }
        }
    }

    prop_compose! {
        fn arb_http_request_data()
            (
                method in "(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)",
                url in "https?://[a-zA-Z0-9.-]+(/[a-zA-Z0-9._-]*)*",
                headers in prop::option::of(arb_http_headers()),
                body in prop::collection::vec(any::<u8>(), 0..1000),
                tls in prop::option::of(arb_tls_details())
            )
        -> HttpRequestData {
            HttpRequestData {
                method,
                url,
                headers,
                body,
                tls,
            }
        }
    }

    prop_compose! {
        fn arb_http_response_data()
            (
                status_code in 100i32..600i32,
                headers in prop::option::of(arb_http_headers()),
                body in prop::collection::vec(any::<u8>(), 0..1000),
                tls in prop::option::of(arb_tls_details())
            )
        -> HttpResponseData {
            HttpResponseData {
                status_code,
                headers,
                body,
                tls,
            }
        }
    }

    prop_compose! {
        fn arb_execution_config()
            (
                concurrent_requests_per_agent in 1u32..100u32,
                timeout_seconds in 1u64..300u64,
                retry_attempts in 0u32..10u32,
                distribution_strategy in prop_oneof![
                    Just(DistributionStrategy::RoundRobin),
                    (1usize..100usize).prop_map(|size| DistributionStrategy::Batch { batch_size: size }),
                    Just(DistributionStrategy::LoadBalanced)
                ]
            )
        -> ExecutionConfig {
            ExecutionConfig {
                concurrent_requests_per_agent,
                timeout_seconds,
                retry_attempts,
                distribution_strategy,
            }
        }
    }

    prop_compose! {
        fn arb_attack_request()
            (
                request_template in arb_http_request_data(),
                target_agents in prop::collection::vec("[a-zA-Z0-9-]+", 1..10),
                execution_config in arb_execution_config()
            )
        -> AttackRequest {
            AttackRequest {
                id: Uuid::new_v4(),
                request_template,
                target_agents,
                execution_config,
                session_data: None,
            }
        }
    }

    prop_compose! {
        fn arb_attack_result()
            (
                request_id in any::<[u8; 16]>().prop_map(Uuid::from_bytes),
                agent_id in "[a-zA-Z0-9-]+",
                request_data in arb_http_request_data(),
                response_data in prop::option::of(arb_http_response_data()),
                duration_ms in prop::option::of(1u64..10000u64),
                error in prop::option::of(".*")
            )
        -> AttackResult {
            AttackResult {
                id: Uuid::new_v4(),
                request_id,
                agent_id,
                request_data,
                response_data,
                executed_at: chrono::Utc::now(),
                duration_ms,
                error,
            }
        }
    }

    proptest! {
        /// **Feature: repeater-intruder, Property 1: Request Processing Integrity**
        /// 
        /// For any HTTP request sent through repeater or intruder, the system should preserve 
        /// request structure while applying modifications and capture complete response data 
        /// including headers, body, status code, and timing information.
        /// 
        /// **Validates: Requirements 1.1, 1.2, 1.4, 1.5**
        #[test]
        fn property_request_processing_integrity(
            mut request in arb_http_request_data(),
            session_headers in prop::collection::hash_map(".*", ".*", 0..5)
        ) {
            // Create a mock session
            let mut session = Session::new("test-session".to_string(), None);
            for (key, value) in session_headers.iter() {
                session.headers.insert(key.clone(), value.clone());
            }

            // Store original request data for comparison
            let original_method = request.method.clone();
            let original_url = request.url.clone();
            let original_body = request.body.clone();
            let original_header_count = request.headers.as_ref().map(|h| h.headers.len()).unwrap_or(0);

            // Apply session data to request
            request.apply_session(&session);

            // Verify request structure is preserved
            prop_assert_eq!(request.method, original_method);
            prop_assert_eq!(request.url, original_url);
            prop_assert_eq!(request.body, original_body);

            // Verify session headers were applied
            if let Some(ref headers) = request.headers {
                // Should have at least the original headers plus session headers
                prop_assert!(headers.headers.len() >= original_header_count);
                
                // All session headers should be present
                for (key, value) in session_headers.iter() {
                    prop_assert_eq!(headers.headers.get(key), Some(value));
                }
            }
        }

        #[test]
        fn property_http_request_header_management(
            mut request in arb_http_request_data(),
            header_key in "[a-zA-Z-]+",
            header_value in ".*"
        ) {
            // Set a header
            request.set_header(header_key.clone(), header_value.clone());
            
            // Verify header was set
            prop_assert_eq!(request.get_header(&header_key), Some(&header_value));
            
            // Verify headers structure exists
            prop_assert!(request.headers.is_some());
        }

        #[test]
        fn property_http_request_body_handling(
            mut request in arb_http_request_data(),
            body_string in ".*"
        ) {
            // Set body from string
            request.set_body_string(body_string.clone());
            
            // Verify body was set correctly
            let retrieved_body = request.body_as_string().unwrap();
            prop_assert_eq!(retrieved_body, body_string.clone());
            
            // Verify body bytes match
            let expected_bytes = body_string.as_bytes().to_vec();
            prop_assert_eq!(request.body, expected_bytes);
        }

        #[test]
        fn property_http_response_success_detection(
            response in arb_http_response_data()
        ) {
            // Success should be determined by status code range
            let expected_success = response.status_code >= 200 && response.status_code < 300;
            prop_assert_eq!(response.is_success(), expected_success);
        }

        #[test]
        fn property_http_response_body_operations(
            response in arb_http_response_data()
        ) {
            // Body length should match actual body size
            prop_assert_eq!(response.body_length(), response.body.len());
            
            // If body is valid UTF-8, string conversion should work
            if let Ok(body_string) = String::from_utf8(response.body.clone()) {
                prop_assert_eq!(response.body_as_string().unwrap(), body_string);
            }
        }

        #[test]
        fn property_attack_request_creation_and_modification(
            request_template in arb_http_request_data(),
            target_agents in prop::collection::vec("[a-zA-Z0-9-]+", 1..10),
            config in arb_execution_config()
        ) {
            // Create attack request
            let attack_request = AttackRequest::new(
                request_template.clone(),
                target_agents.clone()
            ).with_config(config.clone());

            // Verify all fields are set correctly
            prop_assert_eq!(attack_request.request_template.method, request_template.method);
            prop_assert_eq!(attack_request.request_template.url, request_template.url);
            prop_assert_eq!(attack_request.target_agents, target_agents);
            prop_assert_eq!(attack_request.execution_config.concurrent_requests_per_agent, 
                           config.concurrent_requests_per_agent);
            prop_assert_eq!(attack_request.execution_config.timeout_seconds, config.timeout_seconds);
            prop_assert!(attack_request.session_data.is_none());
        }

        #[test]
        fn property_attack_result_creation_and_status(
            request_id in any::<[u8; 16]>().prop_map(Uuid::from_bytes),
            agent_id in "[a-zA-Z0-9-]+",
            request_data in arb_http_request_data(),
            response_data in arb_http_response_data(),
            duration_ms in 1u64..10000u64,
            error_msg in ".*"
        ) {
            // Test successful result
            let success_result = AttackResult::new(
                request_id,
                agent_id.clone(),
                request_data.clone()
            ).with_response(response_data.clone(), duration_ms);

            prop_assert!(success_result.is_success() == response_data.is_success());
            prop_assert!(success_result.error.is_none());
            prop_assert_eq!(success_result.duration_ms, Some(duration_ms));
            prop_assert_eq!(success_result.agent_id, agent_id.clone());

            // Test error result
            let error_result = AttackResult::new(
                request_id,
                agent_id.clone(),
                request_data.clone()
            ).with_error(error_msg.clone());

            prop_assert!(!error_result.is_success());
            prop_assert_eq!(error_result.error, Some(error_msg));
            prop_assert!(error_result.response_data.is_none());
        }

        #[test]
        fn property_serialization_roundtrip(
            attack_request in arb_attack_request(),
            attack_result in arb_attack_result()
        ) {
            // Test AttackRequest serialization roundtrip
            let request_json = serde_json::to_string(&attack_request).unwrap();
            let deserialized_request: AttackRequest = serde_json::from_str(&request_json).unwrap();
            
            prop_assert_eq!(attack_request.request_template.method, deserialized_request.request_template.method);
            prop_assert_eq!(attack_request.request_template.url, deserialized_request.request_template.url);
            prop_assert_eq!(attack_request.target_agents, deserialized_request.target_agents);

            // Test AttackResult serialization roundtrip
            let result_json = serde_json::to_string(&attack_result).unwrap();
            let deserialized_result: AttackResult = serde_json::from_str(&result_json).unwrap();
            
            prop_assert_eq!(attack_result.agent_id, deserialized_result.agent_id);
            prop_assert_eq!(attack_result.request_id, deserialized_result.request_id);
            prop_assert_eq!(attack_result.error, deserialized_result.error);
        }
    }

    // Property test generators for payload system
    prop_compose! {
        fn arb_payload_config()
            (config_type in 0..3u8)
        -> PayloadConfig {
            match config_type {
                0 => PayloadConfig::Wordlist {
                    file_path: "/tmp/test_wordlist.txt".to_string(),
                    encoding: "utf-8".to_string(),
                },
                1 => PayloadConfig::NumberRange {
                    start: 1,
                    end: 100,
                    step: 1,
                    format: "{}".to_string(),
                },
                _ => PayloadConfig::Custom {
                    values: vec!["test1".to_string(), "test2".to_string(), "test3".to_string()],
                },
            }
        }
    }

    prop_compose! {
        fn arb_number_range_config()
            (
                start in -1000i64..1000i64,
                end in -1000i64..1000i64,
                step in 1i64..10i64,
                format in prop_oneof![
                    Just("{}".to_string()),
                    Just("user{}".to_string()),
                    Just("%d".to_string()),
                    Just("%x".to_string())
                ]
            )
        -> PayloadConfig {
            let (start, end) = if start <= end { (start, end) } else { (end, start) };
            PayloadConfig::NumberRange { start, end, step, format }
        }
    }

    prop_compose! {
        fn arb_custom_payload_config()
            (values in prop::collection::vec("[a-zA-Z0-9_-]+", 1..20))
        -> PayloadConfig {
            PayloadConfig::Custom { values }
        }
    }

    prop_compose! {
        fn arb_template_with_markers()
            (
                prefix in "[a-zA-Z0-9/._-]*",
                marker1 in "[a-zA-Z0-9_-]+",
                middle in "[a-zA-Z0-9/._-]*",
                marker2 in "[a-zA-Z0-9_-]+",
                suffix in "[a-zA-Z0-9/._-]*"
            )
        -> String {
            format!("{}§{}§{}§{}§{}", prefix, marker1, middle, marker2, suffix)
        }
    }

    prop_compose! {
        fn arb_attack_mode()
            (mode_type in 0..4u8)
        -> AttackMode {
            match mode_type {
                0 => AttackMode::Sniper,
                1 => AttackMode::BatteringRam,
                2 => AttackMode::Pitchfork,
                _ => AttackMode::ClusterBomb,
            }
        }
    }

    proptest! {
        /// **Feature: repeater-intruder, Property 5: Payload Generation Consistency**
        /// 
        /// For any payload configuration (wordlist, number range, or custom), the system should 
        /// generate payloads within specified bounds, validate file formats, and support 
        /// different attack modes with correct payload combinations.
        /// 
        /// **Validates: Requirements 4.1, 4.2, 4.3**
        #[test]
        fn property_payload_generation_consistency(
            config in arb_custom_payload_config()
        ) {
            tokio_test::block_on(async {
                // Test custom payload generator consistency
                let generator = CustomGenerator::from_config(&config).unwrap();
                
                // Validation should pass for valid configs
                prop_assert!(generator.validate().is_ok());
                
                // Count should match generated payload length
                let count = generator.count().await.unwrap();
                let payloads = generator.generate().await.unwrap();
                prop_assert_eq!(count, payloads.len());
                
                // Generated payloads should match config values
                if let PayloadConfig::Custom { values } = &config {
                    prop_assert_eq!(payloads, values.clone());
                }
                
                // Description should be meaningful
                let description = generator.description();
                prop_assert!(description.contains("Custom payloads"));
                prop_assert!(description.contains(&count.to_string()));
                
                Ok(())
            })?;
        }

        #[test]
        fn property_number_range_generation_bounds(
            config in arb_number_range_config()
        ) {
            tokio_test::block_on(async {
                let generator = NumberRangeGenerator::from_config(&config).unwrap();
                
                // Validation should pass for valid ranges
                prop_assert!(generator.validate().is_ok());
                
                let payloads = generator.generate().await.unwrap();
                let count = generator.count().await.unwrap();
                
                // Count should match generated payload length
                prop_assert_eq!(count, payloads.len());
                
                // All payloads should be within expected bounds
                if let PayloadConfig::NumberRange { start, end, step, format } = &config {
                    let expected_count = ((end - start) / step + 1) as usize;
                    prop_assert_eq!(payloads.len(), expected_count);
                    
                    // Check that payloads follow the sequence
                    for (i, payload) in payloads.iter().enumerate() {
                        let expected_value = start + (i as i64 * step);
                        
                        // Check format application
                        if format.contains("{}") {
                            let expected = format.replace("{}", &expected_value.to_string());
                            prop_assert_eq!(payload, &expected);
                        } else if format == "%d" {
                            prop_assert_eq!(payload, &expected_value.to_string());
                        }
                    }
                }
                
                Ok(())
            })?;
        }

        #[test]
        fn property_payload_position_parsing_consistency(
            template in arb_template_with_markers()
        ) {
            // Parse template
            let parsed_result = PayloadPositionParser::parse(&template);
            
            // If parsing succeeds, positions should be consistent
            if let Ok(parsed) = parsed_result {
                // Number of positions should match number of § pairs
                let marker_count = template.chars().filter(|&c| c == '§').count();
                prop_assert_eq!(parsed.positions.len() * 2, marker_count);
                
                // All positions should have valid indices
                for (i, position) in parsed.positions.iter().enumerate() {
                    prop_assert_eq!(position.index, i);
                    prop_assert!(position.start < position.end);
                    // Note: positions are in the original template, not processed template
                    prop_assert!(position.end <= template.len());
                }
                
                // Processed template should have placeholders
                for (i, _) in parsed.positions.iter().enumerate() {
                    let placeholder = format!("{{PAYLOAD_{}}}", i);
                    prop_assert!(parsed.processed_template.contains(&placeholder));
                }
                
                // Template utils should be consistent
                prop_assert!(TemplateUtils::has_payload_markers(&template));
                prop_assert_eq!(TemplateUtils::count_payload_positions(&template).unwrap(), parsed.positions.len());
            } else {
                // If parsing fails, it should be due to invalid syntax
                let error = parsed_result.unwrap_err();
                prop_assert!(error.to_string().contains("Unmatched") || 
                           error.to_string().contains("Empty") ||
                           error.to_string().contains("Invalid"));
            }
        }

        #[test]
        fn property_payload_injection_roundtrip(
            template in arb_template_with_markers(),
            payload_values in prop::collection::hash_map("[a-zA-Z0-9_-]+", "[a-zA-Z0-9_-]+", 0..10)
        ) {
            // Parse template
            if let Ok(parsed) = PayloadPositionParser::parse(&template) {
                // Create payload values for all required positions
                let mut complete_payload_values = HashMap::new();
                for position in &parsed.positions {
                    let value = payload_values.get(&position.payload_set_id)
                        .unwrap_or(&"default".to_string())
                        .clone();
                    complete_payload_values.insert(position.payload_set_id.clone(), value);
                }
                
                // Inject payloads
                if let Ok(injected) = PayloadPositionParser::inject_payloads(&parsed, &complete_payload_values) {
                    // Injected template should not contain placeholders
                    for i in 0..parsed.positions.len() {
                        let placeholder = format!("{{PAYLOAD_{}}}", i);
                        prop_assert!(!injected.contains(&placeholder));
                    }
                    
                    // Injected template should contain payload values
                    for (_, value) in &complete_payload_values {
                        if !value.is_empty() {
                            prop_assert!(injected.contains(value));
                        }
                    }
                }
            }
        }

        #[test]
        fn property_attack_mode_request_generation(
            mode in arb_attack_mode(),
            template in "GET /api/§param1§/§param2§ HTTP/1.1"
        ) {
            // Create test payload sets
            let mut payload_sets = HashMap::new();
            payload_sets.insert("param1".to_string(), vec!["a".to_string(), "b".to_string()]);
            payload_sets.insert("param2".to_string(), vec!["1".to_string(), "2".to_string(), "3".to_string()]);
            
            if let Ok(parsed) = PayloadPositionParser::parse(&template) {
                let executor = AttackModeFactory::create(&mode);
                
                // Count should be consistent with generation
                if let Ok(count) = executor.count_requests(&parsed, &payload_sets) {
                    if let Ok(requests) = executor.generate_requests(&parsed, &payload_sets) {
                        prop_assert_eq!(requests.len(), count);
                        
                        // All requests should have valid indices
                        for (i, request) in requests.iter().enumerate() {
                            prop_assert_eq!(request.index, i);
                            prop_assert!(!request.request.is_empty());
                            prop_assert!(!request.payload_values.is_empty());
                        }
                        
                        // Verify mode-specific behavior
                        match mode {
                            AttackMode::Sniper => {
                                // Should use only first payload set
                                prop_assert_eq!(count, 2); // param1 has 2 values
                            }
                            AttackMode::BatteringRam => {
                                // Should use first payload set for all positions
                                prop_assert_eq!(count, 2); // param1 has 2 values
                            }
                            AttackMode::Pitchfork => {
                                // Should use minimum length
                                prop_assert_eq!(count, 2); // min(2, 3) = 2
                            }
                            AttackMode::ClusterBomb => {
                                // Should use cartesian product
                                prop_assert_eq!(count, 6); // 2 * 3 = 6
                            }
                        }
                    }
                }
            }
        }

        #[test]
        fn property_payload_generator_factory_consistency(
            config in arb_payload_config()
        ) {
            // Factory should create appropriate generator types
            let generator_result = PayloadGeneratorFactory::create(&config);
            
            // For valid configs, factory should succeed
            match &config {
                PayloadConfig::Custom { values } if !values.is_empty() => {
                    prop_assert!(generator_result.is_ok());
                    let generator = generator_result.unwrap();
                    prop_assert!(generator.description().contains("Custom"));
                }
                PayloadConfig::NumberRange { start, end, step, .. } if *step != 0 && 
                    ((*step > 0 && start <= end) || (*step < 0 && start >= end)) => {
                    prop_assert!(generator_result.is_ok());
                    let generator = generator_result.unwrap();
                    prop_assert!(generator.description().contains("Number range"));
                }
                PayloadConfig::Wordlist { .. } => {
                    // Wordlist generator creation should succeed (validation happens later)
                    prop_assert!(generator_result.is_ok());
                    let generator = generator_result.unwrap();
                    prop_assert!(generator.description().contains("Wordlist"));
                }
                _ => {
                    // Invalid configs may fail
                }
            }
        }
    }
}