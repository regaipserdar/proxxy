# Implementation Plan: Repeater and Intruder Modules

## Overview

This implementation plan transforms the Repeater and Intruder design into a series of incremental coding tasks that build upon the existing Proxxy distributed MITM proxy architecture. The implementation follows a modular approach, creating new crates for attack functionality while integrating seamlessly with existing components.

## Tasks

- [x] 1. Set up core attack engine infrastructure (Coordinated with Foundation)
  - **CRITICAL: Use foundation files created in root directory**
  - Create `attack-engine` crate in workspace
  - Define core data models and traits for attack execution using `proxy-common::Session`
  - Set up basic error handling and result types
  - Configure dependencies for async execution and serialization
  - Integrate with global resource manager for concurrency control
  - _Requirements: 8.1, 8.2, 8.5_
  - _Dependencies: proxy-common/src/session.rs, orchestrator/src/resource_manager.rs_

- [x] 1.1 Write property test for attack engine core types
  - **Property 1: Request Processing Integrity**
  - **Validates: Requirements 1.1, 1.2, 1.4, 1.5**

- [x] 2. Implement database schema extensions (Coordinated with LSR_TASKS.md)
  - [x] 2.1 Create migration for repeater tables
    - **CRITICAL: Coordinate migration order with LSR_TASKS.md Phase 2.2**
    - Add `repeater_tabs` table with tab configuration storage
    - Add `repeater_history` table for execution history
    - Include proper foreign key relationships and indexes
    - Ensure migration number doesn't conflict with LSR migrations (005_add_login_profiles.sql)
    - _Requirements: 7.1, 7.2_
    - _Dependencies: LSR_TASKS.md Phase 2.2 (Database Schema Extensions)_

  - [x] 2.2 Create migration for intruder tables
    - **CRITICAL: Coordinate with LSR and ensure sequential migration numbers**
    - Add `intruder_attacks` table for attack configurations
    - Add `intruder_results` table for attack results
    - Add `payload_sets` table for reusable payload configurations
    - Include proper indexes for performance
    - Use migration number after LSR migrations (e.g., 006_add_intruder_tables.sql)
    - _Requirements: 7.3, 7.4_
    - _Dependencies: LSR_TASKS.md Phase 2.2_

  - [x] 2.3 Implement database access methods
    - Add CRUD operations for repeater tabs and history
    - Add CRUD operations for intruder attacks and results
    - **CRITICAL: Implement batch insert for intruder results (1000 records per transaction)**
    - Add payload set management methods
    - Include proper error handling and transactions
    - Implement buffered result writing with periodic flush (every 5 seconds OR 1000 records)
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 8.1, 8.4_

    - [x] 2.4 Write property test for database operations
  - **Property 8: Data Persistence Consistency**
  - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

- [x] 3. Create repeater module implementation
  - [x] 3.1 Implement RepeaterManager struct
    - Create tab management functionality
    - Implement request editing and validation
    - Add agent selection and validation logic
    - Include session data integration
    - _Requirements: 1.1, 1.2, 1.6, 2.1, 2.5_

  - [x] 3.2 Implement repeater execution logic
    - Create request execution through selected agents
    - Add response capture and processing
    - Implement history storage and retrieval
    - Include timing and metadata collection
    - _Requirements: 1.4, 1.5, 2.3_

  - [x] 3.3 Add agent failure handling for repeater
    - Implement agent availability validation
    - Add graceful error handling for offline agents
    - Include proper error messaging and user notification
    - _Requirements: 2.2, 2.4, 9.2, 9.3_

- [x] 3.4 Write property test for repeater functionality
  - **Property 2: Agent Selection and Routing**
  - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

- [x] 4. Implement payload generation system
  - [x] 4.1 Create PayloadGenerator trait and implementations
    - Implement WordlistGenerator for file-based payloads
    - Implement NumberRangeGenerator for numeric sequences
    - Implement CustomGenerator for user-defined payloads
    - Add proper validation and error handling
    - _Requirements: 4.1, 4.2_

  - [x] 4.2 Implement payload position parsing
    - Create parser for §marker§ syntax in request templates
    - Add validation for payload position syntax
    - Implement position highlighting and validation
    - Include error reporting for invalid syntax
    - _Requirements: 3.2_

  - [x] 4.3 Implement attack mode logic
    - Create Sniper mode (single position, all payloads)
    - Create BatteringRam mode (multiple positions, same payload)
    - Create Pitchfork mode (multiple positions, parallel iteration)
    - Create ClusterBomb mode (multiple positions, all combinations)
    - _Requirements: 4.3_

