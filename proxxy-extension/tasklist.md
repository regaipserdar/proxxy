# 🌐 Proxxy Browser Extension - Implementation Tasks

## 📋 Executive Summary

**What:** A hybrid Chrome Extension + Proxxy GUI system that provides in-browser controls for HAR recording and LSR (Login Sequence Recorder) functionality.

**Why:** 
- **User Experience**: Users can control recording directly from the browser without switching to Proxxy GUI
- **Seamless Integration**: Extension acts as a lightweight remote control while heavy processing stays in Rust
- **Developer Workflow**: Natural integration into existing browser-based security testing workflow
- **Best of Both Worlds**: Simple, responsive UI in browser + powerful backend processing in Proxxy

**How:**
- Chrome Extension provides UI controls (DevTools panel + toolbar button)
- Extension communicates with Proxxy via Native Messaging Protocol
- Proxxy backend handles HAR processing, LSR recording/replay, and session management
- Bidirectional real-time updates via message passing

**Architecture:**
```
┌─────────────────────┐         ┌──────────────────────┐
│  Chrome Extension   │ ◄─────► │   Proxxy Backend     │
│  (UI Controls)      │  Native │   (Business Logic)   │
│                     │  Msg    │                      │
│  - DevTools Panel   │         │  - HAR Manager       │
│  - Popup/Toolbar    │         │  - LSR Recorder      │
│  - Status Display   │         │  - Session Manager   │
└─────────────────────┘         └──────────────────────┘
```

---

## 🎯 Phase 1: Chrome Extension Foundation

### 1.1 Project Structure Setup
**What:** Create Chrome Extension project skeleton with modern tooling  
**Why:** Proper structure ensures maintainability and easy deployment

