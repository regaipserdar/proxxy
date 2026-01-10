# Changelog

All notable changes to Proxxy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1] - 2026-01-10

### Added
- **Project Management**: Complete project lifecycle management
  - Create, load, unload, and delete projects via GraphQL
  - Settings persistence per project (scope, interception rules)
  - Automatic settings load/reset on project switch
  
- **Project Import/Export**: `.proxxy` file format for backup/restore
  - Export projects to portable ZIP archives
  - Import projects from `.proxxy` files
  - Metadata support (name, export date, version)
  - GraphQL mutations: `exportProject`, `importProject`

- **Scope Management**: Advanced traffic filtering
  - Include/exclude pattern matching (glob and regex)
  - Configurable scope rules per project
  - Out-of-scope requests skip DB save and broadcast (but still proxy)
  - GraphQL API: `settings` query, `updateScope` mutation
  - Comprehensive pattern matching tests

- **Database Enhancements**
  - `project_settings` table for key-value configuration storage
  - Generic `get_setting<T>` / `save_setting<T>` methods
  - Scope and interception config persistence

- **Dependencies**
  - `glob = "0.3"` for pattern matching
  - `regex = "1.10"` for regex support
  - `zip = "2.2"` for .proxxy file format

### Changed
- Updated `async-graphql` to 7.1.0 (fixes MetaType::Scalar bug)
- Improved `load_project` to automatically load settings
- Enhanced `unload_project` to reset settings to defaults

### Fixed
- Build warnings removed (unused imports, dead code)
- Borrow checker issues in ZIP import/export
- SQLite error handling improvements

### Documentation
- Added comprehensive project management guide
- Added scope management documentation with examples
- Updated API reference with new mutations
- Added best practices and troubleshooting guides

### Known Issues
- GraphQL WebSocket temporarily disabled (awaiting axum 0.8 upgrade)
- Interception logic reserved for future implementation

## [1.1.0] - 2026-01-09

### Added
- Initial release with core proxy functionality
- Agent-based distributed architecture
- Real-time traffic monitoring
- GraphQL API
- SQLite persistence
- System metrics collection

[1.1.1]: https://github.com/regaipserdar/proxxy/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/regaipserdar/proxxy/releases/tag/v1.1.0
