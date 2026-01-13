# Requirements Document

## Introduction

This specification defines the requirements for adding CLI project support to the Proxxy Orchestrator. Currently, projects must be created and loaded via GraphQL mutations after the server starts. This feature will enable automated testing and headless startup by allowing project specification directly from the command line when launching the binary.

## Glossary

- **Orchestrator**: The central management server for the distributed MITM proxy system
- **Project**: A workspace containing configuration, database, and traffic data for a specific testing session
- **CLI**: Command Line Interface for the orchestrator binary
- **Database**: SQLite database containing project data, stored in `{project_name}.proxxy/proxxy.db`
- **ScopeConfig**: Configuration defining which URLs are in scope for traffic recording
- **InterceptionConfig**: Configuration for traffic interception settings
- **Headless_Mode**: Running the orchestrator without requiring manual GraphQL operations

## Requirements

### Requirement 1: CLI Argument Support

**User Story:** As a developer, I want to specify a project name via command line arguments, so that I can automate orchestrator startup for testing and CI/CD pipelines.

#### Acceptance Criteria

1. WHEN the orchestrator binary is launched with `--project <NAME>` argument, THE Orchestrator SHALL accept the project name parameter
2. WHEN the orchestrator binary is launched with `-p <NAME>` argument, THE Orchestrator SHALL accept the project name parameter as a short form
3. WHEN no project argument is provided, THE Orchestrator SHALL start normally without auto-loading any project
4. WHEN an invalid project name is provided, THE Orchestrator SHALL validate the name using existing validation rules (alphanumeric, -, _)
5. THE Orchestrator SHALL display help information showing the project argument when `--help` is used

### Requirement 2: Automatic Project Creation

**User Story:** As a developer, I want projects to be created automatically if they don't exist, so that I can start testing immediately without manual setup.

#### Acceptance Criteria

1. WHEN a project name is specified via CLI and the project does not exist, THE Orchestrator SHALL create the project automatically
2. WHEN creating a project automatically, THE Orchestrator SHALL use the existing `Database.create_project` method
3. WHEN project creation fails, THE Orchestrator SHALL log the error and exit gracefully
4. WHEN a project is created automatically, THE Orchestrator SHALL log an info message indicating the creation
5. THE Orchestrator SHALL validate project names before creation using existing validation rules

### Requirement 3: Automatic Project Loading

**User Story:** As a developer, I want specified projects to be loaded automatically during startup, so that the orchestrator is ready for use without manual GraphQL operations.

#### Acceptance Criteria

1. WHEN a project name is specified via CLI, THE Orchestrator SHALL load the project before starting HTTP/gRPC servers
2. WHEN loading a project automatically, THE Orchestrator SHALL use the existing `Database.load_project` method
3. WHEN project loading fails, THE Orchestrator SHALL log the error and exit gracefully
4. WHEN a project is loaded automatically, THE Orchestrator SHALL log an info message with project details
5. THE Orchestrator SHALL ensure ScopeConfig and InterceptionConfig are loaded into application state

### Requirement 4: Configuration State Management

**User Story:** As a system administrator, I want project configurations to be properly initialized, so that scope and interception settings work correctly from startup.

#### Acceptance Criteria

1. WHEN a project is auto-loaded, THE Orchestrator SHALL load ScopeConfig from the project database
2. WHEN a project is auto-loaded, THE Orchestrator SHALL load InterceptionConfig from the project database  
3. WHEN configuration loading fails, THE Orchestrator SHALL use default configurations and log a warning
4. WHEN configurations are loaded, THE Orchestrator SHALL update the in-memory state before server startup
5. THE Orchestrator SHALL ensure configuration state is available to all server components

### Requirement 5: Startup Sequence Integration

**User Story:** As a developer, I want CLI project loading to integrate seamlessly with existing startup, so that all orchestrator features work correctly.

#### Acceptance Criteria

1. WHEN CLI project loading is enabled, THE Orchestrator SHALL perform project operations after database initialization
2. WHEN CLI project loading is enabled, THE Orchestrator SHALL perform project operations before HTTP/gRPC server startup
3. WHEN project operations complete successfully, THE Orchestrator SHALL continue with normal server startup
4. WHEN project operations fail, THE Orchestrator SHALL exit before starting servers
5. THE Orchestrator SHALL maintain all existing startup logging and initialization steps

### Requirement 6: Error Handling and Logging

**User Story:** As a developer, I want clear error messages and logging, so that I can troubleshoot CLI project issues effectively.

#### Acceptance Criteria

1. WHEN CLI project operations begin, THE Orchestrator SHALL log info messages indicating the auto-loading process
2. WHEN project creation occurs, THE Orchestrator SHALL log the project name and path
3. WHEN project loading occurs, THE Orchestrator SHALL log the project name and database path
4. WHEN errors occur during CLI project operations, THE Orchestrator SHALL log detailed error messages
5. WHEN CLI project operations complete, THE Orchestrator SHALL log success confirmation before server startup

### Requirement 7: Backward Compatibility

**User Story:** As an existing user, I want the orchestrator to work exactly as before when no CLI project is specified, so that my current workflows are not disrupted.

#### Acceptance Criteria

1. WHEN no `--project` argument is provided, THE Orchestrator SHALL start with no active project loaded
2. WHEN no CLI project is specified, THE Orchestrator SHALL maintain existing GraphQL project management functionality
3. WHEN existing command line arguments are used, THE Orchestrator SHALL continue to work as before
4. WHEN the orchestrator starts without a CLI project, THE Orchestrator SHALL use default ScopeConfig and InterceptionConfig
5. THE Orchestrator SHALL maintain all existing API endpoints and GraphQL mutations for project management