- [x] 4.4 Write property test for payload generation
  - **Property 5: Payload Generation Consistency**
  - **Validates: Requirements 4.1, 4.2, 4.3**

- [x] 5. Create intruder attack engine
  - [x] 5.1 Implement IntruderManager struct
    - Create attack configuration management
    - Implement payload set configuration
    - Add agent selection for distributed attacks
    - Include attack template creation and validation
    - _Requirements: 3.1, 3.3, 3.4_

  - [x] 5.2 Implement payload distribution algorithms
    - Create RoundRobin distribution strategy
    - Create Batch distribution strategy
    - Add load balancing across agents
    - Include agent failure detection and redistribution
    - _Requirements: 3.5, 4.4, 4.5_

  - [x] 5.3 Implement attack execution coordination
    - Create concurrent request execution across agents
    - Add progress tracking and statistics
    - Implement graceful attack termination
    - Include result collection and aggregation
    - _Requirements: 5.1, 5.3, 8.1_

- [x] 5.4 Write property test for attack distribution
  - **Property 4: Payload Distribution Algorithms**
  - **Validates: Requirements 3.5, 4.4, 4.5**

- [x] 6. Implement session integration (Coordinated with LSR_TASKS.md)
  - [x] 6.1 Create SessionData integration
    - **CRITICAL: Use common session interface from LSR (proxy-common::Session)**
    - Implement session header and cookie injection compatible with LSR format
    - Add session data application to requests using LSR session structure
    - Create session selection and switching logic that works with LSR profiles
    - Include session expiration handling compatible with LSR lifecycle
    - _Requirements: 6.1, 6.2, 6.3, 6.5_
    - _Dependencies: LSR_TASKS.md Phase 6.4 (Session Cookie Management)_

  - [x] 6.2 Add session authentication handling
    - **CRITICAL: Coordinate with LSR session validation**
    - Implement authentication failure detection using LSR session indicators
    - Add session refresh options that trigger LSR profile re-execution
    - Create fallback handling for expired sessions via LSR integration
    - Include proper error reporting and user options
    - _Requirements: 6.4_
    - _Dependencies: LSR_TASKS.md Phase 6.4, NUCLEI_TASKS.md Phase 4.2_

- [x] 6.3 Write property test for session integration
  - **Property 7: Session Integration Completeness**
  - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

- [x] 7. Extend gRPC protocol and agent infrastructure
  - [x] 7.1 Update proto definitions (Coordinated with proto/TAGS.md)
    - **CRITICAL: Use reserved tag range 20-29 from proto/TAGS.md**
    - **CRITICAL: Coordinate with AGENT_TASKS.md Protocol Updates**
    - Add RepeaterRequest and IntruderRequest messages using tags 21-22
    - Add AttackCommand message type using tag 20
    - Extend OrchestratorMessage with attack commands (align with ExecuteRequest structure)
    - Add LifecycleCommand for agent management (from AGENT_TASKS.md)
    - Update service definitions as needed
    - _Requirements: 10.5_
    - _Dependencies: AGENT_TASKS.md Phase 1.1, proto/TAGS.md_

  - [x] 7.2 Implement unified HTTP execution engine in agents
    - **CRITICAL: Build on AGENT_TASKS.md HTTP Engine (Phase 2)**
    - Extend existing ExecuteRequest handler to support RepeaterRequest and IntruderRequest
    - Implement session data injection for attack requests
    - Add response capture and streaming back to orchestrator
    - Integrate with global HTTP client from AGENT_TASKS.md
    - Include proper error handling and reporting
    - Support graceful shutdown during attack execution
    - _Requirements: 10.2_
    - _Dependencies: AGENT_TASKS.md Phase 2.1, 2.2_

