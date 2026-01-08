# Implementation Plan: Distributed MITM Proxy

## Overview

This implementation plan creates a Rust Cargo workspace for a distributed MITM proxy system with four main components: proxy-core library, proxy-agent binary, orchestrator service, and tauri desktop application.

## Tasks

- [x] 1. Create workspace root structure and configuration
  - Create root Cargo.toml with workspace configuration
  - Define shared dependencies in workspace.dependencies section
  - Set up consistent Rust edition across workspace
  - _Requirements: 1.1, 1.4, 6.1, 7.1_

- [x] 1.1 Write property test for workspace dependency consistency
  - **Property 1: Workspace Dependency Consistency**
  - **Validates: Requirements 6.2**

- [x] 2. Create proxy-core library crate
  - Create proxy-core directory and Cargo.toml
  - Configure as library crate with required dependencies
  - Add Hudsucker, Hyper, Tower, Tokio (full features), and Rcgen dependencies
  - Create src/lib.rs with basic module structure
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ]* 2.1 Write property test for dependency feature specification
  - **Property 2: Dependency Feature Specification**
  - **Validates: Requirements 6.4**

- [x] 3. Create proxy-agent binary crate
  - Create proxy-agent directory and Cargo.toml
  - Configure as binary crate depending on proxy-core
  - Add Tokio dependency for async execution
  - Create src/main.rs with basic structure
  - _Requirements: 3.1, 3.2, 3.3_

- [x] 4. Create orchestrator library crate
  - Create orchestrator directory and Cargo.toml
  - Configure as library crate with gRPC and database dependencies
  - Add Tonic, Prost, Sqlx, and Tokio dependencies
  - Create src/lib.rs with basic module structure
  - create detailed log settings and health status
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 5. Create tauri-app desktop application
  - Create tauri-app directory and Cargo.toml
  - Configure as Tauri application with orchestrator dependency
  - Add Tauri dependency with appropriate features
  - Add Tokio dependency for async operations
  - Create src/main.rs and basic Tauri configuration
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5.1 Write property test for crate entry point consistency
  - **Property 3: Crate Entry Point Consistency**
  - **Validates: Requirements 7.4**

- [x] 6. Verify workspace structure and build
  - Ensure all four crate directories exist with proper structure
  - Verify each crate has appropriate src/ directory and entry files
  - taauri-app folder name must be UI 
  - Test that workspace builds successfully with `cargo build`
  - _Requirements: 1.2, 1.3, 7.2, 7.4_

- [x] 6.1 Write unit tests for workspace configuration validation
  - Test root Cargo.toml contains all expected member crates
  - Test each crate type is configured correctly
  - Test all required dependencies are present
  - _Requirements: 1.1, 1.2, 2.1-2.5, 3.1-3.3, 4.1-4.5, 5.1-5.4_

- [ ] 7. Final checkpoint - Ensure workspace is complete and functional
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Create gRPC protocol definition
  - Create proto/ directory in workspace root
  - Create proto/proxy.proto with gRPC service definition
  - Define HttpHeaders, HttpRequestData, HttpResponseData messages
  - Define TrafficEvent with oneof for request/response
  - Define InterceptCommand placeholder message
  - Define ProxyService with StreamTraffic bidirectional streaming RPC
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 9. Configure protocol compilation in proxy-core
  - Add tonic-build dependency to proxy-core Cargo.toml
  - Create build.rs in proxy-core to compile proto files
  - Configure tonic-build to generate Rust code from proto/proxy.proto
  - Update proxy-core/src/lib.rs to expose generated proto module
  - _Requirements: 8.5, 8.6_

- [x] 10. Implement Certificate Authority management
  - [x] Create proxy-core/src/ca.rs module
  - [x] Implement CertificateAuthority struct with rcgen
  - [x] Add methods to load existing CA certificates from disk (ca.pem, ca.key)
  - [x] ITs also export .crt format
  - [x] Add method to generate new Root CA if certificates don't exist
  - [x] Add method to generate domain-specific certificates dynamically
  - [x] Implement persistent storage for CA certificates
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 10.1 Write property test for certificate generation
  - **Property 4: Certificate Domain Consistency**
  - Test that generated certificates match requested domains
  - **Validates: Requirements 9.4**

