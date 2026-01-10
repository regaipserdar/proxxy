# 📝 NUCLEI_TASKS.md - Nuclei Scanner & Visual Builder Implementation

**Context:** Integration of ProjectDiscovery's Nuclei engine as a modular library.
**Strategy:** We support two modes:
1. **Standard Mode:** Running existing templates (CVEs, Misconfigs) with LSR Sessions.
2. **Visual Builder Mode:** Compiling React Flow graphs dynamically into Nuclei YAML templates.
**Constraint:** Must be a standalone crate. Communications with LSR happen only via `proxy-common::Session`.

---

## ☢️ Phase 1: Infrastructure & Crate Setup

### 1.1 Library Creation
- [ ] **Create scanner-nuclei library**
  - `cargo new --lib scanner-nuclei`
  - Add to workspace members in root `Cargo.toml`
  - Set up basic module structure (`src/lib.rs`, `src/manager.rs`, `src/compiler.rs`, `src/runner.rs`, `src/parser.rs`, `src/nodes.rs`)
  - **Priority:** HIGH | **Status:** PENDING

### 1.2 Dependency Management
- [ ] **Configure scanner-nuclei/Cargo.toml**
  - **Internal:** `proxy-common` (path: `../proxy-common`) - *Crucial for Session sharing*
  - **Async Runtime:** `tokio` (workspace version) - Process management
  - **Serialization:** `serde`, `serde_json`, `serde_yml` (workspace versions) - Parsing output & Generating YAML (⚠️ Use serde_yml fork instead of deprecated serde_yaml)
  - **HTTP Client:** `reqwest` (workspace version) - Downloading Nuclei binary
  - **Archive Handling:** `flate2`, `tar`, `zip` - Unpacking downloaded binaries
  - **File Management:** `tempfile` - Managing generated template files
  - **Directory Management:** `dirs` (workspace version) - Cross-platform directory paths (Linux: ~/.local/share/proxxy, Windows: C:\Users\X\AppData\Local\proxxy, Mac: ~/Library/Application Support/proxxy)
  - **Utilities:** `uuid`, `tracing` (workspace versions) - ID generation and logging
  - **Priority:** HIGH | **Status:** PENDING

### 1.3 Workspace Integration
- [ ] **Update workspace configuration**
  - Add scanner-nuclei to workspace members
  - Ensure version consistency across dependencies
  - Add to relevant integration test configurations
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🛠️ Phase 2: Binary Management System