- [x] 7.3 Add agent lifecycle management integration
  - **NEW: Integrate with AGENT_TASKS.md Lifecycle Management**
  - Implement graceful attack termination during agent restart/shutdown
  - Handle agent lifecycle events during distributed attacks
  - Add attack state preservation during agent restarts
  - Implement attack redistribution when agents go offline
  - _Requirements: 2.4, 4.5, 8.2_
  - _Dependencies: AGENT_TASKS.md Phase 3_

- [x] 7.4 Write integration test for gRPC attack commands
  - Test end-to-end attack command flow
  - Verify request execution and response capture
  - Test error handling and recovery scenarios
  - _Requirements: 10.2, 10.5_

- [x] 8. Checkpoint - Core functionality validation
  - Ensure all core attack engine components work together
  - Verify database operations and data persistence
  - do not Test basic repeater and intruder functionality 
  - Ask the user if questions arise

- [x] 9. Implement real-time monitoring and results
  - [x] 9.1 Create result streaming infrastructure
    - Implement real-time result broadcasting
    - Add progress tracking and statistics calculation
    - Create result highlighting based on configurable criteria
    - Include export functionality for results
    - _Requirements: 5.1, 5.2, 5.4, 5.5_

  - [x] 9.2 Add performance monitoring
    - Implement concurrency limiting per agent
    - Add backpressure mechanisms for high load
    - Create dynamic load balancing adjustments
    - Include memory management and cleanup
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 9.3 Write property test for real-time monitoring
  - **Property 6: Real-time Monitoring Accuracy**
  - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

- [ ] 9.4 Write property test for performance management
  - **Property 9: Performance and Concurrency Management**
  - **Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5**

- [x] 10. Extend GraphQL API for repeater and intruder
  - [x] 10.1 Add repeater GraphQL types and resolvers
    - Create RepeaterTab, RepeaterExecution GraphQL types
    - Add queries for repeater tabs and history
    - Add mutations for tab creation, editing, and execution
    - Include subscriptions for real-time updates
    - _Requirements: 1.1, 1.4, 1.5, 1.6_

  - [x] 10.2 Add intruder GraphQL types and resolvers
    - Create IntruderAttack, IntruderResult GraphQL types
    - Add queries for attack configurations and results
    - Add mutations for attack creation, start, and stop
    - Include subscriptions for real-time attack progress
    - _Requirements: 3.1, 5.1, 5.2_

  - [x] 10.3 Add session integration to GraphQL
    - Create session selection queries
    - Add session application mutations
    - Include session status and expiration handling
    - _Requirements: 6.1, 6.2, 6.5_

- [x] 10.4 Write integration test for GraphQL API
  - Test complete GraphQL workflows for repeater and intruder
  - Verify real-time subscriptions and updates
  - Test error handling and validation
  - _Requirements: 10.1, 10.3, 10.4_

- [x] 11. Implement security and error handling
  - [x] 11.1 Add sensitive data masking
    - make sure also disable functionality on the masking functions 
    - Implement authentication data masking in logs
    - Add sensitive value masking in UI displays
    - Create secure handling of session data
    - Include proper data sanitization
    - _Requirements: 9.1_

  - [x] 11.2 Implement comprehensive error handling
    - Add detailed error messages with remediation suggestions
    - Implement quick failure detection for agents
    - Add input validation with clear error messages
    - Create graceful degradation under resource exhaustion
    - _Requirements: 9.2, 9.3, 9.4, 9.5_

- [x] 11.3 Write property test for security and error handling
  - **Property 10: Security and Error Handling**
  - **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**

- [ ] 12. Create GUI components for repeater
  - [ ] 12.1 Implement RepeaterTab component
    - Create request editing interface with syntax highlighting
    - Add agent selection dropdown with status indicators
    - Implement response display with search functionality
    - Include history panel with request-response pairs
    - _Requirements: 1.2, 2.1, 2.5_

  - [ ] 12.2 Add repeater integration to traffic interface
    - Add "Send to Repeater" context menu option
    - Implement tab creation from traffic history
    - Add proper data transfer and initialization
    - _Requirements: 1.1, 10.1_

- [ ] 12.3 Write unit tests for repeater GUI components
  - Test component rendering and user interactions
  - Verify data flow and state management
  - Test error handling and validation
  - _Requirements: 10.4_