- [x] 11. Implement HTTP traffic handlers
  - [x] Create proxy-core/src/handlers.rs module
  - [x] Implement LogHandler struct that implements hudsucker::HttpHandler
  - [x] Implement handle_request method with proper body handling
  - [x] Implement handle_response method with proper body handling
  - [x] Add request ID generation using UUID for traffic tracking
  - [x] Add logging for intercepted requests and responses
  - [x] Ensure body streams are properly preserved and not consumed
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ]* 11.1 Write property test for request/response body preservation
  - **Property 5: Body Stream Preservation**
  - Test that request/response bodies are not corrupted during handling
  - **Validates: Requirements 10.4**

- [x] 12. Implement main proxy server
  - [x] Update proxy-core/src/lib.rs with ProxyServer struct
  - [x] Implement ProxyServer::new(port: u16) method
  - [x] Implement ProxyServer::run() method using hudsucker::Proxy
  - [x] Integrate CertificateAuthority for HTTPS interception
  - [x] Integrate LogHandler for traffic processing
  - [x] Add proper error handling and async support
  - _Requirements: 11.1, 11.2, 11.3_

- [x] 12.1 Write integration test for proxy server startup
  - [x] Test that proxy server starts successfully on specified port
  - [x] Test that CA certificates are properly loaded/generated
  - **Validates: Requirements 11.1, 11.2**

- [x] 13. Checkpoint - Verify proxy-core functionality
  - [x] Ensure all proxy-core tests pass
  - [x] Test that gRPC protocol compiles successfully
  - [x] Test that proxy server can start and handle basic traffic
  - [x] Ask the user if questions arise.
  - [x] Create a tests files on the test folder.
  - [x] Create a detailed logs for the proxy server.
  - [x] Create a health check endpoint for the proxy server.
  - [x] Create a metrics endpoint for the proxy server.
  - [x] Create a metrics endpoint for the orchestrator.
  - [x] Create a detailed comment lines on the code.

- [x] 14. Implement Proxy Agent (CLI)
  - [x] Update `proxy-agent` to be a headless binary
  - [x] Implement command line arguments logic (clap)
  - [x] Integrate `proxy-core` (LogHandler, CA, etc.)
  - [x] Implement gRPC client to stream logs to Orchestrator

- [x] 15. Enhanced Protocol Definitions (Phase 1)
  - [x] Update `proto/proxy.proto` structure <!-- id: 23 -->
  - [x] Add `RegisterAgent`, `TlsDetails`, `WebSocketFrame` messages <!-- id: 24 -->
  - [ ] Add `ScopeConfig` message (for dynamic filtering rules)
  - [x] Verify generated Rust types (`tonic_build`) <!-- id: 25 -->
  - [ ] **Tests**
    - [ ] Unit Test: Protocol Serialization/Deserialization

- [x] 16. Agent Core Enhancements (Phase 2)
  - [x] **Agent Lifecycle Manager**
    - [x] Implement `RegisterAgent` handshake on startup
    - [x] Implement registration logic,logs,metrics
    - [x] Implement robust exponential backoff reconnection logic
    - [x] **Tests**
      - [x] Unit Test: Backoff Logic
      - [x] Integration Test: Registration Flow
  - [x] **Traffic Filtering (Edge Filter)**
    - [x] Implement `ScopeMatcher` engine
    - [x] Add `wildcard_match` support (e.g., "*.google.com")
    - [x] Integrate scope check in Hudsucker interceptor (`!scope.is_allowed -> forward_directly`)
    - [x] **Tests**
      - [x] Unit Test: Scope Matching Logic
  - [x] **Intercept Controller (Pause/Resume)**
    - [x] Implement `InterceptController` struct
    - [x] Implement  logic,logs,metrics
    - [x] Use `DashMap<String, oneshot::Sender<InterceptDecision>>` for flow control
    - [x] Implement logic to pause request -> wait for decision -> resume/drop
    - [x] **Tests**
      - [x] Unit Test: Pause/Resume Logic

