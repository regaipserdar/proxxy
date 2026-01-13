# Design Document

## Overview

This design implements CLI project support for the Proxxy Orchestrator by extending the existing command-line argument parsing and integrating project operations into the startup sequence. The solution leverages existing `Database` methods for project creation and loading while ensuring proper configuration state management.

## Architecture

The CLI project support will be implemented through three main components:

1. **CLI Argument Extension**: Extend the existing `clap` argument structure to accept project parameters
2. **Startup Integration**: Modify the `Orchestrator::start()` method to perform project operations before server initialization
3. **Configuration Loading**: Ensure ScopeConfig and InterceptionConfig are properly loaded and available to server components

## Components and Interfaces

### CLI Arguments Structure

The existing `Args` struct in `main.rs` will be extended with an optional project field:

```rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // ... existing fields ...
    
    /// Project name to auto-load on startup
    #[arg(long, short = 'p')]
    project: Option<String>,
}
```

### Project Startup Handler

A new component will handle CLI project operations:

```rust
pub struct ProjectStartupHandler {
    db: Arc<Database>,
}

impl ProjectStartupHandler {
    pub async fn handle_cli_project(
        &self, 
        project_name: &str,
        scope_state: Arc<RwLock<ScopeConfig>>,
        interception_state: Arc<RwLock<InterceptionConfig>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Project creation and loading logic
    }
}
```

### Configuration State Management

The existing configuration loading pattern from GraphQL mutations will be reused:

```rust
// Load configurations from database
let scope_config = db.get_scope_config().await?;
let interception_config = db.get_interception_config().await?;

// Update in-memory state
*scope_state.write().await = scope_config;
*interception_state.write().await = interception_config;
```

## Data Models

### Project Validation

The existing project name validation will be reused:
- Alphanumeric characters, hyphens, and underscores only
- Implemented in `Database::create_project()` method

### Configuration Models

Existing models will be used without modification:
- `ScopeConfig`: Defines URL patterns for traffic recording scope
- `InterceptionConfig`: Defines rules for traffic interception

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing all acceptance criteria, several properties can be consolidated to eliminate redundancy:

- Properties 1.4 and 2.5 both test project name validation and can be combined into a single comprehensive validation property
- Properties 2.4, 3.4, 6.2, and 6.3 all test logging behavior and can be consolidated into logging properties
- Properties 3.5, 4.1, 4.2, 4.4, and 4.5 all relate to configuration state management and can be combined
- Properties 5.1, 5.2, and 5.3 all test startup sequence timing and can be consolidated

### Core Properties

Property 1: CLI argument parsing
*For any* valid project name provided via `--project` or `-p` arguments, the orchestrator should parse and accept the project name parameter
**Validates: Requirements 1.1, 1.2**

Property 2: Default behavior preservation  
*For any* orchestrator startup without CLI project arguments, the system should start normally with no active project loaded and default configurations
**Validates: Requirements 1.3, 7.1, 7.4**

Property 3: Project name validation
*For any* project name containing invalid characters (non-alphanumeric, non-hyphen, non-underscore), the orchestrator should reject the name and exit with an error message
**Validates: Requirements 1.4, 2.5**

Property 4: Automatic project creation
*For any* valid project name specified via CLI that does not exist, the orchestrator should create the project automatically before loading
**Validates: Requirements 2.1**

Property 5: Error handling and graceful exit
*For any* project operation failure (creation or loading), the orchestrator should log detailed error messages and exit before starting servers
**Validates: Requirements 2.3, 3.3, 5.4**

Property 6: Startup sequence timing
*For any* CLI project specification, project operations (creation and loading) should complete after database initialization but before HTTP/gRPC server startup
**Validates: Requirements 3.1, 5.1, 5.2, 5.3**

Property 7: Configuration state consistency
*For any* auto-loaded project, the in-memory ScopeConfig and InterceptionConfig should match the database-stored configurations and be available to all server components
**Validates: Requirements 3.5, 4.1, 4.2, 4.4, 4.5**

Property 8: Configuration fallback behavior
*For any* configuration loading failure, the orchestrator should use default configurations and log appropriate warning messages
**Validates: Requirements 4.3**

Property 9: Comprehensive logging
*For any* CLI project operation, appropriate info messages should be logged indicating the operation type, project name, and paths involved
**Validates: Requirements 2.4, 3.4, 6.1, 6.2, 6.3, 6.5**

Property 10: Error logging detail
*For any* CLI project operation error, detailed error messages should be logged with sufficient information for troubleshooting
**Validates: Requirements 6.4**

Property 11: Backward compatibility preservation
*For any* existing CLI arguments or GraphQL operations, functionality should remain unchanged when no CLI project is specified
**Validates: Requirements 7.2, 7.3, 7.5**

Property 12: Startup logging preservation
*For any* orchestrator startup, all existing startup logging and initialization steps should be maintained regardless of CLI project usage
**Validates: Requirements 5.5**

<function_calls>
<invoke name="prework">
<parameter name="featureName">cli-project-support

## Error Handling

### Project Operation Failures

The system will handle various failure scenarios gracefully:

1. **Invalid Project Names**: Validation occurs before any operations, with clear error messages
2. **Project Creation Failures**: File system errors, permission issues, or disk space problems
3. **Project Loading Failures**: Database connection issues, corrupted databases, or missing files
4. **Configuration Loading Failures**: Fallback to default configurations with warning logs

### Error Recovery Strategy

- **Early Exit**: Project operation failures prevent server startup to avoid inconsistent state
- **Detailed Logging**: All errors include context information for troubleshooting
- **Graceful Degradation**: Configuration loading failures use defaults rather than failing completely

## Testing Strategy

### Dual Testing Approach

The implementation will use both unit tests and property-based tests for comprehensive coverage:

**Unit Tests** will verify:
- Specific CLI argument parsing scenarios
- Error message content and formatting
- Integration with existing GraphQL functionality
- Startup sequence logging output

**Property-Based Tests** will verify:
- Universal properties across all valid project names
- Error handling behavior across all invalid inputs
- Configuration state consistency across all loading scenarios
- Backward compatibility across all existing argument combinations

### Property-Based Testing Configuration

- **Testing Framework**: `proptest` crate (already in dependencies)
- **Test Iterations**: Minimum 100 iterations per property test
- **Test Tagging**: Each property test will reference its design document property
- **Tag Format**: `Feature: cli-project-support, Property {number}: {property_text}`

### Test Coverage Areas

1. **CLI Argument Parsing**: Verify all valid and invalid project name formats
2. **Project Operations**: Test creation and loading across various scenarios
3. **Configuration Management**: Verify state consistency and fallback behavior
4. **Error Handling**: Test all failure modes and recovery mechanisms
5. **Backward Compatibility**: Ensure existing functionality remains intact
6. **Startup Sequence**: Verify proper timing and order of operations

The property-based tests will generate random project names, simulate various failure conditions, and verify that the system maintains correctness properties across all scenarios.