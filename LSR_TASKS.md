# 📝 LSR_TASKS.md - Login Sequence Recorder (LSR) Implementation

**Context:** Single Source of Truth for the `Proxxy` LSR module.
**Scope:** `flow-engine` library, Workspace integration, Advanced Replay, and Developer Experience.
**Strict Mode:** Adhere to constraints. No circular dependencies.

---

## 🛠️ Phase 1: Infrastructure & Workspace Setup

### 1.1 Core Library Creation
- [ ] **Create flow-engine library structure**
  - `cargo new --lib flow-engine`
  - Add to workspace members in root `Cargo.toml`
  - Set up basic module structure (`src/lib.rs`, `src/lsr/mod.rs`)
  - **Priority:** HIGH | **Status:** PENDING

### 1.2 Dependency Management
- [ ] **Configure flow-engine/Cargo.toml**
  - **Browser Automation:** `chromiumoxide` (latest stable)
  - **Async Runtime:** `tokio` (workspace version)
  - **Serialization:** `serde`, `serde_json` (workspace versions)
  - **Error Handling:** `thiserror`, `anyhow` (workspace versions)
  - **Security:** `secrecy` (for sensitive data)
  - **Utilities:** `uuid`, `url`, `base64`, `regex` (workspace versions where available)
  - **HAR Processing:** Research and add appropriate HAR parsing crate
  - **Internal Dependencies:** Path references to `orchestrator` and `proxy-core`
  - **Priority:** HIGH | **Status:** PENDING

### 1.3 Workspace Integration
- [ ] **Update workspace dependency management**
  - Ensure version consistency across crates
  - Add flow-engine to relevant integration tests
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🗄️ Phase 2: Data Models & Database Extensions