### 2.1 Nuclei Binary Manager (`src/manager.rs`)
- [ ] **Struct: `NucleiBinary`**
  ```rust
  pub struct NucleiBinary {
      pub binary_path: PathBuf,
      pub version: Option<String>,
      pub templates_path: PathBuf,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Binary Existence Check**
  ```rust
  impl NucleiBinary {
      pub fn check_exists() -> bool
      pub fn get_binary_path() -> PathBuf
      pub fn get_templates_path() -> PathBuf
  }
  ```
  - Use `dirs` crate for cross-platform directory paths:
    - Linux: `~/.local/share/proxxy/bin/`
    - Windows: `C:\Users\X\AppData\Local\proxxy\bin\`
    - Mac: `~/Library/Application Support/proxxy/bin/`
  - Verify binary is executable
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Installation & Update System**
  ```rust
  pub async fn install_or_update() -> Result<NucleiBinary, NucleiError>
  ```
  - **OS Detection:** Detect OS (Windows/Linux/Mac) and Arch (x64/ARM)
  - **Download Logic:** Download latest release from GitHub Releases
  - **Unpack Process:** Unpack and `chmod +x` for Unix systems
  - **Version Tracking:** Store and track installed version
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Template Management**
  ```rust
  pub async fn update_templates() -> Result<(), NucleiError>
  pub async fn verify_templates() -> Result<bool, NucleiError>
  ```
  - Run `nuclei -update-templates` silently
  - Verify template integrity
  - Handle template update failures
  - **Priority:** MEDIUM | **Status:** PENDING

### 2.2 Configuration Management
- [ ] **Nuclei Configuration**
  ```rust
  pub struct NucleiConfig {
      pub binary_path: PathBuf,
      pub templates_path: PathBuf,
      pub auto_update: bool,
      pub concurrent_scans: u8,
      pub timeout_seconds: u64,
  }
  ```
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🎨 Phase 3: Visual Template Compiler

### 3.1 Data Models (`src/compiler.rs`)
- [ ] **Nuclei YAML Schema Structs**
  ```rust
  #[derive(Serialize, Deserialize, Debug)]
  pub struct NucleiTemplate {
      pub id: String,
      pub info: TemplateInfo,
      pub requests: Vec<Request>,
      pub variables: Option<HashMap<String, String>>,
  }

  #[derive(Serialize, Deserialize, Debug)]
  pub struct Request {
      pub method: Option<String>,
      pub path: Vec<String>,
      pub headers: Option<HashMap<String, String>>,
      pub body: Option<String>,
      pub matchers: Vec<Matcher>,
      pub extractors: Vec<Extractor>,
  }

  #[derive(Serialize, Deserialize, Debug)]
  pub struct Matcher {
      pub r#type: MatcherType,
      pub condition: Option<MatcherCondition>,
      pub name: Option<String>,
      pub part: Option<MatcherPart>,
      pub internal: Option<bool>,
  }

  #[derive(Serialize, Deserialize, Debug)]
  pub struct Extractor {
      pub r#type: ExtractorType,
      pub name: Option<String>,
      pub part: Option<ExtractorPart>,
      pub group: Option<u8>,
      pub regex: Option<Vec<String>>,
      pub internal: Option<bool>,
  }
  ```
  - Ensure `serde_yaml` produces valid Nuclei syntax
  - **Priority:** HIGH | **Status:** PENDING

### 3.2 Compiler Logic
- [ ] **Graph to YAML Compilation**
  ```rust
  pub fn compile_graph_to_yaml(
      nodes: &Vec<Node>,
      edges: &Vec<Edge>
  ) -> Result<String, CompilationError>
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Traversal Algorithm**
  - **Start Node Identification:** Identify the `StartNode` in the graph
  - **Edge Tracing:** Trace edges to different node types
  - **Request Mapping:** `RequestNode` -> Map to `requests` block
  - **Matcher Mapping:** `MatcherNode` -> Map to `matchers` block
  - **Extractor Mapping:** `ExtractorNode` -> Map to `extractors` block
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **YAML Generation & Validation**
  ```rust
  pub fn generate_yaml_from_template(template: &NucleiTemplate) -> Result<String, YamlError>
  pub fn validate_nuclei_syntax(yaml_content: &str) -> Result<bool, ValidationError>
  ```
  - Generate valid YAML from template structs
  - Validate generated YAML against Nuclei schema
  - **Priority:** MEDIUM | **Status:** PENDING

### 3.3 Template File Management
- [ ] **Temporary File Handling**
  ```rust
  pub fn save_template_to_file(yaml_content: &str) -> Result<PathBuf, FileError>
  pub fn cleanup_template_files(older_than: Duration) -> Result<(), FileError>
  ```
  - Save generated templates to temporary files (e.g., `/tmp/proxxy_flow_gen.yaml`)
  - Cleanup old template files
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🏃 Phase 4: Execution Engine

