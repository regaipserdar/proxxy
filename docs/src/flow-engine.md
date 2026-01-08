# Flow Engine & Login Sequence Recorder

## 🎯 Vision

Proxxy v1.1+ introduces **Flow Engine** and **Login Sequence Recorder (LSR)** - a revolutionary automation and replay system built on Rust's **Zero-Cost Abstractions** and **Memory Safety** principles.

### Performance Goals
- **10x less RAM** than Python/Node.js competitors (Puppeteer, Selenium)
- **Ultra-low latency** through zero-copy parsing
- **Concurrent execution** of thousands of flows
- **Self-healing selectors** for resilient automation

---

## 🧱 1. Flow Engine (Automation & Replay Core)

The Flow Engine executes recorded scenarios in two modes:
- **Visual Mode**: Browser-based execution (Chrome DevTools Protocol)
- **Protocol Mode**: HTTP-only execution (no browser overhead)

### Technology Stack

| Component | Crate | Why This Choice? |
|-----------|-------|------------------|
| **Browser Control** | `chromiumoxide` | Rust equivalent of Puppeteer. Direct Chrome DevTools Protocol (CDP) communication via WebSocket. Much lighter than WebDriver, event-driven architecture. |
| **Async Runtime** | `tokio` | Rust's standardized async engine. Multi-thread work-stealing scheduler enables concurrent management of thousands of flows. |
| **HTTP Client (Protocol Mode)** | `hyper` + `tower` | Raw packet-level replay capability. Unlike `reqwest` (opinionated with built-in cookie store), `hyper` gives us full control. `tower` provides retry and timeout middleware. |
| **Data Extraction** | `jsonpath-rust` + `scraper` | JSONPath for API responses, `scraper` (CSS selector engine) for HTML parsing. |
| **Template Engine** | `tera` | Runtime variable substitution (`{{username}}`, `{{csrf_token}}`). Jinja2-like, fast and secure. |
| **Scripting (Optional)** | `rhai` | Embedded scripting for complex logic (e.g., "if captcha appears, wait"). Rust-integrated, safe scripting language. |

### 🚀 Performance Architecture

**Zero-Copy Parsing**: Instead of converting HTTP response body to String (copying), we parse directly from `Bytes` (using `bytes` crate) by reference.

```rust
use bytes::Bytes;

// ❌ Bad: Copies data
let body_string = String::from_utf8(response_body)?;
let json: Value = serde_json::from_str(&body_string)?;

// ✅ Good: Zero-copy
let body_bytes = Bytes::from(response_body);
let json: Value = serde_json::from_slice(&body_bytes)?;
```

---

## 🎥 2. Login Sequence Recorder (LSR)

LSR monitors user actions, analyzes them, and generates **resilient selectors** that survive DOM changes.

### Technology Stack

| Component | Crate | Why This Choice? |
|-----------|-------|------------------|
| **Event Stream** | `tokio-stream` | Process hundreds of events (mouse move, click, keypress) as an async stream. |
| **Selector Generation** | `scraper` + Custom Logic | Calculate shortest, unique CSS/XPath for clicked elements using HTML parsing engine. |
| **Event Correlation** | `uuid` + `std::time` | Match DOM events with Network requests by timestamp. |
| **Keylogging Prevention** | `secrecy` | Keep password field data encrypted in RAM, prevent accidental logging. |
| **State Storage** | `sled` | **Critical**: Lock-free, embedded key-value store. Much faster than SQLite for recording. Eliminates I/O bottleneck. |

---

## 🧬 Common Library (Integration & Data Structures)

### A. Data Serialization

**Formats:**
- **JSON**: User-readable storage
- **Bincode**: Internal Agent ↔ Orchestrator communication

**Crates:** `serde`, `serde_json`, `bincode`

**Why:** `serde` is Rust's standard. `bincode` is much faster and smaller than JSON, ideal for binary communication.

### B. Smart Selector Algorithm

LSR's core innovation: **Self-Healing Selectors**

```rust
pub struct ElementSelector {
    pub css_id: Option<String>,           // #submit
    pub css_class: Option<String>,        // .btn.btn-primary
    pub xpath: String,                    // //div[@id='app']/button[1]
    pub text_content: Option<String>,     // "Giriş Yap"
    pub attributes: HashMap<String, String>, // name="submit", data-testid="login-button"
}
```

