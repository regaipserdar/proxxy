# Traffic Policy System

## Overview

The Proxxy traffic policy system separates **static configuration** (startup settings) from **dynamic policy** (runtime rules). This allows operators to modify traffic handling rules on-the-fly via the UI without restarting the proxy.

## Architecture

### 1. Static Configuration (`ProxyStartupConfig`)
Set once at startup, never changes:
- Listen address/port
- Orchestrator endpoint
- Admin API port
- Certificate paths

### 2. Dynamic Policy (`TrafficPolicy`)
Updated at runtime via gRPC from Orchestrator UI:
- **Scope**: Which domains to intercept
- **Interception Rules**: Block, Drop, Pause, Delay, Inject
- **Match & Replace**: Automatic content modification

## Components

### Scope Configuration

```rust
use proxy_core::{ScopeConfig, OutOfScopeAction};

let scope = ScopeConfig {
    include: vec!["*.target.com".to_string(), "api.example.com".to_string()],
    exclude: vec!["*.google-analytics.com".to_string()],
    out_of_scope_action: OutOfScopeAction::Pass,
};
```

**Out-of-Scope Actions:**
- `LogOnly`: Save to DB but don't show in UI
- `Drop`: Kill connection immediately (save bandwidth)
- `Pass`: Forward without processing

### Interception Rules

```rust
use proxy_core::{InterceptionRule, RuleCondition, RuleAction};

let rule = InterceptionRule {
    id: "block-admin".to_string(),
    name: "Block Admin Panel".to_string(),
    enabled: true,
    conditions: vec![
        RuleCondition::UrlContains("/admin".to_string()),
        RuleCondition::Method("POST".to_string()),
    ],
    action: RuleAction::Block {
        reason: "Admin access not allowed".to_string(),
    },
};
```

**Available Conditions:**
- `UrlContains(String)`: Simple substring match
- `UrlRegex(String)`: Regex pattern matching
- `Method(String)`: HTTP method (GET, POST, etc.)
- `HasHeader(String)`: Check if header exists
- `HeaderValueMatch { key, regex }`: Match header value
- `BodyRegex(String)`: Search in request body
- `Port(u16)`: Match specific port

**Available Actions:**
- `Pause`: Stop request and wait for UI approval (Intercept mode)
- `Block { reason }`: Return 403 Forbidden
- `Drop`: Send TCP RST (silent kill)
- `Delay(u64)`: Delay request by N milliseconds
- `InjectHeader { key, value }`: Add/modify headers
- `ModifyBody { find, replace }`: Replace body content

### Match & Replace Rules

```rust
use proxy_core::{MatchReplaceRule, MatchLocation};

let rule = MatchReplaceRule {
    enabled: true,
    match_regex: r"Authorization: Bearer (.+)".to_string(),
    replace_string: "Authorization: Bearer REDACTED".to_string(),
    location: MatchLocation::RequestHeader,
};
```

## Usage Example

### Agent Startup

```rust
use proxy_core::{ProxyStartupConfig, TrafficPolicy, CertificateAuthority};
use std::sync::{Arc, RwLock};

// 1. Static config (from CLI args or config file)
let config = ProxyStartupConfig {
    listen_address: "127.0.0.1".to_string(),
    listen_port: 8080,
    orchestrator_endpoint: "http://127.0.0.1:50051".to_string(),
    admin_port: 9091,
    certificate_config: Default::default(),
};

// 2. Dynamic policy (starts with defaults, updated via gRPC)
let policy = Arc::new(RwLock::new(TrafficPolicy::default()));

// 3. Start proxy with both
let ca = CertificateAuthority::from_pem(&ca_cert, &ca_key)?;
let proxy = ProxyServer::new(config, ca, policy.clone())?;
```

### Runtime Policy Update (from Orchestrator)

```rust
// Orchestrator sends new policy via gRPC
let new_policy = TrafficPolicy {
    scope: ScopeConfig {
        include: vec!["*.target.com".to_string()],
        exclude: vec![],
        out_of_scope_action: OutOfScopeAction::Drop,
    },
    interception_rules: vec![
        InterceptionRule {
            id: "pause-login".to_string(),
            name: "Intercept Login Requests".to_string(),
            enabled: true,
            conditions: vec![
                RuleCondition::UrlContains("/login".to_string()),
                RuleCondition::Method("POST".to_string()),
            ],
            action: RuleAction::Pause,
        },
    ],
    match_replace_rules: vec![],
};

// Agent receives and applies
*policy.write().unwrap() = new_policy;
```

### Request Handler Integration

```rust
async fn handle_request(req: Request, policy: Arc<RwLock<TrafficPolicy>>) -> Result<Response> {
    let policy = policy.read().unwrap();
    
    // 1. Scope check
    if !policy.scope.is_allowed(&req.url) {
        match policy.scope.out_of_scope_action {
            OutOfScopeAction::Drop => return Err(Error::DropConnection),
            OutOfScopeAction::LogOnly => {
                // Save to DB but don't process
                return forward_passthrough(req).await;
            }
            OutOfScopeAction::Pass => {
                return forward_passthrough(req).await;
            }
        }
    }
    
    // 2. Rule evaluation
    let req_context = RequestContext {
        url: req.url.clone(),
        method: req.method.clone(),
        headers: req.headers.clone(),
        body: req.body.clone(),
        port: 443,
    };
    
    for rule in &policy.interception_rules {
        if rule.matches(&req_context) {
            match &rule.action {
                RuleAction::Pause => {
                    // Send to UI and wait for decision
                    let decision = wait_for_ui_decision(&req.id).await?;
                    // Process decision...
                }
                RuleAction::Block { reason } => {
                    return Ok(Response::forbidden(reason));
                }
                RuleAction::Drop => {
                    return Err(Error::DropConnection);
                }
                RuleAction::Delay(ms) => {
                    tokio::time::sleep(Duration::from_millis(*ms)).await;
                }
                RuleAction::InjectHeader { key, value } => {
                    req.headers.insert(key.clone(), value.clone());
                }
                RuleAction::ModifyBody { find, replace } => {
                    // Apply body modification...
                }
            }
        }
    }
    
    // 3. Forward request
    forward_request(req).await
}
```

## Key Differences: Drop vs Block

| Action | Behavior | Use Case |
|--------|----------|----------|
| **Block** | Returns HTTP 403 Forbidden | Polite rejection, user sees error page |
| **Drop** | Sends TCP RST, kills socket | Firewall testing, stealth mode |

## Testing

```bash
# Run policy tests
cargo test -p proxy-core --lib policy

# Check specific test
cargo test -p proxy-core test_scope_config_wildcard
```

## Protocol Integration

The `TrafficPolicy` is serialized to JSON and sent via gRPC:

```protobuf
message UpdatePolicyRequest {
  string agent_id = 1;
  string policy_json = 2; // Serialized TrafficPolicy
}
```

Orchestrator UI → gRPC → Agent → `Arc<RwLock<TrafficPolicy>>`