- [x] 17. Orchestrator Core Enhancements (Phase 3)
  - [x] **Session Manager**
    - [x] Implement `AgentRegistry` (DashMap)
    - [x] Implement methods: `register_agent`, `get_agent_tx`
    - [x] Implement logic,logs,metrics
    - [x] Handle Agent registration and control channel storage
    - [x] **Tests**
      - [x] Unit Test: Registry Operations
  - [x] **Stream Router**
    - [x] Implement `ProxyService` trait
    - [x] Logic: `register_agent` -> updates registry
    - [x] Logic: `stream_traffic` -> accepts stream, stores tx in registry
    - [x] Route incoming `ProxyEvent` -> Persistence -> Broadcast
    - [x] Route UI decisions -> `AgentRegistry` -> Specific Agent
    - [x] **Tests**
      - [x] Integration Test: gRPC Flow
  - [x] **Broadcast Layer**
   - [x] Implement  logic,err,logs
    - [x] Implement `tokio::broadcast` for UI/subscriber fan-out
 `tokio::broadcast::channel` for real-time UI updates
    - [ ] Connect to GraphQL Subscription system

- [x] 18. Persistence Layer (Phase 4)
  - [x] Setup persistence (SQLite/PostgreSQL)
  - [x] Create schemas: `proxy_events`, `tls_details`, `websocket_frames`
  - [x] Implement async write operations
  - [x] **Tests**
    - [x] Integration Test: Database Operations

- [x] 19. Repeater Engine (Phase 5)
 - [x] Implement  logic,logs,metrics
 - [x] **Tests**: Replay Integration Test
  - [ ] Implement `ExecuteRepeater` logic
  - [ ] Allow re-sending captured requests from Orchestrator/Agent
  - [ ] Capture and return responses with TLS details
  - [ ] **Tests**
    - [ ] Unit Test: Request Replay Logic

- [x] 20. Implement System Metrics Collection via gRPC
  - [-] **Update gRPC Protocol for System Metrics**
    - Extend `proto/proxy.proto` with `SystemMetricsEvent`, `SystemMetrics` messages
    - Add `StreamMetrics` RPC method for bidirectional metrics streaming
    - Add `MetricsCommand` and `MetricsConfig` for dynamic configuration
    - Regenerate Rust types with `tonic-build`
  - [ ] **Add sysinfo dependency to proxy-core**
    - Add `sysinfo = "0.30"` to proxy-core Cargo.toml
    - Add required tokio features for async system monitoring
  - [ ] **Create SystemMetricsCollector in proxy-core**
    - Create `proxy-core/src/system_metrics.rs`
    - Implement `SystemMetricsCollector` with `sysinfo` integration
    - Add gRPC streaming logic with configurable intervals (default: 5 seconds)
    - Handle `MetricsCommand` for dynamic configuration updates
  - [ ] **Integrate with proxy-agent**
    - Add metrics streaming to agent's gRPC client connection
    - Start metrics collection alongside traffic streaming
    - Handle reconnection logic for metrics stream
  - [ ] **Extend Orchestrator gRPC Service**
    - Implement `StreamMetrics` RPC method in orchestrator
    - Add database storage for system metrics
    - Integrate with broadcast layer for real-time UI updates
    - Add metrics configuration management
  - [ ] **Database Schema for System Metrics**
    - Create `system_metrics` table with proper indexing
    - Add migration for new schema
    - Implement async storage operations
  - [ ] **GraphQL Integration**
    - Extend GraphQL schema with system metrics types
    - Add queries for current and historical metrics
    - Add subscriptions for real-time metrics updates
  - [ ] **Tests**
    - [x] Unit Test: SystemMetricsCollector accuracy and bounds checking
    - [x] Integration Test: gRPC metrics streaming end-to-end
    - [x] Property Test: Metrics data consistency and streaming reliability
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6_

- [ ] 21. Implement Tauri App (GUI) & GraphQL API - REACT FRONTEND
  - [x] **Implement GraphQL Schema in `orchestrator`**
    - `Query`: getRequests(filter, limit, offset), getRequestById, etc.
    - `Subscription`: requestReceived (ProxyEvent stream)
  - [x] **Expose GraphQL Endpoint via Tauri**
    - Tauri `localhost` server or direct command invocation.
  - [x] **Frontend (React) Setup:**
    - Install `Apollo Client` or `TanStack Query`.
    - Create filtering components.
  - [ ] Ensure `UI` crate uses `proxy-core` for local proxying
  - [ ] **Add System Metrics Dashboard**
    - Create real-time charts for CPU, RAM, network usage
    - Display metrics for each connected agent
    - Add alerts for high resource usage
  


- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties
- Unit tests validate specific configuration and structure requirements
- The workspace structure follows Rust best practices for multi-crate projects