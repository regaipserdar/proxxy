# Protobuf Message Tag Coordination

This document coordinates protobuf message tags across all Proxxy modules to prevent conflicts and ensure proper protocol evolution.

## Tag Range Assignments

To prevent tag collisions, each module has reserved ranges:

| Module | Range | Message Types | Status |
|--------|-------|---------------|--------|
| **Core** | 1-9 | Basic proxy messages | ✅ Implemented |
| **Agent** | 10-19 | Agent lifecycle, execution | 📝 AGENT_TASKS.md |
| **Repeater/Intruder** | 20-29 | Attack commands | 📝 Repeater/Intruder spec |
| **LSR** | 30-39 | Recording commands | 📝 LSR_TASKS.md |
| **Nuclei** | 40-49 | Scan commands | 📝 NUCLEI_TASKS.md |
| **Future Modules** | 50+ | Reserved for expansion | 🔮 Future |

## Current Protocol Structure

### Core Messages (1-9)
```protobuf
message OrchestratorMessage {
  oneof message {
    // Core proxy functionality
    InterceptCommand intercept = 1;
    TrafficData traffic = 2;
    AgentStatus agent_status = 3;
    HealthCheck health_check = 4;
    // 5-9 reserved for core expansion
  }
}
```

### Agent Messages (10-19)
```protobuf
message OrchestratorMessage {
  oneof message {
    // Agent lifecycle and execution
    ExecuteRequest execute = 10;
    LifecycleCommand lifecycle = 11;
    AgentConfig agent_config = 12;
    // 13-19 reserved for agent expansion
  }
}
```

### Repeater/Intruder Messages (20-29)
```protobuf
message OrchestratorMessage {
  oneof message {
    // Attack commands
    AttackCommand attack = 20;
    RepeaterRequest repeater = 21;
    IntruderRequest intruder = 22;
    AttackStatus attack_status = 23;
    // 24-29 reserved for attack expansion
  }
}
```

### LSR Messages (30-39)
```protobuf
message OrchestratorMessage {
  oneof message {
    // Recording commands
    RecordingCommand recording = 30;
    RecordingStatus recording_status = 31;
    ProfileExecution profile_execution = 32;
    // 33-39 reserved for LSR expansion
  }
}
```

### Nuclei Messages (40-49)
```protobuf
message OrchestratorMessage {
  oneof message {
    // Scan commands
    ScanCommand scan = 40;
    ScanStatus scan_status = 41;
    ScanResult scan_result = 42;
    // 43-49 reserved for Nuclei expansion
  }
}
```

## Message Type Definitions

### Agent Messages (AGENT_TASKS.md)
```protobuf
message ExecuteRequest {
  string request_id = 1;
  HttpRequestData request = 2;
  string session_id = 3;
  map<string, string> session_headers = 4;
}

message LifecycleCommand {
  enum Action {
    RESTART = 0;
    SHUTDOWN = 1;
  }
  Action action = 1;
  bool force = 2;
}
```

### Attack Messages (Repeater/Intruder)
```protobuf
message AttackCommand {
  oneof command {
    RepeaterRequest repeater_request = 1;
    IntruderRequest intruder_request = 2;
    bool stop_attack = 3;
  }
}

message RepeaterRequest {
  string request_id = 1;
  HttpRequestData request = 2;
  string session_id = 3;
  map<string, string> session_headers = 4;
}

message IntruderRequest {
  string attack_id = 1;
  string request_id = 2;
  HttpRequestData request = 3;
  repeated string payload_values = 4;
  string session_id = 5;
  map<string, string> session_headers = 6;
}
```

### LSR Messages (LSR_TASKS.md)
```protobuf
message RecordingCommand {
  oneof command {
    StartRecording start = 1;
    StopRecording stop = 2;
    PauseRecording pause = 3;
  }
}

message StartRecording {
  string session_id = 1;
  string start_url = 2;
  string agent_id = 3;
}
```

### Nuclei Messages (NUCLEI_TASKS.md)
```protobuf
message ScanCommand {
  oneof command {
    StartScan start = 1;
    StopScan stop = 2;
    GetScanStatus status = 3;
  }
}

message StartScan {
  string scan_id = 1;
  repeated string targets = 2;
  string session_id = 3;
  ScanConfig config = 4;
}
```

## Rules

1. **Tag Uniqueness**: Each tag number can only be used once in the entire protocol
2. **Range Respect**: Stay within your assigned range
3. **Sequential Assignment**: Use tags sequentially within your range
4. **Backward Compatibility**: Never reuse or change existing tags
5. **Documentation**: Update this file when adding new messages

## Coordination Process

Before adding new messages:

1. **Check Range**: Verify you have available tags in your range
2. **Update This File**: Document your new message types
3. **Generate Code**: Run protobuf compilation
4. **Test**: Verify no conflicts with other modules
5. **Commit**: Include both .proto and this documentation

## Conflict Resolution

If tag conflicts occur:

1. **Immediate Stop**: Don't deploy conflicting changes
2. **Identify Conflict**: Check which modules are conflicting
3. **Reassign Tags**: Move one module to available tags
4. **Update Documentation**: Reflect changes in this file
5. **Regenerate**: Recompile all affected protobuf code

## Future Expansion

When ranges are exhausted:

1. **Request New Range**: Update this file with new assignments
2. **Coordinate**: Ensure no conflicts with existing modules
3. **Document**: Update all relevant task files
4. **Migrate**: Update existing code to use new ranges

## Validation

To validate protocol consistency:

```bash
# Check for tag conflicts
grep -r "= [0-9]" proto/*.proto | sort -t= -k2 -n

# Verify range compliance
./scripts/validate_proto_tags.sh
```

---

*Last Updated: 2024-01-12*
*Next Review: Before each new message type addition*