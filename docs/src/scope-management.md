# Scope Management

Scope management allows you to define which URLs should be recorded and displayed in Proxxy. This is essential for focusing on your target application and filtering out noise from third-party services.

## Overview

The scope system in Proxxy:
- ✅ **Filters recording**: Out-of-scope requests are not saved to the database
- ✅ **Filters UI**: Out-of-scope requests are not broadcast to the GUI
- ✅ **Preserves proxy flow**: Out-of-scope requests still pass through the proxy normally

**Important**: Scope filtering only affects what is *recorded*, not what is *proxied*. All requests continue to flow through the proxy regardless of scope.

## Scope Configuration

### Get Current Scope Settings

```graphql
query {
  settings {
    scope {
      enabled
      includePatterns
      excludePatterns
      useRegex
    }
  }
}
```

### Update Scope Settings

```graphql
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["*.example.com", "api.target.com"]
    excludePatterns: ["*.google.com", "*.facebook.com"]
    useRegex: false
  }) {
    enabled
    includePatterns
    excludePatterns
    useRegex
  }
}
```

## Pattern Matching

### Glob Patterns (Default)

When `useRegex: false`, patterns use glob syntax:

| Pattern | Matches | Examples |
|---------|---------|----------|
| `*.example.com` | Any subdomain | `api.example.com`, `www.example.com` |
| `example.com` | Exact match | `example.com` only |
| `*example*` | Contains | `myexample.com`, `example-api.com` |
| `api.*.com` | Wildcard in middle | `api.target.com`, `api.test.com` |

**Examples:**
```graphql
# Include all subdomains of example.com
includePatterns: ["*.example.com", "example.com"]

# Exclude CDNs and analytics
excludePatterns: ["*.cloudflare.com", "*.google-analytics.com"]
```

### Regex Patterns

When `useRegex: true`, patterns use regular expressions:

```graphql
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["^api\\.(staging|prod)\\.example\\.com$"]
    excludePatterns: ["\\.(css|js|png|jpg|gif)$"]
    useRegex: true
  }) {
    enabled
  }
}
```

**Regex Examples:**

| Pattern | Matches |
|---------|---------|
| `^api\\.example\\.com$` | Exact domain |
| `^.*\\.example\\.com$` | All subdomains |
| `^(api\|www)\\.example\\.com$` | Specific subdomains |
| `\\.(jpg\|png\|gif)$` | Image files |

## Scope Rules

### Rule Priority

1. **Excludes first**: If a URL matches an exclude pattern, it's out of scope (regardless of includes)
2. **Includes second**: If no excludes match, check include patterns
3. **Empty includes**: If include list is empty, everything is in scope (except excludes)

### Behavior Matrix

| Scope Enabled | Include Patterns | Exclude Patterns | URL Matches | Result |
|---------------|------------------|------------------|-------------|--------|
| `false` | Any | Any | Any | ✅ In scope |
| `true` | Empty | Empty | Any | ✅ In scope |
| `true` | `["*.example.com"]` | Empty | `api.example.com` | ✅ In scope |
| `true` | `["*.example.com"]` | Empty | `google.com` | ❌ Out of scope |
| `true` | `["*.example.com"]` | `["cdn.example.com"]` | `cdn.example.com` | ❌ Out of scope (exclude wins) |
| `true` | `["*.example.com"]` | `["cdn.example.com"]` | `api.example.com` | ✅ In scope |

## Common Use Cases

### Single Target Application

```graphql
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["*.target-app.com", "target-app.com"]
    excludePatterns: []
    useRegex: false
  })
}
```

### Multiple Domains

```graphql
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: [
      "*.example.com",
      "*.partner-api.com",
      "legacy.oldapp.net"
    ]
    excludePatterns: []
    useRegex: false
  })
}
```

### Exclude Third-Party Services

```graphql
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["*.myapp.com"]
    excludePatterns: [
      "*.google.com",
      "*.facebook.com",
      "*.cloudflare.com",
      "*.googleapis.com",
      "*.gstatic.com"
    ]
    useRegex: false
  })
}
```

### Development vs Production

```graphql
# Development environment
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["localhost:*", "*.dev.example.com"]
    excludePatterns: []
    useRegex: false
  })
}

# Production environment (regex)
mutation {
  updateScope(input: {
    enabled: true
    includePatterns: ["^(api|www)\\.example\\.com$"]
    excludePatterns: []
    useRegex: true
  })
}
```

## Disabling Scope

To record all traffic:

```graphql
mutation {
  updateScope(input: {
    enabled: false
    includePatterns: []
    excludePatterns: []
    useRegex: false
  })
}
```

## URL Extraction

Scope matching is performed on the **host** portion of the URL:

| Full URL | Extracted Host | Matched Against |
|----------|----------------|-----------------|
| `https://api.example.com/users` | `api.example.com` | Patterns |
| `http://localhost:8080/test` | `localhost` | Patterns |
| `https://sub.domain.example.com:443/path` | `sub.domain.example.com` | Patterns |

**Note**: Port numbers are stripped before matching (unless included in the pattern).

## Performance Considerations

- **Glob is faster**: Use glob patterns when possible for better performance
- **Minimize patterns**: Keep pattern lists concise
- **Specific patterns**: Use specific patterns rather than broad wildcards

## Debugging

### Check if URL is in scope

The scope check happens in the orchestrator's `stream_traffic` method. Debug logs show:

```
⏭️ Out-of-scope (not recording): https://cdn.example.com/script.js
```

### Verify settings

```graphql
query {
  settings {
    scope {
      enabled
      includePatterns
      excludePatterns
      useRegex
    }
  }
}
```

## Best Practices

1. **Start broad, refine later**: Begin with a broad scope, then add excludes as needed
2. **Test patterns**: Use the GraphQL playground to test pattern changes
3. **Document patterns**: Keep notes on why specific patterns were added
4. **Review regularly**: Periodically review and clean up unused patterns

## See Also

- [Project Management](./project-management.md)
- [Traffic Policy](./traffic-policy.md)
- [API Reference](./api-reference.md)
