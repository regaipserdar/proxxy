# ⚙️ Phase 4: Project Configuration & Scope Management

**Goal:** Transform simple DB folders into fully configurable projects with Scope and Interception rules.

---

## 🗄️ 1. Database & Models

### Migration
- [x] Create `migrations/20240102_project_settings.sql`
  - `project_settings` table (Key-Value JSON store)
  - Default values for `scope` and `interception` keys

### Rust Models
- [x] Create `src/models/settings.rs`
  - [x] `ScopeConfig` struct (enabled, include/exclude patterns, regex flag)
  - [x] `InterceptionConfig` struct (enabled, rules array)
  - [x] `InterceptionRule` struct (id, name, condition, action)
  - [x] `RuleCondition` enum (Method, UrlContains, HeaderMatch, All)
  - [x] `RuleAction` enum (Pause, Drop, Modify)
- [x] Create `src/models/mod.rs` for exports

### Database Methods
- [x] Add `get_setting<T>()` generic method
- [x] Add `save_setting<T>()` generic method
- [x] Add `get_scope_config()` / `save_scope_config()`
- [x] Add `get_interception_config()` / `save_interception_config()`

---

## 🔄 2. AppState & Hot Reload

### State Management
- [x] Add `scope: Arc<RwLock<ScopeConfig>>` to AppState
- [x] Add `interception: Arc<RwLock<InterceptionConfig>>` to AppState
- [x] Load settings from DB on `load_project()`
- [x] Reset settings on `unload_project()`

### Project Import/Export
- [x] Add `.proxxy` file format (ZIP archive)
- [x] `export_project()` - Export project to .proxxy file
- [x] `import_project()` - Import project from .proxxy file
- [x] GraphQL mutations: `exportProject`, `importProject`

---

## 📡 3. GraphQL API

### Queries
- [x] `settings` → Returns `ProjectSettings` (scope + interception)
- [x] `ProjectSettings` GraphQL type

### Mutations
- [x] `updateScope(input: ScopeInput!)` → Updates DB + refreshes in-memory state
- [x] `toggleInterception(enabled: Boolean!)` → Master ON/OFF switch
- [x] `addInterceptionRule(rule: InterceptionRuleInput!)` → Adds new rule
- [x] `removeInterceptionRule(id: String!)` → Removes rule by ID

### Input Types
- [x] `ScopeInput` (enabled, includePatterns, excludePatterns, useRegex)
- [x] `InterceptionRuleInput` (name, condition, action)

---

## 🎯 4. Traffic Filtering Logic

### Scope Module
- [x] Create `src/scope.rs`
- [x] Implement `is_in_scope(config: &ScopeConfig, url: &str) -> bool`
- [x] Support glob patterns (using `glob` crate)
- [x] Optional regex support (using `regex` crate)

### Server Integration
- [x] Modify `stream_traffic()` in `server.rs`
- [x] Check scope BEFORE broadcasting/saving
- [x] Skip out-of-scope requests (no DB write, no broadcast)
- [x] Add debug logging for filtered requests

---

## 🎨 5. GUI Integration

### Settings Page
- [ ] Create `SettingsPage.tsx` component
- [ ] Route: `/settings`

### Scope Editor
- [ ] Textarea for include patterns (one per line)
- [ ] Textarea for exclude patterns (one per line)
- [ ] Toggle for regex mode
- [ ] "Save Scope" button

### Interception Controls
- [ ] Master toggle switch (ON/OFF)
- [ ] Rule list with enable/disable per rule
- [ ] "Add Rule" modal
- [ ] "Delete Rule" button

### Context Menu Integration
- [ ] "Add to Scope" option on traffic list rows
- [ ] "Exclude from Scope" option

---

## ✅ 6. Testing & Verification

### Unit Tests
- [ ] `scope_tests` - Pattern matching logic
- [ ] `settings_tests` - DB read/write

### Integration Tests
- [ ] GraphQL mutation tests
- [ ] End-to-end scope filtering test

### Manual Testing
- [ ] Load project, add scope patterns
- [ ] Verify filtered traffic doesn't appear in DB
- [ ] Verify UI updates reflect settings changes

---

## 📦 Crate Dependencies

```toml
# Cargo.toml additions
glob = "0.3"        # Pattern matching
regex = "1.10"      # Optional regex support
```

---

## 📁 New Files Summary

| Path | Description |
|------|-------------|
| `orchestrator/migrations/20240102_project_settings.sql` | Settings table |
| `orchestrator/src/models/mod.rs` | Models module |
| `orchestrator/src/models/settings.rs` | Config structs |
| `orchestrator/src/scope.rs` | Scope matching logic |
| `proxxy-gui/src/pages/SettingsPage.tsx` | Settings UI |