**Self-Healing Logic:**
1. Try `css_id` (#submit)
2. If not found, try `xpath`
3. If not found, try `text_content` ("Giriş Yap" button)
4. If not found, try `attributes` (data-testid="login-button")

This is done using `scraper` crate for real-time HTML analysis during replay.

---

## 📦 Implementation Roadmap

### Phase 1: Foundation (v1.2)
- [ ] Add `chromiumoxide` dependency
- [ ] Implement basic CDP connection
- [ ] Create Flow data structures
- [ ] Add `sled` for state storage

### Phase 2: LSR Core (v1.3)
- [ ] Event stream processing
- [ ] Smart selector generation
- [ ] Event correlation (DOM ↔ Network)
- [ ] Password field encryption

### Phase 3: Flow Engine (v1.4)
- [ ] Visual mode execution (CDP)
- [ ] Protocol mode execution (HTTP-only)
- [ ] Template engine integration
- [ ] Self-healing selector logic

### Phase 4: Advanced Features (v1.5)
- [ ] Scripting support (`rhai`)
- [ ] Flow debugging tools
- [ ] Performance profiling
- [ ] Parallel flow execution

---

## 🔧 Dependencies to Add

```toml
[dependencies]
# Browser Control
chromiumoxide = "0.5"

# Data Storage
sled = "0.34"

# Parsing & Extraction
scraper = "0.18"
jsonpath-rust = "0.3"

# Template Engine
tera = "1.19"

# Scripting (Optional)
rhai = "1.16"

# Security
secrecy = "0.8"

# Zero-Copy
bytes = "1.5"
```

---

## 🎯 Performance Benchmarks (Target)

| Metric | Puppeteer (Node.js) | Selenium (Python) | Proxxy (Rust) |
|--------|---------------------|-------------------|---------------|
| **Memory per Flow** | ~100 MB | ~150 MB | **~10 MB** |
| **Startup Time** | ~2s | ~3s | **~100ms** |
| **Concurrent Flows** | ~10 | ~5 | **~1000** |
| **Selector Lookup** | ~5ms | ~10ms | **~0.5ms** |

---

## 🚀 Example: Recording a Login Flow

```rust
use proxxy_lsr::{Recorder, Event};

#[tokio::main]
async fn main() {
    let mut recorder = Recorder::new("https://example.com/login").await?;
    
    // Start recording
    recorder.start().await?;
    
    // User performs actions (clicks, types)
    // LSR automatically captures:
    // - DOM events
    // - Network requests
    // - Smart selectors
    
    // Stop and save
    let flow = recorder.stop().await?;
    flow.save("login_flow.json")?;
}
```

---

## 🎬 Example: Replaying a Flow

```rust
use proxxy_flow::{FlowEngine, ExecutionMode};

#[tokio::main]
async fn main() {
    let engine = FlowEngine::new(ExecutionMode::Visual).await?;
    
    // Load flow
    let flow = Flow::load("login_flow.json")?;
    
    // Execute with variables
    let result = engine.execute(flow, hashmap! {
        "username" => "test@example.com",
        "password" => "secret123"
    }).await?;
    
    println!("Login successful: {}", result.success);
}
```

---

## 🔐 Security Considerations

1. **Password Encryption**: All password fields use `secrecy` crate
2. **No Plaintext Logs**: Sensitive data never appears in logs
3. **Secure Storage**: `sled` database encrypted at rest
4. **Memory Safety**: Rust's ownership prevents data leaks

---

## 📊 Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Proxxy Flow System                    │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐         ┌──────────────┐              │
│  │     LSR      │────────▶│  Flow Engine │              │
│  │  (Recorder)  │         │  (Executor)  │              │
│  └──────────────┘         └──────────────┘              │
│         │                        │                       │
│         │                        │                       │
│         ▼                        ▼                       │
│  ┌──────────────┐         ┌──────────────┐              │
│  │     Sled     │         │ chromiumoxide│              │
│  │  (Storage)   │         │   (Browser)  │              │
│  └──────────────┘         └──────────────┘              │
│                                  │                       │
│                                  ▼                       │
│                           ┌──────────────┐              │
│                           │    Chrome    │              │
│                           │     CDP      │              │
│                           └──────────────┘              │
└─────────────────────────────────────────────────────────┘
```

---

**Version:** 1.1.0 (Planned for v1.2+)  
**Status:** 📋 Design Phase  
**Target:** 10x Performance Improvement over Competitors
