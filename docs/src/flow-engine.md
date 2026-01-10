# Login Sequence Recorder (LSR) Implementation

**Context:** This document is the *Single Source of Truth* for the LSR module in the `Proxxy` project.
**Scope:** Development of the `flow-engine` library, Workspace integration, and HAR support.

## 🛠️ 1. Infrastructure & Workspace Integration
*Infrastructure setup and data structure compatibility with Orchestrator.*

- [ ] **Workspace Setup**
    - [ ] Create library via `cargo new --lib flow-engine`.
    - [ ] Add `"flow-engine"` to `[workspace.members]` in root `Cargo.toml`.
- [ ] **Dependency Management (`flow-engine/Cargo.toml`)**
    - [ ] Add libraries (latest stable):
        - `chromiumoxide` (Browser Automation)
        - `tokio` (Async Runtime)
        - `serde`, `serde_json` (Serialization)
        - `thiserror`, `anyhow` (Error Handling)
        - `secrecy` (Secure String)
        - `uuid` (ID Generation)
        - `url` (Parsing)
        - `base64` (Encoding)
        - `har` (or equivalent HAR parsing crate)
    - [ ] **Internal Dependencies:**
        - Path reference to `orchestrator` (if sharing structs).
        - Path reference to `proxy-core` (or a common crate).

## 🧬 2. Data Models & HAR Support (`src/lsr/model.rs`)
*Data models and HAR file conversions.*

- [ ] **Struct: `LoginProfile`**
    - Fields: `id`, `name`, `start_url`, `steps`, `meta`.
    - Must be JSON serializable via `serde`.
- [ ] **Enum: `LoginStep`**
    - Variants: `Navigate`, `Click` (SmartSelector), `Type` (Secret), `Wait`, `CheckSession`.
- [ ] **HAR Integration (`src/lsr/har.rs`)**
    - [ ] **Import:** Write `from_har(path: &str) -> LoginProfile` to convert standard `.har` files.
        - *Logic:* Analyze POST requests for form data (-> `Type`), URL transitions (-> `Navigate`).
    - [ ] **Export:** Export `LoginProfile` as a simulated `.har` file for debugging.

## 🎥 3. Recorder Engine Implementation
*Browser management and CDP event listeners.*

- [ ] **Module: `src/lsr/browser.rs`**
    - [ ] Write `launch_browser(proxy_addr: Option<&str>)`.
    - [ ] Use `chromiumoxide` with arguments compatible with Workspace Proxy config (`--proxy-server`).
    - [ ] Ignore SSL errors (`--ignore-certificate-errors`).
- [ ] **Module: `src/lsr/recorder.rs`**
    - [ ] **Event Listeners:** Listen to DOM and Network events via CDP.
    - [ ] **JS Injection:** Inject JS to capture Click and Keyboard events.
    - [ ] **Orchestrator Sync:** Notify Orchestrator of "RecordingActive" status (via gRPC/Channel).

## 🧠 4. Smart Selector & Analysis (`src/lsr/analyzer.rs`)
*Generating Resilient (Robust) Selectors.*

**Constraint:** Do not use a single "magic" library. Build a custom algorithm using `chromiumoxide` Node data. DO NOT use external HTML parsers for selector generation logic to ensure performance and consistency with the live page.

- [ ] **Struct: `SelectorStrategy`**
    - Fields: `priority: u8`, `value: String`, `selector_type: SelectorType` (ID, XPath, CSS, Attribute).

- [ ] **Algorithm: `calculate_best_selector(node: &cdp::browser_protocol::dom::Node) -> SmartSelector`**
    - [ ] **Step 1 (Attributes):** Scan node attributes (`data-testid`, `id`, `name`).
        - *Filter:* Exclude numeric or semi-random IDs via regex (`regex = r"^[a-zA-Z][a-zA-Z0-9-_]+$"`).
        - **Priorities:**
            1.  **Stable ID** (e.g., `submit-btn`)
            2.  **Test Attributes** (`data-testid`, `data-cy`, `aria-label`)
            3.  **Unique Name** (`name="username"`)
    - [ ] **Step 2 (Text Content):** If node is an Element and has `innerText`, generate Semantic XPath.
        - Format: `//{tag}[contains(text(), '{text}')]` (e.g., `//button[normalize-space()='Login']`).
    - [ ] **Step 3 (Path Generation):** Fallback: Traverse up the DOM tree (Parent traversing).
        - Start from the nearest parent with a Stable ID: `#parent-id > div > button`.
        - Hierarchical CSS as last resort.

- [ ] **Validation (Crucial):**
    - [ ] Test the generated selector in the live page: `document.querySelectorAll(selector).length`.
    - [ ] If result is not `1` (Unique), fallback to the next strategy or extend the path.

- [ ] **Input Analysis:** Automatically detect sensitive data fields (`type="password"`) and mark `LoginStep::Type` as `is_masked: true`.

**Blacklist (Never Use):**
- Random/Utility classes (e.g., Tailwind `w-full`, `p-4`).
- Hashed classes (e.g., `css-1x2y3z`).
- Dynamic IDs (ending in numbers or changing per session).
- Excessively long full paths (`html > body > div > ...`).

## ▶️ 5. Replayer Engine (`src/lsr/replayer.rs`)
*Playback logic and Orchestrator Cookie synchronization.*

- [ ] **Execution Logic**
    - [ ] Execute profile step-by-step.
    - [ ] Attempt Self-Healing (alternative selectors) if element is not found.
- [ ] **Session Sync**
    - [ ] On login success, fetch cookies (`Network.getCookies`).
    - [ ] Inject cookies into **Orchestrator**'s `SessionManager` or `CookieJar` so that traffic routed through the Proxy uses this authenticated session.

## 🧪 6. Testing Strategy
*Unit and Integration tests.*

- [ ] **Mock Server Integration**
    - [ ] Use `wiremock` to simulate a login form (`/login`).
- [ ] **Tests (`tests/lsr_tests.rs`)**
    - [ ] **HAR Import:** Verify correct `LoginProfile` generation from sample `.har`.
    - [ ] **Full Flow:** Record -> Replay -> Cookie Check loop on Mock Server.
    - [ ] **Workspace Check:** Verify `flow-engine` correctly uses types from `proxy-core` or `orchestrator`.

## 📚 7. Documentation
- [ ] **Architecture:** Architecture diagrams showing module placement and data flow.
- [ ] **Guide:** Guide on generating and importing profiles from HAR files.