### 2.1 Core Data Structures (`src/lsr/model.rs`)
- [ ] **LoginProfile struct**
  ```rust
  pub struct LoginProfile {
      pub id: uuid::Uuid,
      pub name: String,
      pub start_url: String,
      pub steps: Vec<LoginStep>,
      pub meta: ProfileMeta,
      pub created_at: chrono::DateTime<chrono::Utc>,
      pub updated_at: chrono::DateTime<chrono::Utc>,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **LoginStep enum**
  ```rust
  pub enum LoginStep {
      Navigate { url: String, wait_for: Option<String> },
      Click { selector: SmartSelector, wait_for: Option<String> },
      Type { selector: SmartSelector, value: SecretString, is_masked: bool },
      Wait { duration_ms: u64, condition: Option<WaitCondition> },
      CheckSession { validation_url: String, success_indicators: Vec<String> },
      CustomAction { action_type: String, parameters: serde_json::Value }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **SmartSelector struct**
  ```rust
  pub struct SmartSelector {
      pub value: String,
      pub selector_type: SelectorType,
      pub priority: u8,
      pub alternatives: Vec<String>,
      pub validation_result: Option<ValidationResult>,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 2.2 Database Schema Extensions
- [ ] **Create migration file `migrations/005_add_login_profiles.sql`**
  ```sql
  CREATE TABLE IF NOT EXISTS login_profiles (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      start_url TEXT NOT NULL,
      steps TEXT NOT NULL, -- JSON array
      meta TEXT, -- JSON metadata
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      agent_id TEXT, -- Which agent recorded this
      status TEXT DEFAULT 'active' -- active, archived, failed
  );
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Create profile_executions table**
  ```sql
  CREATE TABLE IF NOT EXISTS profile_executions (
      id TEXT PRIMARY KEY,
      profile_id TEXT NOT NULL,
      agent_id TEXT NOT NULL,
      started_at INTEGER NOT NULL,
      completed_at INTEGER,
      status TEXT NOT NULL, -- running, success, failed
      error_message TEXT,
      steps_completed INTEGER DEFAULT 0,
      total_steps INTEGER NOT NULL,
      session_cookies TEXT, -- JSON cookies for debugging
      FOREIGN KEY(profile_id) REFERENCES login_profiles(id)
  );
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 2.3 HAR Integration (`src/lsr/har.rs`)
- [ ] **HAR Import Function**
  ```rust
  pub async fn from_har(path: &str) -> Result<LoginProfile, HarError>
  ```
  - Parse HAR file structure
  - Analyze HTTP requests for login patterns
  - Extract form data and navigation sequences
  - Generate LoginStep variants from HAR entries
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **HAR Export Function**
  ```rust
  pub fn to_har(profile: &LoginProfile) -> Result<HarExport, HarError>
  ```
  - Convert LoginProfile to HAR format
  - Simulate HTTP requests for debugging
  - Include timing and metadata information
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🌐 Phase 3: Browser Automation Infrastructure

### 3.1 Browser Management (`src/lsr/browser.rs`)
- [ ] **Browser Launcher**
  ```rust
  pub async fn launch_browser(proxy_config: Option<ProxyConfig>) -> Result<Browser, BrowserError>
  ```
  - Configure Chromium with proxy settings
  - Handle SSL certificate errors
  - Set up appropriate browser arguments
  - Manage browser lifecycle
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Proxy Integration**
  - Configure browser to use Proxxy agent as proxy
  - Handle certificate trust issues
  - Ensure proper traffic routing through Proxxy
  - **Priority:** HIGH | **Status:** PENDING

### 3.2 Page Management (`src/lsr/page.rs`)
- [ ] **Page Controller**
  ```rust
  pub struct PageController {
      browser: Browser,
      current_page: Option<Page>,
  }
  ```
  - Navigate to URLs with proper waiting
  - Handle page events and errors
  - Manage page state and context
  - **Priority:** HIGH | **Status:** PENDING

### 3.3 JavaScript Injection (`src/lsr/injection.rs`)
- [ ] **Event Capture Scripts**
  - Develop JavaScript for DOM event interception
  - Capture click events with element information
  - Monitor form submissions and keyboard input
  - Handle dynamic content and SPA navigation
  - **Priority:** HIGH | **Status:** PENDING

---

## 🧠 Phase 4: Smart Selector Generation System

### 4.1 Selector Analysis (`src/lsr/analyzer.rs`)
- [ ] **Node Analysis Algorithm**
  ```rust
  pub fn calculate_best_selector(node: &cdp::browser_protocol::dom::Node) -> SmartSelector
  ```
  - **Priority 1:** Stable IDs (`data-testid`, `id`, `name`)
  - **Priority 2:** Test attributes (`data-cy`, `aria-label`)
  - **Priority 3:** Semantic text content (XPath with text)
  - **Priority 4:** Structural CSS selectors
  - **Priority 5:** Full DOM path (last resort)
  - **Priority:** HIGH | **Status:** PENDING

### 4.2 Selector Validation
- [ ] **Live Testing**
  ```rust
  pub async fn validate_selector(page: &Page, selector: &str) -> ValidationResult
  ```
  - Test selector uniqueness in live page
  - Check for element visibility and interactability
  - Generate alternative selectors if validation fails
  - **Priority:** HIGH | **Status:** PENDING

### 4.3 Selector Blacklist System
- [ ] **Pattern Recognition**
  - Identify and exclude utility classes (Tailwind, Bootstrap)
  - Filter out hashed CSS classes
  - Avoid dynamic IDs with numeric patterns
  - Prevent excessively long selector chains
  - **Priority:** HIGH | **Status:** PENDING

### 4.4 Self-Healing Mechanisms
- [ ] **Alternative Selector Generation**
  - Generate multiple selector strategies
  - Rank by reliability and performance
  - Implement fallback chain for execution
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🎥 Phase 5: Recording Engine

### 5.1 Event Capture System (`src/lsr/recorder.rs`)
- [ ] **DOM Event Listeners**
  - Capture click events with target element details
  - Monitor keyboard input and form submissions
  - Track navigation and page transitions
  - Handle AJAX and SPA route changes
  - **Priority:** HIGH | **Status:** PENDING

### 5.2 Recording State Management
- [ ] **Recording Session**
  ```rust
  pub struct RecordingSession {
      pub id: uuid::Uuid,
      pub browser: Browser,
      pub steps: Vec<LoginStep>,
      pub start_url: String,
      pub status: RecordingStatus,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 5.3 Orchestrator Integration
- [ ] **Status Broadcasting**
  - Notify orchestrator of recording state changes
  - Stream recording progress via gRPC
  - Handle recording interruption and resumption
  - **Priority:** MEDIUM | **Status:** PENDING

---

## ▶️ Phase 6: Replay Engine

### 6.1 Execution Engine (`src/lsr/replayer.rs`)
- [ ] **Profile Executor**
  ```rust
  pub async fn execute_profile(
      profile: &LoginProfile,
      browser: &Browser,
      config: &ExecutionConfig
  ) -> Result<ExecutionResult, ReplayError>
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 6.2 Step Execution Logic
- [ ] **Individual Step Handlers**
  - Navigate steps with proper waiting
  - Click actions with element verification
  - Type actions with sensitive data masking
  - Wait steps with condition checking
  - Session validation with success indicators
  - **Priority:** HIGH | **Status:** PENDING

### 6.3 Self-Healing During Replay
- [ ] **Error Recovery**
  - Attempt alternative selectors on element not found
  - Handle timing issues with adaptive waiting
  - Recover from network errors and timeouts
  - Implement retry logic with exponential backoff
  - **Priority:** MEDIUM | **Status:** PENDING

### 6.4 Session Cookie Management
- [ ] **Cookie Extraction & Injection**
  ```rust
  pub async fn extract_session_cookies(page: &Page) -> Result<Vec<Cookie>, CookieError>
  pub async fn inject_cookies_to_orchestrator(cookies: Vec<Cookie>) -> Result<(), CookieError>
  ```
  - Extract cookies after successful login
  - Format cookies for orchestrator consumption
  - Inject cookies into agent's cookie jar
  - Validate session establishment
  - **Priority:** HIGH | **Status:** PENDING

---

## 🧪 Phase 7: Comprehensive Testing Strategy

### 7.1 Unit Tests
- [ ] **Model Tests** (`tests/unit/models.rs`)
  - LoginProfile serialization/deserialization
  - LoginStep variant testing
  - SmartSelector generation validation
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Analyzer Tests** (`tests/unit/analyzer.rs`)
  - Selector generation algorithms
  - Priority ranking validation
  - Blacklist pattern matching
  - **Priority:** MEDIUM | **Status:** PENDING

### 7.2 Integration Tests
- [ ] **Browser Automation Tests** (`tests/integration/browser.rs`)
  - Browser launch and configuration
  - Proxy integration testing
  - Page navigation and interaction
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Recording Tests** (`tests/integration/recording.rs`)
  - End-to-end recording workflow
  - Event capture accuracy
  - HAR import/export functionality
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Replay Tests** (`tests/integration/replay.rs`)
  - Profile execution accuracy
  - Self-healing mechanism testing
  - Cookie management validation
  - **Priority:** MEDIUM | **Status:** PENDING

### 7.3 Mock Server Infrastructure
- [ ] **Test Server Setup** (`tests/mock_server.rs`)
  - Use `wiremock` for login form simulation
  - Create various authentication scenarios
  - Test different login flow patterns
  - **Priority:** MEDIUM | **Status:** PENDING

### 7.4 Performance Tests
- [ ] **Load Testing** (`tests/performance/`)
  - Concurrent recording sessions
  - Memory usage profiling
  - Browser resource management
  - **Priority:** LOW | **Status:** PENDING

---

## 📚 Phase 8: Documentation & Developer Experience

### 8.1 API Documentation
- [ ] **Comprehensive Code Docs**
  - Document all public APIs with examples
  - Include error handling guidance
  - Provide usage patterns and best practices
  - **Priority:** MEDIUM | **Status:** PENDING

### 8.2 Architecture Documentation
- [ ] **Design Documents**
  - Create architecture diagrams
  - Document component interactions
  - Explain selector generation algorithm
  - **Priority:** MEDIUM | **Status:** PENDING

### 8.3 User Guides
- [ ] **Getting Started Guide**
  - Step-by-step recording tutorial
  - HAR import/export instructions
  - Troubleshooting common issues
  - **Priority:** LOW | **Status:** PENDING

### 8.4 Integration Documentation
- [ ] **Developer Integration Guide**
  - How to integrate with existing Proxxy setup
  - Configuration options and tuning
  - Extension points and customization
  - **Priority:** LOW | **Status:** PENDING

---

## 🔒 Phase 9: Security & Production Hardening

### 9.1 Security Implementation
- [ ] **Sensitive Data Protection**
  - Use `secrecy` crate for password masking
  - Implement secure in-memory storage
  - Ensure no sensitive data in logs
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Browser Security**
  - Sandboxed browser execution
  - Restricted file system access
  - Secure proxy configuration
  - **Priority:** HIGH | **Status:** PENDING

### 9.2 Error Handling & Resilience
- [ ] **Comprehensive Error Types**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum FlowEngineError {
      #[error("Browser launch failed: {0}")]
      BrowserLaunch(String),
      #[error("Selector generation failed: {0}")]
      SelectorGeneration(String),
      #[error("Recording error: {0}")]
      Recording(String),
      // ... other error variants
  }
  ```
  - **Priority:** MEDIUM | **Status:** PENDING

### 9.3 Resource Management
- [ ] **Browser Lifecycle Management**
  - Proper cleanup on errors
  - Resource leak prevention
  - Concurrent session limits
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🚀 Phase 10: Production Deployment & Monitoring

### 10.1 Configuration Management
- [ ] **Production Configuration**
  - Environment-specific settings
  - Browser executable paths
  - Resource limits and timeouts
  - **Priority:** MEDIUM | **Status:** PENDING

### 10.2 Monitoring & Observability
- [ ] **Metrics Integration**
  - Browser performance metrics
  - Recording success/failure rates
  - Resource usage tracking
  - **Priority:** LOW | **Status:** PENDING

### 10.3 Logging Strategy
- [ ] **Structured Logging**
  - Integration with existing tracing infrastructure
  - Debug logging for troubleshooting
  - Audit logging for security events
  - **Priority:** LOW | **Status:** PENDING

---

## 🔄 Phase 11: GUI Integration

### 11.1 Backend API Extensions
- [ ] **GraphQL Schema Extensions**
  - Add login profile queries and mutations
  - Recording status subscriptions
  - Profile execution tracking
  - **Priority:** MEDIUM | **Status:** PENDING

### 11.2 REST API Endpoints
- [ ] **Flow Engine API**
  - `/api/flows/profiles` - CRUD operations
  - `/api/flows/record` - Recording control
  - `/api/flows/replay` - Profile execution
  - **Priority:** MEDIUM | **Status:** PENDING

### 11.3 Frontend Integration
- [ ] **UI Components**
  - Profile management interface
  - Recording control panel
  - Execution status dashboard
  - **Priority:** LOW | **Status:** PENDING

---

## 📊 Phase 12: Performance Optimization & Scalability

### 12.1 Browser Pool Management
- [ ] **Resource Optimization**
  - Browser instance pooling
  - Memory usage optimization
  - Startup time reduction
  - **Priority:** LOW | **Status:** PENDING

### 12.2 Selector Caching
- [ ] **Performance Enhancements**
  - Selector validation caching
  - DOM analysis optimization
  - Parallel processing capabilities
  - **Priority:** LOW | **Status:** PENDING

---

## ✅ Acceptance Criteria

Each phase should be considered complete when:

1. **All unit tests pass** with >90% code coverage
2. **Integration tests validate** end-to-end functionality
3. **Documentation is comprehensive** and up-to-date
4. **Security review is completed** for sensitive components
5. **Performance benchmarks meet** production requirements
6. **GUI integration is functional** and user-tested

---

## 🎯 Success Metrics

- **Recording Accuracy:** >95% successful replay rate
- **Performance:** <10s browser startup, <2s step execution
- **Resource Usage:** <500MB RAM per recording session
- **Reliability:** >99% successful completion rate
- **User Experience:** Intuitive GUI with <5min learning curve

---

## 📋 Task Summary

| Phase | Tasks | High Priority | Medium Priority | Low Priority |
|-------|-------|---------------|-----------------|--------------|
| 1. Infrastructure | 3 | 2 | 1 | 0 |
| 2. Data Models | 3 | 2 | 1 | 0 |
| 3. Browser Automation | 3 | 2 | 1 | 0 |
| 4. Selector System | 4 | 2 | 1 | 1 |
| 5. Recording Engine | 3 | 2 | 1 | 0 |
| 6. Replay Engine | 4 | 2 | 1 | 1 |
| 7. Testing | 4 | 0 | 3 | 1 |
| 8. Documentation | 4 | 0 | 2 | 2 |
| 9. Security | 3 | 2 | 1 | 0 |
| 10. Production | 3 | 0 | 1 | 2 |
| 11. GUI Integration | 3 | 0 | 2 | 1 |
| 12. Optimization | 2 | 0 | 0 | 2 |
| **TOTAL** | **39** | **16** | **15** | **8** |

---

## 🚀 Implementation Roadmap

### Week 1-2: Foundation (Phase 1-2)
- Create flow-engine library
- Set up dependencies and workspace integration
- Implement core data models
- Create database schema extensions

### Week 3-4: Browser Infrastructure (Phase 3-4)
- Build browser management system
- Implement JavaScript injection
- Develop smart selector generation
- Create validation and blacklist systems

### Week 5-6: Core Engines (Phase 5-6)
- Implement recording engine
- Build replay engine with execution logic
- Add self-healing mechanisms
- Create session cookie management

### Week 7-8: Quality & Testing (Phase 7-9)
- Comprehensive testing suite
- Security implementation
- Error handling and resilience
- Resource management

### Week 9-10: Integration & Production (Phase 10-12)
- Production configuration
- GUI integration
- Performance optimization
- Documentation completion

---

## 📝 Notes & Constraints

- **No Circular Dependencies:** Ensure `flow-engine` only depends on `proxy-core` and `orchestrator`, not vice versa
- **Memory Safety:** Use Rust's ownership system and `secrecy` crate for sensitive data
- **Performance:** Target 10x better memory usage than Python/Node.js alternatives
- **Compatibility:** Must work with existing Proxxy agent infrastructure
- **Security:** All sensitive data must be masked and never logged
- **Testing:** Comprehensive test coverage required for production deployment

---

*Last Updated: $(date '+%Y-%m-%d %H:%M:%S')*
*Version: 1.0.0*
*Status: Ready for Implementation*