- [ ] **Create extension directory structure**
  ```
  proxxy/
  └── extensions/
      └── proxxy-chrome/
          ├── manifest.json
          ├── src/
          │   ├── background/
          │   ├── devtools/
          │   ├── popup/
          │   └── content/
          ├── assets/
          ├── dist/
          └── package.json
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Configure build tooling**
  - Choose build tool: Vite/Webpack/Parcel for bundling
  - Setup TypeScript for type safety
  - Configure hot reload for development
  - Add minification for production builds
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Create manifest.json v3**
  ```json
  {
    "manifest_version": 3,
    "name": "Proxxy Security Toolkit",
    "version": "1.0.0",
    "description": "Browser controls for Proxxy HAR & LSR features",
    "permissions": [
      "debugger",
      "tabs",
      "storage",
      "nativeMessaging"
    ],
    "host_permissions": ["<all_urls>"],
    "devtools_page": "devtools.html",
    "action": {
      "default_popup": "popup.html",
      "default_icon": "assets/icon-48.png"
    },
    "background": {
      "service_worker": "background.js"
    }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 1.2 UI Components Design
**What:** Design and implement user-facing interface components  
**Why:** Intuitive UI is critical for user adoption

- [ ] **Create DevTools Panel (HAR Control)**
  - Panel layout with Start/Stop/Clear/Download buttons
  - Real-time request counter display
  - Filter controls (by domain, type, size)
  - Visual recording indicator (pulsing red dot)
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Create DevTools Panel (LSR Control)**
  - Record/Stop/Replay buttons
  - Step counter and progress indicator
  - Profile selector dropdown
  - Session status indicator
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Create Toolbar Popup UI**
  - Quick status overview (HAR: Recording, LSR: Idle)
  - Quick action buttons
  - Link to open DevTools panel
  - Settings shortcut
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Design System & Styling**
  - Match Proxxy brand colors and theme
  - Dark/Light mode support
  - Responsive layout for different DevTools sizes
  - Loading states and error states
  - **Priority:** MEDIUM | **Status:** PENDING

### 1.3 Background Service Worker
**What:** Implement persistent background script for state management  
**Why:** Service worker handles extension lifecycle and message routing

- [ ] **Service Worker Setup**
  ```typescript
  // background/index.ts
  import { NativeMessagingHost } from './native-host';
  import { StateManager } from './state';
  
  const nativeHost = new NativeMessagingHost('com.proxxy.native');
  const state = new StateManager();
  
  chrome.runtime.onInstalled.addListener(() => {
    console.log('Proxxy Extension installed');
  });
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Message Router Implementation**
  - Route messages between panels, popup, and native host
  - Handle connection lifecycle (connect/disconnect)
  - Implement message queuing for reliability
  - Error handling and retry logic
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **State Synchronization**
  - Maintain recording state (HAR/LSR status)
  - Sync state across all extension components
  - Persist state to chrome.storage
  - Broadcast state changes to open panels
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🔌 Phase 2: Native Messaging Protocol

### 2.1 Protocol Definition
**What:** Define message format for Extension ↔ Proxxy communication  
**Why:** Standardized protocol ensures reliable bidirectional communication

- [ ] **Message Schema Design**
  ```typescript
  // Protocol types
  interface NativeMessage {
    id: string;           // UUID for request tracking
    module: 'har' | 'lsr';
    action: string;       // 'start', 'stop', 'status', etc.
    payload?: any;
    timestamp: number;
  }
  
  interface NativeResponse {
    id: string;           // Matches request ID
    success: boolean;
    data?: any;
    error?: string;
  }
  
  interface StatusUpdate {
    module: 'har' | 'lsr';
    status: string;
    data: any;
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Command Definitions**
  - **HAR Module:**
    - `har_start`: Start recording
    - `har_stop`: Stop and save
    - `har_clear`: Clear buffer
    - `har_export`: Download HAR file
    - `har_status`: Get current state
  - **LSR Module:**
    - `lsr_record_start`: Begin recording
    - `lsr_record_stop`: End recording
    - `lsr_replay`: Execute profile
    - `lsr_list_profiles`: Get saved profiles
    - `lsr_delete_profile`: Remove profile
  - **Priority:** HIGH | **Status:** PENDING

### 2.2 Extension-Side Implementation
**What:** Implement native messaging client in extension  
**Why:** Handles Chrome's native messaging API and message serialization

- [ ] **Native Host Connector**
  ```typescript
  // background/native-host.ts
  export class NativeMessagingHost {
    private port: chrome.runtime.Port | null = null;
    private messageQueue: Map<string, PendingMessage> = new Map();
    
    connect() {
      this.port = chrome.runtime.connectNative('com.proxxy.native');
      this.port.onMessage.addListener(this.handleMessage);
      this.port.onDisconnect.addListener(this.handleDisconnect);
    }
    
    async sendCommand(module, action, payload) {
      const id = crypto.randomUUID();
      const message = { id, module, action, payload, timestamp: Date.now() };
      
      return new Promise((resolve, reject) => {
        this.messageQueue.set(id, { resolve, reject });
        this.port.postMessage(message);
      });
    }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Response Handler & Timeout**
  - Match responses to requests by ID
  - Implement 30s timeout for pending requests
  - Handle native host disconnection gracefully
  - Reconnection logic with exponential backoff
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Stream Handler for Real-time Updates**
  - Subscribe to status updates from Proxxy
  - Handle streaming HAR request data
  - Update UI in real-time (request counter, etc.)
  - **Priority:** MEDIUM | **Status:** PENDING

### 2.3 Rust-Side Native Host
**What:** Build native messaging host binary in Proxxy  
**Why:** Bridges Chrome Extension with Proxxy backend functionality

- [ ] **Create native host module** (`flow-engine/src/native_host/`)
  ```rust
  // src/native_host/mod.rs
  use serde::{Deserialize, Serialize};
  use std::io::{self, Read, Write};
  use tokio::sync::mpsc;
  
  #[derive(Debug, Serialize, Deserialize)]
  pub struct NativeMessage {
      pub id: String,
      pub module: String,
      pub action: String,
      pub payload: Option<serde_json::Value>,
      pub timestamp: u64,
  }
  
  #[derive(Debug, Serialize, Deserialize)]
  pub struct NativeResponse {
      pub id: String,
      pub success: bool,
      pub data: Option<serde_json::Value>,
      pub error: Option<String>,
  }
  
  pub struct NativeHost {
      har_manager: Arc<HarManager>,
      lsr_recorder: Arc<LsrRecorder>,
  }
  
  impl NativeHost {
      pub async fn run(&self) -> Result<(), Error> {
          loop {
              let msg = self.read_message()?;
              let response = self.handle_message(msg).await;
              self.write_message(&response)?;
          }
      }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Message I/O Implementation**
  ```rust
  fn read_message(&self) -> io::Result<NativeMessage> {
      // Chrome sends: [4 bytes length][JSON message]
      let mut len_bytes = [0u8; 4];
      io::stdin().read_exact(&mut len_bytes)?;
      let len = u32::from_le_bytes(len_bytes) as usize;
      
      let mut buffer = vec![0u8; len];
      io::stdin().read_exact(&mut buffer)?;
      
      serde_json::from_slice(&buffer)
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }
  
  fn write_message(&self, msg: &NativeResponse) -> io::Result<()> {
      let json = serde_json::to_vec(msg)?;
      let len = (json.len() as u32).to_le_bytes();
      
      io::stdout().write_all(&len)?;
      io::stdout().write_all(&json)?;
      io::stdout().flush()?;
      
      Ok(())
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Command Dispatcher**
  ```rust
  async fn handle_message(&self, msg: NativeMessage) -> NativeResponse {
      let result = match msg.module.as_str() {
          "har" => self.handle_har_command(&msg).await,
          "lsr" => self.handle_lsr_command(&msg).await,
          _ => Err(format!("Unknown module: {}", msg.module)),
      };
      
      match result {
          Ok(data) => NativeResponse {
              id: msg.id,
              success: true,
              data: Some(data),
              error: None,
          },
          Err(e) => NativeResponse {
              id: msg.id,
              success: false,
              data: None,
              error: Some(e),
          },
      }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Status Broadcasting**
  - Background task that sends periodic status updates
  - Notify extension of state changes (HAR started, LSR step completed)
  - Use channels for async communication
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🎛️ Phase 3: HAR Module Integration

### 3.1 HAR Control Implementation
**What:** Wire extension buttons to HAR manager backend  
**Why:** Enable users to control HAR recording from browser

- [ ] **Extension HAR Panel**
  ```typescript
  // devtools/panels/har.ts
  class HARPanel {
    private nativeHost: NativeMessagingHost;
    private state = { recording: false, requests: 0 };
    
    async startRecording() {
      const result = await this.nativeHost.sendCommand('har', 'start', {
        filter: this.getFilterConfig()
      });
      
      if (result.success) {
        this.updateUI({ recording: true });
      }
    }
    
    async stopRecording() {
      const result = await this.nativeHost.sendCommand('har', 'stop', {});
      this.updateUI({ recording: false });
    }
    
    async downloadHAR() {
      const result = await this.nativeHost.sendCommand('har', 'export', {
        filename: `capture_${Date.now()}.har`
      });
      // Trigger browser download
      chrome.downloads.download({
        url: URL.createObjectURL(new Blob([result.data.har])),
        filename: result.data.filename
      });
    }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Rust HAR Command Handlers**
  ```rust
  async fn handle_har_command(&self, msg: &NativeMessage) -> Result<serde_json::Value, String> {
      match msg.action.as_str() {
          "start" => {
              let filter = msg.payload.as_ref()
                  .and_then(|p| p.get("filter"))
                  .and_then(|f| f.as_str());
              
              self.har_manager.start_recording(filter).await?;
              Ok(json!({ "status": "recording" }))
          }
          "stop" => {
              let har_data = self.har_manager.stop_recording().await?;
              Ok(json!({ "status": "stopped", "requests": har_data.entries.len() }))
          }
          "export" => {
              let har_json = self.har_manager.export_har().await?;
              Ok(json!({ "har": har_json, "filename": msg.payload["filename"] }))
          }
          "clear" => {
              self.har_manager.clear().await?;
              Ok(json!({ "status": "cleared" }))
          }
          _ => Err(format!("Unknown HAR action: {}", msg.action))
      }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Real-time Request Counter**
  - Stream request count updates to extension
  - Update DevTools badge with request count
  - Implement efficient diffing to avoid excessive updates
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Filter Configuration UI**
  - Domain filtering (include/exclude patterns)
  - Resource type filtering (XHR, Document, Image, etc.)
  - Size threshold filtering
  - Save filter presets
  - **Priority:** MEDIUM | **Status:** PENDING

### 3.2 HAR Visualization in Extension
**What:** Display HAR data summary in extension UI  
**Why:** Quick overview without opening Proxxy GUI

- [ ] **Request List View**
  - Tabular display of recent requests
  - Columns: Method, URL, Status, Size, Time
  - Click to see request/response details
  - **Priority:** LOW | **Status:** PENDING

- [ ] **Summary Statistics**
  - Total requests, total size, recording duration
  - Requests by domain (pie chart)
  - Timeline visualization
  - **Priority:** LOW | **Status:** PENDING

---

## 🔐 Phase 4: LSR Module Integration

### 4.1 LSR Recording Controls
**What:** Enable login recording from browser extension  
**Why:** Users can start/stop recording without leaving browser context

- [ ] **Extension LSR Panel**
  ```typescript
  // devtools/panels/lsr.ts
  class LSRPanel {
    async startRecording(profileName: string) {
      const result = await this.nativeHost.sendCommand('lsr', 'record_start', {
        profile_name: profileName,
        start_url: this.getCurrentTabUrl()
      });
      
      if (result.success) {
        this.showRecordingIndicator();
        this.subscribeToStepUpdates();
      }
    }
    
    async stopRecording() {
      const result = await this.nativeHost.sendCommand('lsr', 'record_stop', {});
      this.displayRecordedSteps(result.data.steps);
    }
    
    subscribeToStepUpdates() {
      // Listen for real-time step capture events
      this.nativeHost.onStatusUpdate('lsr', (update) => {
        this.appendStep(update.data.step);
      });
    }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Rust LSR Command Handlers**
  ```rust
  async fn handle_lsr_command(&self, msg: &NativeMessage) -> Result<serde_json::Value, String> {
      match msg.action.as_str() {
          "record_start" => {
              let profile_name = msg.payload["profile_name"].as_str().unwrap();
              let start_url = msg.payload["start_url"].as_str().unwrap();
              
              let session = self.lsr_recorder.start_recording(profile_name, start_url).await?;
              Ok(json!({ "session_id": session.id, "status": "recording" }))
          }
          "record_stop" => {
              let profile = self.lsr_recorder.stop_recording().await?;
              Ok(json!({ 
                  "profile_id": profile.id,
                  "steps": profile.steps.len(),
                  "steps_detail": profile.steps 
              }))
          }
          "replay" => {
              let profile_id = msg.payload["profile_id"].as_str().unwrap();
              let result = self.lsr_recorder.replay_profile(profile_id).await?;
              Ok(json!({ "success": result.success, "cookies": result.cookies }))
          }
          "list_profiles" => {
              let profiles = self.lsr_recorder.list_profiles().await?;
              Ok(json!({ "profiles": profiles }))
          }
          _ => Err(format!("Unknown LSR action: {}", msg.action))
      }
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 4.2 Profile Management UI
**What:** Manage saved login profiles from extension  
**Why:** Quick access to replay without opening main Proxxy UI

- [ ] **Profile List View**
  - Display saved profiles with name, URL, created date
  - Search/filter profiles
  - Quick replay button
  - Delete profile action
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Profile Details Modal**
  - Show recorded steps
  - Edit profile name
  - View success/failure history
  - Export profile as JSON
  - **Priority:** MEDIUM | **Status:** PENDING

### 4.3 Replay Visualization
**What:** Show replay progress in real-time  
**Why:** User feedback during automated login execution

- [ ] **Progress Indicator**
  - Current step display (e.g., "Step 3/7: Typing password")
  - Progress bar
  - Estimated time remaining
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Replay Log Stream**
  - Real-time log of actions (Navigate, Click, Type)
  - Success/failure indicators
  - Error messages if replay fails
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Session Cookie Display**
  - Show extracted cookies after successful login
  - Copy to clipboard functionality
  - Inject cookies to current tab option
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🔧 Phase 5: Installation & Configuration

### 5.1 Native Host Registration
**What:** Automate native messaging host registration  
**Why:** Users shouldn't manually edit registry/manifest files

- [ ] **Create native host manifest file**
  ```json
  // com.proxxy.native.json
  {
    "name": "com.proxxy.native",
    "description": "Proxxy Native Messaging Host",
    "path": "/path/to/proxxy-native-host",
    "type": "stdio",
    "allowed_origins": [
      "chrome-extension://YOUR_EXTENSION_ID/"
    ]
  }
  ```
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **Platform-specific installers**
  - **Windows:** Registry entry creation script
    ```
    HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\com.proxxy.native
    ```
  - **macOS:** Copy manifest to `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/`
  - **Linux:** Copy manifest to `~/.config/google-chrome/NativeMessagingHosts/`
  - **Priority:** HIGH | **Status:** PENDING

- [ ] **CLI command for installation**
  ```bash
  proxxy extension install
  # Output: 
  # ✓ Native host registered
  # ✓ Extension ID: abc123...
  # → Install extension from chrome://extensions
  ```
  - **Priority:** HIGH | **Status:** PENDING

### 5.2 Extension Settings Panel
**What:** Configuration UI for extension behavior  
**Why:** Users need to customize behavior without editing code

- [ ] **Settings UI**
  - Proxxy binary path configuration
  - Auto-connect on browser start
  - Default HAR filters
  - Default LSR profile directory
  - Theme selection (Dark/Light)
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Connection Health Check**
  - Test native host connection
  - Display Proxxy version
  - Show connection status indicator
  - Troubleshooting diagnostics
  - **Priority:** MEDIUM | **Status:** PENDING

### 5.3 Update Mechanism
**What:** Handle extension and native host version compatibility  
**Why:** Prevent breaking changes when Proxxy updates

- [ ] **Version Negotiation**
  ```rust
  // On connection, exchange versions
  {
    "action": "handshake",
    "extension_version": "1.2.0",
    "protocol_version": "1.0"
  }
  // Response:
  {
    "proxxy_version": "0.5.0",
    "protocol_version": "1.0",
    "compatible": true
  }
  ```
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Migration Handler**
  - Detect protocol version mismatches
  - Show upgrade prompt if incompatible
  - Graceful degradation if possible
  - **Priority:** LOW | **Status:** PENDING

---

## 🧪 Phase 6: Testing & Quality Assurance

### 6.1 Extension Unit Tests
**What:** Test individual components in isolation  
**Why:** Ensure reliability before integration

- [ ] **Test Framework Setup**
  - Use Jest + Chrome Extension Testing Library
  - Mock chrome.* APIs
  - Setup coverage reporting
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Component Tests**
  - Test HAR panel button interactions
  - Test LSR panel state management
  - Test message serialization/deserialization
  - Test native host reconnection logic
  - **Priority:** MEDIUM | **Status:** PENDING

### 6.2 Integration Tests
**What:** Test extension ↔ native host communication  
**Why:** Verify end-to-end message flow

- [ ] **Mock Native Host**
  - Create test harness that simulates Proxxy responses
  - Test all command/response patterns
  - Test error scenarios (timeout, disconnect)
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **E2E Test Scenarios**
  - Full HAR recording workflow
  - Full LSR record → replay workflow
  - Connection failure recovery
  - **Priority:** MEDIUM | **Status:** PENDING

### 6.3 Rust Native Host Tests
**What:** Test native messaging host in isolation  
**Why:** Verify protocol compliance and error handling

- [ ] **Protocol Tests**
  ```rust
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_message_serialization() {
          let msg = NativeMessage { ... };
          let bytes = serialize_message(&msg);
          assert_eq!(bytes.len(), msg_size + 4); // 4 byte header
      }
      
      #[tokio::test]
      async fn test_har_start_command() {
          let host = NativeHost::new();
          let response = host.handle_message(NativeMessage {
              action: "start",
              module: "har",
              ...
          }).await;
          assert!(response.success);
      }
  }
  ```
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 📚 Phase 7: Documentation & Distribution

### 7.1 User Documentation
**What:** Comprehensive guides for end users  
**Why:** Reduce support burden and improve UX

- [ ] **Installation Guide**
  - Step-by-step with screenshots
  - Troubleshooting common issues
  - Platform-specific notes
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **Feature Tutorials**
  - HAR recording walkthrough
  - LSR profile creation guide
  - Advanced filtering techniques
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **FAQ & Troubleshooting**
  - "Extension shows disconnected" → Solutions
  - "No profiles appear" → Check permissions
  - Performance optimization tips
  - **Priority:** LOW | **Status:** PENDING

### 7.2 Developer Documentation
**What:** Technical docs for contributors  
**Why:** Enable community contributions and maintenance

- [ ] **Architecture Documentation**
  - Message flow diagrams
  - State management explanation
  - Native messaging protocol spec
  - **Priority:** MEDIUM | **Status:** PENDING

- [ ] **API Reference**
  - All native host commands documented
  - TypeScript interfaces for messages
  - Error codes and meanings
  - **Priority:** LOW | **Status:** PENDING

### 7.3 Distribution Strategy
**What:** Package and distribute extension  
**Why:** Make it easy for users to install

- [ ] **Chrome Web Store Submission**
  - Create store listing with screenshots
  - Privacy policy and terms
  - Submit for review
  - **Priority:** LOW | **Status:** PENDING

- [ ] **Self-Hosted Option**
  - Generate unpacked extension ZIP
  - Include in Proxxy releases
  - Document manual installation
  - **Priority:** MEDIUM | **Status:** PENDING

---

## 🚀 Phase 8: Advanced Features (Future)

### 8.1 Enhanced UI Features
- [ ] **Keyboard Shortcuts**
  - `Ctrl+Shift+H`: Toggle HAR recording
  - `Ctrl+Shift+L`: Start LSR recording
  - **Priority:** LOW | **Status:** PENDING

- [ ] **Notification System**
  - Browser notifications for recording start/stop
  - Error notifications
  - **Priority:** LOW | **Status:** PENDING

### 8.2 Cross-Browser Support
- [ ] **Firefox Extension**
  - Adapt to WebExtensions API differences
  - Separate manifest.json for Firefox
  - **Priority:** LOW | **Status:** PENDING

- [ ] **Edge Extension**
  - Test compatibility
  - Submit to Edge Add-ons store
  - **Priority:** LOW | **Status:** PENDING

### 8.3 Performance Optimizations
- [ ] **Message Batching**
  - Batch status updates to reduce overhead
  - Debounce UI updates
  - **Priority:** LOW | **Status:** PENDING

- [ ] **Lazy Loading**
  - Load panels only when opened
  - Defer non-critical initialization
  - **Priority:** LOW | **Status:** PENDING

---

## ✅ Acceptance Criteria

### Minimum Viable Product (MVP)
- ✅ Extension installs and connects to Proxxy native host
- ✅ HAR recording can be started/stopped from extension
- ✅ LSR recording can be started/stopped from extension
- ✅ Basic error handling and user feedback
- ✅ Works on Chrome (Windows, macOS, Linux)

### Production Ready
- ✅ All automated tests passing (>80% coverage)
- ✅ Connection resilience (auto-reconnect)
- ✅ User documentation complete
- ✅ Chrome Web Store approved (or self-hosted ZIP available)
- ✅ Performance: <100ms message round-trip latency

---

## 📊 Implementation Roadmap

### Sprint 1 (Week 1-2): Foundation
- Phase 1: Extension structure and UI mockups
- Phase 2: Native messaging protocol definition
- Basic connection establishment

### Sprint 2 (Week 3-4): HAR Integration
- Phase 3: HAR control implementation
- Basic UI for HAR recording
- Test HAR start/stop flow

### Sprint 3 (Week 5-6): LSR Integration
- Phase 4: LSR recording controls
- Profile management UI
- Replay functionality

### Sprint 4 (Week 7): Polish & Testing
- Phase 6: Comprehensive testing
- Bug fixes and performance tuning
- Documentation

### Sprint 5 (Week 8): Release
- Phase 5: Installation automation
- Phase 7: Documentation and distribution
- Beta release to select users

---

## 🔢 Task Summary

| Phase | Tasks | High Priority | Medium Priority | Low Priority |
|-------|-------|---------------|-----------------|--------------|
| 1. Foundation | 8 | 5 | 2 | 1 |
| 2. Protocol | 7 | 5 | 2 | 0 |
| 3. HAR | 6 | 3 | 2 | 1 |
| 4. LSR | 9 | 3 | 4 | 2 |
| 5. Install | 7 | 3 | 3 | 1 |
| 6. Testing | 5 | 0 | 4 | 1 |
| 7. Docs | 6 | 0 | 3 | 3 |
| 8. Future | 7 | 0 | 1 | 6 |
| **TOTAL** | **55** | **19** | **21** | **15** |

---

## 📝 Notes & Constraints

**Technical Constraints:**
- Chrome Extension Manifest V3 required (V2 deprecated)
- Native messaging limited to stdio communication
- Extension cannot directly access file system (must go through native host)
- Service worker has limited lifetime (must handle wake/sleep)

**Security Considerations:**
- Extension ID must be pinned in native host manifest
- Validate all messages from extension (don't trust client)
- Sanitize file paths and user inputs
- No sensitive data in extension storage (use Proxxy backend)

**Performance Targets:**
- Message latency: <100ms (extension → native host → extension)
- UI responsiveness: 60 FPS, no jank
- Memory footprint: <50MB for extension
- Startup time: <1s to connect to native host

**User Experience:**
- Zero-config installation