# Implementation Plan: Response Body Capture Fix

## Overview

This implementation plan addresses the critical issue where HTTP response bodies are not being captured by the Proxxy proxy-core. The solution involves modifying the `handle_response` method in `proxy-core/src/handlers.rs` to properly read, capture, and reconstruct HTTP response bodies while maintaining proxy functionality and adding timeout handling.

## Tasks

- [x] 1. Create body capture configuration and error types
  - Add `BodyCaptureConfig` struct with timeout settings
  - Add `BodyCaptureError` enum with timeout error variants
  - Add configuration validation logic
  - _Requirements: 7.1, 7.2, 7.4, 7.5, 8.5_

- [ ]* 1.1 Write property test for configuration validation
  - **Property 8: Configuration compliance**
  - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

- [x] 2. Implement response body reading with timeout handling
  - [x] 2.1 Create `read_response_body` function with timeout support
    - Implement stream reading with configurable timeouts
    - Add size limit enforcement with truncation
    - Handle chunked transfer encoding properly
    - _Requirements: 1.1, 1.4, 2.1, 2.3, 8.1, 8.2_

  - [ ]* 2.2 Write property test for complete body capture
    - **Property 2: Complete body capture**
    - **Validates: Requirements 1.1, 1.2**

  - [ ]* 2.3 Write property test for size limit enforcement
    - **Property 3: Size limit enforcement**
    - **Validates: Requirements 1.4, 3.1**

  - [ ]* 2.4 Write property test for stream consumption completeness
    - **Property 4: Stream consumption completeness**
    - **Validates: Requirements 2.1**

  - [ ]* 2.5 Write property test for timeout handling
    - **Property 13: Timeout handling**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.4**

- [x] 3. Implement response reconstruction logic
  - [x] 3.1 Create `capture_and_reconstruct_response` function
    - Read response body while preserving headers and status
    - Reconstruct identical response for client forwarding
    - Handle binary and compressed data correctly
    - _Requirements: 1.3, 2.2, 2.4, 4.1, 4.2, 4.3_

  - [ ]* 3.2 Write property test for round-trip integrity
    - **Property 1: Response body round-trip integrity**
    - **Validates: Requirements 1.3, 2.2**

  - [ ]* 3.3 Write property test for binary data integrity
    - **Property 5: Binary data integrity**
    - **Validates: Requirements 3.3, 4.2**

  - [ ]* 3.4 Write property test for text and encoding preservation
    - **Property 6: Text and encoding preservation**
    - **Validates: Requirements 4.1, 4.3, 4.4**

- [x] 4. Update LogHandler to use body capture
  - [x] 4.1 Modify LogHandler struct to include BodyCaptureConfig
    - Add configuration fields to LogHandler
    - Update constructor to accept configuration
    - _Requirements: 7.1, 7.2, 7.4, 7.5_

  - [x] 4.2 Replace hardcoded empty body in handle_response
    - Remove `body: vec![]` hardcoding
    - Integrate body capture and reconstruction logic
    - Add error handling for capture failures
    - _Requirements: 1.1, 1.2, 1.5, 5.1, 5.2_

  - [ ]* 4.3 Write property test for error isolation
    - **Property 7: Error isolation**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

- [x] 5. Add memory management and concurrent processing
  - [x] 5.1 Implement memory usage tracking
    - Track total memory used for concurrent body captures
    - Implement memory limit enforcement
    - Add backpressure mechanisms for high load
    - _Requirements: 3.2, 3.5, 6.4_

  - [ ]* 5.2 Write property test for memory limit enforcement
    - **Property 9: Memory limit enforcement**
    - **Validates: Requirements 3.2, 7.5**

- [x] 6. Add content-type filtering and selective capture
  - [x] 6.1 Implement content-type based filtering
    - Add content-type header parsing
    - Implement whitelist/blacklist filtering logic
    - Skip body capture for filtered content types
    - _Requirements: 7.3_

  - [x]* 6.2 Write unit tests for content-type filtering
    - Test various content-type scenarios
    - Test filter configuration edge cases
    - _Requirements: 7.3_

- [x] 7. Handle special cases and edge conditions
  - [x] 7.1 Implement HEAD request handling
    - Detect HEAD requests and handle gracefully
    - Ensure no body capture for HEAD responses
    - _Requirements: 2.5_

  - [x] 7.2 Implement chunked encoding support
    - Properly decode chunked transfer encoding
    - Capture complete decoded body
    - _Requirements: 2.3_

  - [ ]* 7.3 Write property test for chunked encoding handling
    - **Property 10: Chunked encoding handling**
    - **Validates: Requirements 2.3**

  - [x] 7.4 Implement compressed response handling
    - Capture raw compressed data without decompression
    - Preserve compression for client
    - _Requirements: 2.4_

  - [ ]* 7.5 Write property test for compressed data preservation
    - **Property 11: Compressed data preservation**
    - **Validates: Requirements 2.4**

- [x] 8. Integration and configuration updates
  - [x] 8.1 Update proxy-agent to pass configuration to LogHandler
    - Add configuration loading from environment or config file
    - Pass BodyCaptureConfig to LogHandler constructor
    - Add default configuration values
    - _Requirements: 7.1, 7.2, 7.4, 7.5, 8.5_

  - [x] 8.2 Add configuration validation and error handling
    - Validate timeout values are positive
    - Validate size limits are reasonable
    - Provide helpful error messages for invalid config
    - _Requirements: 8.5_

- [x] 9. Performance optimization and testing
  - [x] 9.1 Add performance monitoring
    - Measure latency impact of body capture
    - Add metrics for capture success/failure rates
    - Monitor memory usage during high load
    - _Requirements: 6.1, 6.2_

  - [ ]* 9.2 Write property test for performance impact when disabled
    - **Property 12: Performance impact when disabled**
    - **Validates: Requirements 6.5**

  - [ ]* 9.3 Write integration tests for end-to-end functionality
    - Test full proxy request/response cycles with body capture
    - Verify database storage and GraphQL API access
    - Test various content types and sizes
    - _Requirements: 1.1, 1.2, 4.5_

- [x] 10. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties
- Integration tests verify end-to-end functionality
- The implementation prioritizes data integrity and proxy stability
- Timeout handling ensures the proxy doesn't hang on slow responses