### 4.1 Scan Request Models (`src/runner.rs`)
- [ ] **Struct: `ScanRequest`**
  ```rust
  pub struct ScanRequest {
      pub target: Vec<String>,
      pub session: Option<Session>,
      pub source: ScanSource,
      pub config: ScanConfig,
  }

  pub enum ScanSource {
      Predefined { tags: String, severity: Option<String> },
      VisualGraph { yaml_path: PathBuf },
      CustomTemplate { template_path: PathBuf },
  }

  pub struct ScanConfig {
      pub concurrent: bool,
      pub timeout: Duration,
      pub rate_limit: Option<u32>,
      pub follow_redirects: bool,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 4.2 Command Builder Pattern
- [ ] **NucleiCommandBuilder**
  ```rust
  pub struct NucleiCommandBuilder {
      binary_path: PathBuf,
      args: Vec<String>,
  }

  impl NucleiCommandBuilder {
      pub fn new(binary_path: PathBuf) -> Self
      pub fn arg_target(mut self, target: Vec<String>) -> Self
      pub fn arg_template(mut self, template_path: PathBuf) -> Self
      pub fn arg_tags(mut self, tags: String) -> Self
      pub fn arg_severity(mut self, severity: String) -> Self
      pub fn arg_proxy(mut self, proxy_url: String) -> Self
      pub fn arg_auth(mut self, session: &Session) -> Self
      pub fn arg_output_format(mut self, format: OutputFormat) -> Self
      pub fn build(self) -> Command
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Target Handling**
  - Single URL: Adds `-u <url>` argument
  - Multiple URLs: Creates temp file and adds `-l <file>` argument
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Authentication Injection (LSR Bridge)**
  ```rust
  pub fn inject_session_auth(command: &mut Command, session: &Session) -> Result<(), AuthError>
  ```
  - Convert ALL Session headers to `-H "Key: Value"` format (not just cookies)
  - Handle Authorization: Bearer, X-CSRF-Token, Cookie, and other authentication headers
  - Add all headers as `-H "Header: Value"` arguments to Nuclei command
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Proxy Integration**
  - Always add `-proxy http://127.0.0.1:9095` to route traffic through Proxxy
  - Handle proxy authentication if needed
  - **Priority:** HIGH | **Status:** PENDING

### 4.3 Execution Engine
- [ ] **Scan Executor**
  ```rust
  pub async fn execute_scan(
      request: ScanRequest,
      binary: &NucleiBinary
  ) -> Result<mpsc::Receiver<NucleiFinding>, ExecutionError>
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Process Management with Zombie Prevention**
  - Spawn command with `Stdio::piped()`
  - Maintain `HashMap<ScanID, Child>` to track running processes
  - Implement timeout and cancellation with proper process kill:
    - Unix: Send SIGKILL to ensure process termination
    - Windows: Use taskkill to force terminate process
  - Handle Tauri shutdown and "Stop" button presses to prevent zombie processes
  - **Priority:** HIGH | **Status:** PENDING

---

## 📡 Phase 5: Stream Parser System

### 5.1 Output Parser (`src/parser.rs`)
- [ ] **NucleiFinding Struct**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(ignore_unknown_fields)]
  pub struct NucleiFinding {
      pub template_id: String,
      pub template_path: String,
      pub info: FindingInfo,
      pub matcher_name: Option<String>,
      pub extracted_values: Vec<String>,
      pub curl_command: Option<String>,
      pub request: Option<RequestInfo>,
      pub response: Option<ResponseInfo>,
      pub timestamp: chrono::DateTime<chrono::Utc>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(ignore_unknown_fields)]
  pub struct FindingInfo {
      pub name: String,
      pub severity: Severity,
      pub author: Vec<String>,
      pub tags: Vec<String>,
      pub description: Option<String>,
      pub reference: Vec<String>,
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 5.2 JSON Stream Parser
- [ ] **Asynchronous Line Parser**
  ```rust
  pub async fn parse_nuclei_output(
      stdout: ChildStdout,
      sender: mpsc::Sender<NucleiFinding>
  ) -> Result<(), ParseError>
  ```
  - Read `stdout` line-by-line asynchronously
  - Parse each line into `NucleiFinding` struct
  - Handle malformed JSON gracefully
  - **Priority:** HIGH | **Status:** PENDING

### 5.3 Real-time Streaming
- [ ] **Channel-based Communication**
  - Stream findings back to caller via `mpsc::Sender`
  - Enable real-time GUI updates
  - Handle backpressure and channel overflow
  - **Priority:** MEDIUM | **Status:** PENDING

### 5.4 Error Handling & Recovery
- [ ] **Parse Error Management**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum ParseError {
      #[error("Invalid JSON format: {0}")]
      InvalidJson(#[from] serde_json::Error),
      #[error("IO error: {0}")]
      IoError(#[from] std::io::Error),
      #[error("Channel closed unexpectedly")]
      ChannelClosed,
  }
  ```
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🧩 Phase 6: React Flow Nodes & GUI Models

### 6.1 Execution Nodes (`src/nodes.rs`)
- [ ] **NucleiRunnerNode**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct NucleiRunnerNode {
      pub id: String,
      pub node_type: NodeType,
      pub position: Position,
      pub data: NucleiRunnerData,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct NucleiRunnerData {
      pub scan_mode: ScanMode,
      pub target_source: TargetSource,
      pub template_source: TemplateSource,
      pub session_required: bool,
      pub config: ScanConfig,
  }

  pub enum ScanMode {
      Standard,
      VisualGraph,
      CustomTemplate,
  }
  ```
  - **Inputs:** `Session` (from LSR)
  - **Config:** Select "Standard Scan" vs "Visual Graph"
  - **Action:** Calls `runner::execute`
  - **Priority:** HIGH | **Status:** PENDING

### 6.2 Visual Builder Nodes
- [ ] **RequestNode**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct RequestNode {
      pub id: String,
      pub method: HttpMethod,
      pub path: String,
      pub headers: HashMap<String, String>,
      pub body: Option<String>,
      pub payload_type: PayloadType,
  }
  ```
  - UI to define Method (GET/POST), Path, Body
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **MatcherNode**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MatcherNode {
      pub id: String,
      pub matcher_type: MatcherType,
      pub condition: MatcherCondition,
      pub value: String,
      pub part: MatcherPart,
      pub negative: bool,
  }
  ```
  - UI to define Type (Word, Regex, Status) and Condition (AND/OR)
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **ExtractorNode**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ExtractorNode {
      pub id: String,
      pub extractor_type: ExtractorType,
      pub regex: Vec<String>,
      pub name: String,
      pub part: ExtractorPart,
      pub group: Option<u8>,
  }
  ```
  - UI to define Regex for capturing data to pass to the next request
  - **Priority:** MEDIUM | **Status:** PENDING

### 6.3 Node Validation
- [ ] **Graph Validation**
  ```rust
  pub fn validate_visual_graph(nodes: &Vec<Node>, edges: &Vec<Edge>) -> Result<ValidationReport, ValidationError>
  ```
  - Ensure graph connectivity
  - Validate node configurations
  - Check for circular dependencies
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🧪 Phase 7: Testing Strategy

### 7.1 Unit Tests
- [ ] **Compiler Tests** (`tests/unit/compiler.rs`)
  - Construct mock graph (Request -> Matcher) in Rust
  - Run `compile_graph_to_yaml`
  - Assert output string is valid YAML and matches Nuclei syntax
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Binary Manager Tests** (`tests/unit/manager.rs`)
  - Test OS detection logic
  - Mock download and installation process
  - Verify binary path resolution
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Parser Tests** (`tests/unit/parser.rs`)
  - Test JSON parsing with sample nuclei output
  - Verify error handling for malformed JSON
  - Test streaming functionality
  - **Priority:** MEDIUM | **Status:** PENDING

### 7.2 Integration Tests
- [ ] **Mock Execution Test** (`tests/integration/mock_execution.rs`)
  - Create dummy script mimicking Nuclei (prints JSON to stdout)
  - Point `scanner-nuclei` to this dummy
  - Verify JSON parsing works end-to-end
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Real Binary Integration Test** (`tests/integration/real_binary.rs`)
  - Download real Nuclei binary
  - Compile simple visual graph (GET / -> Match 200)
  - Run against `http://honey.scanme.sh`
  - Verify `Proxy` receives traffic and findings are parsed
  - **Priority:** LOW | **Status:** PENDING

### 7.3 Performance Tests
- [ ] **Concurrent Scan Tests** (`tests/performance/concurrent.rs`)
  - Test multiple simultaneous scans
  - Verify resource usage limits
  - Measure performance impact
  - **Priority:** LOW | **Status:** PENDING

### 7.4 Mock Server Infrastructure
- [ ] **Test Server Setup** (`tests/mock_server.rs`)
  - Create HTTP server with known vulnerabilities
  - Test various nuclei templates against it
  - Verify finding accuracy
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🔒 Phase 8: Security & Production Hardening

### 8.1 Binary Security
- [ ] **Binary Verification**
  ```rust
  pub async fn verify_binary_integrity(binary_path: &Path) -> Result<bool, SecurityError>
  ```
  - Verify downloaded binary checksum
  - Check binary signature if available
  - Validate binary permissions
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Sandboxed Execution**
  - Run nuclei with restricted permissions
  - Limit file system access
  - Control network access
  - **Priority:** MEDIUM | **Status:** PENDING

### 8.2 Session Security
- [ ] **Secure Session Handling**
  - Ensure session data is properly masked
  - Prevent session leakage in logs
  - Handle session expiration
  - **Priority:** HIGH | **Status:** PENDING

### 8.3 Template Security
- [ ] **Template Validation**
  ```rust
  pub fn validate_template_security(yaml_content: &str) -> Result<SecurityReport, SecurityError>
  ```
  - Check for malicious template content
  - Validate template syntax
  - Prevent code injection
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 📊 Phase 9: Monitoring & Observability

### 9.1 Metrics Collection
- [ ] **Scan Metrics**
  ```rust
  #[derive(Debug, Serialize)]
  pub struct ScanMetrics {
      pub scan_id: String,
      pub targets_count: usize,
      pub templates_count: usize,
      pub findings_count: usize,
      pub duration: Duration,
      pub success_rate: f64,
  }
  ```
  - Track scan performance
  - Monitor finding rates
  - Measure resource usage
  - **Priority:** MEDIUM | **Status:** PENDING

### 9.2 Logging Integration
- [ ] **Structured Logging**
  - Integrate with existing tracing infrastructure
  - Log scan start/end events
  - Record errors and warnings
  - **Priority:** MEDIUM | **Status:** PENDING

### 9.3 Health Checks
- [ ] **System Health**
  ```rust
  pub async fn health_check() -> HealthStatus
  ```
  - Verify binary installation
  - Check template updates
  - Test basic functionality
  - **Priority:** LOW | **Status:** PENDING

---

## 🔄 Phase 10: GUI Integration

### 10.1 React Flow Integration
- [ ] **Node Types Registration**
  - Register NucleiRunnerNode with React Flow
  - Register Visual Builder nodes
  - Implement node custom UI components
  - **Priority:** MEDIUM | **Status:** PENDING

### 10.2 Real-time Updates
- [ ] **WebSocket Integration**
  - Stream findings to GUI in real-time
  - Handle connection management
  - Update scan progress indicators
  - **Priority:** MEDIUM | **Status:** PENDING

### 10.3 Template Management UI
  - [ ] **Template Browser**
  - Browse available nuclei templates
  - Filter by tags and severity
  - Preview template content
  - **Priority:** LOW | **Status:** PENDING

### 10.5 Tauri Command Interface (`src-tauri/src/commands/nuclei.rs`)
- [ ] **Tauri Bridge Commands**
  ```rust
  #[tauri::command]
  pub async fn install_nuclei(window: Window) -> Result<(), String>

  #[tauri::command]
  pub async fn start_scan(config: ScanRequest, session_id: String, window: Window) -> Result<(), String>

  #[tauri::command]
  pub async fn stop_scan(scan_id: String) -> Result<(), String>

  #[tauri::command]
  pub fn validate_graph(json: String) -> Result<(), String>
  ```
- **Installation Command:** Stream download progress to Frontend via emit events
- **Start Scan Command:** Call `scanner-nuclei::execute_scan` and forward findings to Frontend via emit("finding", data)
- **Stop Scan Command:** Kill process via SIGKILL (Unix) or taskkill (Windows) to prevent zombie processes
- **Validate Graph Command:** Send React Flow JSON to `scanner-nuclei::compiler` and return validation errors with line numbers
- **Priority:** HIGH | **Status:** PENDING

---

## 🚀 Phase 11: Performance Optimization

### 11.1 Concurrent Execution
- [ ] **Parallel Scanning**
  - Implement concurrent scan execution
  - Manage resource pools
  - Optimize for multi-core systems
  - **Priority:** MEDIUM | **Status:** PENDING

### 11.2 Caching System
- [ ] **Result Caching**
  - Cache scan results for repeated targets
  - Implement cache invalidation
  - Optimize cache storage
  - **Priority:** LOW | **Status:** PENDING

### 11.3 Resource Management
- [ ] **Memory Optimization**
  - Optimize memory usage for large scans
  - Implement streaming for large result sets
  - Manage temporary file cleanup
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 📚 Phase 12: Documentation & Examples

### 12.1 API Documentation
- [ ] **Comprehensive Code Docs**
  - Document all public APIs
  - Include usage examples
  - Document error handling
  - **Priority:** MEDIUM | **Status:** PENDING

### 12.2 User Guides
- [ ] **Getting Started Guide**
  - Installation instructions
  - Basic usage examples
  - Visual Builder tutorial
  - **Priority:** LOW | **Status:** PENDING

### 12.3 Integration Examples
- [ ] **Code Examples**
  - Standard mode examples
  - Visual Builder examples
  - LSR integration examples
  - **Priority:** LOW | **Status:** PENDING

---

## ✅ Acceptance Criteria

Each phase should be considered complete when:

1. **All unit tests pass** with >90% code coverage
2. **Integration tests validate** both Standard and Visual modes
3. **Security review is completed** for binary execution
4. **Performance benchmarks meet** production requirements
5. **GUI integration is functional** with real-time updates
6. **Documentation is comprehensive** and up-to-date

---

## 🎯 Success Metrics

- **Binary Installation:** >99% success rate across platforms
- **Template Compilation:** <1s compilation time for complex graphs
- **Scan Execution:** Real-time finding streaming with <100ms latency
- **Visual Builder:** Intuitive graph-to-YAML conversion with >95% accuracy
- **Resource Usage:** <200MB RAM per concurrent scan
- **Integration:** Seamless LSR session sharing with zero data leakage

---

## 📋 Task Summary

| Phase | Tasks | High Priority | Medium Priority | Low Priority |
|-------|-------|---------------|-----------------|--------------|
| 1. Infrastructure | 3 | 2 | 1 | 0 |
| 2. Binary Management | 3 | 2 | 1 | 0 |
| 3. Visual Compiler | 3 | 2 | 1 | 0 |
| 4. Execution Engine | 3 | 2 | 1 | 0 |
| 5. Stream Parser | 3 | 2 | 1 | 0 |
| 6. GUI Nodes | 3 | 1 | 2 | 0 |
| 7. Testing | 4 | 0 | 3 | 1 |
| 8. Security | 3 | 2 | 1 | 0 |
| 9. Monitoring | 3 | 0 | 2 | 1 |
| 10. GUI Integration | 4 | 1 | 2 | 1 |
| 11. Performance | 3 | 0 | 2 | 1 |
| 12. Documentation | 3 | 0 | 1 | 2 |
| **TOTAL** | 38 | 17 | 18 | 3 |

---

## 🚀 Implementation Roadmap

### Week 1: Foundation (Phase 1-2)
- Create scanner-nuclei library
- Set up dependencies and workspace integration
- Implement binary management system
- Create installation and update mechanisms

### Week 2: Core Engine (Phase 3-4)
- Build visual template compiler
- Implement execution engine with command builder
- Create authentication and proxy integration
- Develop stream parser system

### Week 3: GUI Integration (Phase 5-6)
- Implement React Flow nodes
- Create visual builder components
- Add real-time streaming capabilities
- Build node validation system

### Week 4: Quality & Security (Phase 7-8)
- Comprehensive testing suite
- Security implementation and hardening
- Performance optimization
- Monitoring and observability

### Week 5: Production Ready (Phase 9-12)
- GUI integration completion
- Documentation and examples
- Performance optimization
- Production deployment preparation

---

## 📝 Notes & Constraints

- **Standalone Crate:** Must be independent, only communicates via `proxy-common::Session`
- **Binary Management:** Automatic download and installation of Nuclei binary
- **Visual Builder:** Graph-to-YAML compilation with validation
- **Session Bridge:** Secure LSR session sharing for authenticated scans
- **Real-time Streaming:** Live finding updates to GUI
- **Security First:** Binary verification and sandboxed execution
- **Performance:** Concurrent scanning with resource management

---

*Last Updated: $(date '+%Y-%m-%d %H:%M:%S')*
*Version: 1.0.0*
*Status: Ready for Implementation*