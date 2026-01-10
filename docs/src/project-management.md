# Project Management

Proxxy provides comprehensive project management capabilities, allowing you to organize your testing sessions, configure scope rules, and backup/restore projects.

## Overview

Each project in Proxxy is a self-contained workspace that includes:
- HTTP traffic history
- Agent configurations
- Scope and interception rules
- System metrics

## Creating a Project

### Via GraphQL

```graphql
mutation {
  createProject(name: "my-pentest") {
    success
    message
  }
}
```

### Via REST API

```bash
# Projects are created automatically when you load them
# See the load_project mutation below
```

## Loading a Project

When you load a project, all associated settings (scope, interception rules) are automatically loaded into memory:

```graphql
mutation {
  loadProject(name: "my-pentest") {
    success
    message  # "Project 'my-pentest' loaded with settings"
  }
}
```

## Unloading a Project

Unloading a project disconnects the database and resets all settings to defaults:

```graphql
mutation {
  unloadProject {
    success
    message  # "Project unloaded and settings reset"
  }
}
```

## Listing Projects

```graphql
query {
  projects {
    name
    path
    sizeBytes
    lastModified
    isActive
  }
}
```

## Deleting a Project

```graphql
mutation {
  deleteProject(name: "old-project") {
    success
    message
  }
}
```

## Project Import/Export

### Export to .proxxy File

Export a project to a portable `.proxxy` file (ZIP archive):

```graphql
mutation {
  exportProject(
    name: "my-pentest"
    outputPath: "/path/to/backup.proxxy"
  ) {
    success
    message
  }
}
```

The `.proxxy` file contains:
- **proxxy.db**: SQLite database with all traffic, agents, and settings
- **metadata.json**: Project metadata (name, export date, version)

### Import from .proxxy File

Import a project from a `.proxxy` backup:

```graphql
mutation {
  importProject(
    proxxyPath: "/path/to/backup.proxxy"
    projectName: "restored-project"  # Optional, uses original name if omitted
  ) {
    success
    message
  }
}
```

## .proxxy File Format

The `.proxxy` file is a ZIP archive with the following structure:

```
project.proxxy (ZIP)
├── proxxy.db          # SQLite database
└── metadata.json      # Project metadata
```

**metadata.json** example:
```json
{
  "name": "my-pentest",
  "exported_at": "2026-01-10T22:30:00Z",
  "version": "1.0"
}
```

## Best Practices

### Project Organization

- **One project per target**: Create separate projects for different applications or testing sessions
- **Descriptive names**: Use clear, descriptive project names (e.g., `acme-corp-api-2026-01`)
- **Regular exports**: Export projects regularly for backup and archival

### Storage Management

- **Monitor size**: Check project sizes regularly using the `projects` query
- **Archive old projects**: Export and delete old projects to save disk space
- **Cleanup**: Delete test/temporary projects after use

### Backup Strategy

```bash
# Example backup script
# Export all active projects weekly
for project in $(list_projects); do
  export_project "$project" "/backups/$project-$(date +%Y%m%d).proxxy"
done
```

## Project Directory Structure

Each project is stored in the workspace directory:

```
workspace/
├── project-1/
│   └── proxxy.db
├── project-2/
│   └── proxxy.db
└── project-3/
    └── proxxy.db
```

## Settings Persistence

Project settings are stored in the `project_settings` table within each project's database:

- **Scope configuration**: Include/exclude patterns, regex mode
- **Interception rules**: Traffic interception and modification rules

These settings are automatically loaded when you load a project and reset when you unload it.

## See Also

- [Scope Management](./scope-management.md)
- [Traffic Policy](./traffic-policy.md)
- [API Reference](./api-reference.md)