- [ ] 13. Create GUI components for intruder
  - [ ] 13.1 Implement IntruderAttack component
    - Create attack configuration interface
    - Add payload position marking and validation
    - Implement payload set configuration
    - Add agent selection for distributed attacks
    - _Requirements: 3.2, 3.3, 3.4_

  - [ ] 13.2 Implement attack results interface
    - Create real-time results table with filtering
    - Add progress indicators and statistics
    - Implement result highlighting and export
    - Include attack control buttons (start/stop)
    - _Requirements: 5.1, 5.2, 5.4, 5.5_

  - [ ] 13.3 Add intruder integration to traffic interface
    - Add "Send to Intruder" context menu option
    - Implement attack configuration from traffic history
    - Add proper template creation and initialization
    - _Requirements: 3.1, 10.1_

- [ ] 13.4 Write unit tests for intruder GUI components
  - Test component rendering and user interactions
  - Verify real-time updates and progress tracking
  - Test attack control and error handling
  - _Requirements: 10.4_

- [ ] 14. Implement data restoration and persistence
  - [ ] 14.1 Add application state restoration
    - Implement repeater tab restoration on startup
    - Add historical attack data access
    - Create configuration persistence across sessions
    - Include proper data migration and versioning
    - _Requirements: 7.5_

  - [ ] 14.2 Add export and import functionality
    - Implement attack template export/import
    - Add result export in multiple formats (JSON, CSV, XML)
    - Create configuration backup and restore
    - Include proper validation and error handling
    - _Requirements: 5.5_

- [ ] 14.3 Write integration test for data persistence
  - Test application restart and data restoration
  - Verify export/import functionality
  - Test data migration and versioning
  - _Requirements: 7.5_

- [ ] 15. Final integration and architecture compliance
  - [ ] 15.1 Verify architecture integration
    - Ensure traffic routing through existing agent infrastructure
    - Verify database schema compliance with existing patterns
    - Test GUI component integration with existing design system
    - Validate gRPC protocol extensions
    - _Requirements: 10.2, 10.3, 10.4, 10.5_

  - [ ] 15.2 Add comprehensive logging and monitoring
    - Implement structured logging for all attack operations
    - Add metrics collection for performance monitoring
    - Create health checks for attack engine components
    - Include proper error tracking and alerting
    - _Requirements: 8.3, 9.3_

- [ ] 15.3 Write property test for architecture integration
  - **Property 11: Architecture Integration Compliance**
  - **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**

- [ ] 16. Final checkpoint - Complete system validation
  - Ensure all tests pass and system works end-to-end
  - Verify all requirements are implemented and tested
  - Test complete workflows from GUI to agent execution
  - Validate performance under load and error conditions
  - Ask the user if questions arise

## Notes

- **CRITICAL FOUNDATION DEPENDENCIES**: 
  - ALL development must use foundation files: `proxy-common/src/session.rs`, `MIGRATIONS.md`, `proto/TAGS.md`, `orchestrator/src/resource_manager.rs`
  - Tasks 7.1, 7.2, and 7.3 must be coordinated with AGENT_TASKS.md development
  - Task 6.1-6.2 must use `proxy-common::Session` interface (NEVER create custom session types)
  - Task 2.1-2.2 must use migration numbers 010-014 from MIGRATIONS.md
  - Task 7.1 must use protobuf tags 20-29 from proto/TAGS.md
- **Protocol Alignment**: gRPC protocol changes must be synchronized between Orchestrator and Agent development
- **Session Integration**: Must use `proxy-common::Session` interface compatible with LSR and Nuclei
- **Database Coordination**: Migration files must be numbered sequentially across all modules
- **Resource Management**: All concurrency must go through global ResourceManager
- **HTTP Engine Reuse**: Leverage the unified HTTP execution engine from AGENT_TASKS.md Phase 2
- **Lifecycle Integration**: Attack engine must handle agent lifecycle events (restart/shutdown) gracefully
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation and user feedback
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The implementation builds incrementally, with each task depending on previous ones
- All sensitive data handling follows security best practices
- The architecture integrates seamlessly with existing Proxxy components