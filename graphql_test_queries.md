# Complete GraphQL Queries for Proxxy Orchestrator Testing

This document contains all the GraphQL queries and mutations you can use to test the Proxxy Orchestrator API and verify response accessibility.

## Basic Connectivity Test

### 1. Hello Query (Basic Test)
```graphql
query HelloTest {
  hello
}
```

**Expected Response:**
```json
{
  "data": {
    "hello": "Hello from Proxxy!"
  }
}
```

## Project Management Queries

### 2. List All Projects
```graphql
query GetProjects {
  projects {
    name
    isActive
    path
    createdAt
    lastModified
  }
}
```

**Expected Response:**
```json
{
  "data": {
    "projects": [
      {
        "name": "integration_test_project",
        "isActive": true,
        "path": "workspace/integration_test_project.proxxy",
        "createdAt": "2024-01-12T23:58:22Z",
        "lastModified": "2024-01-12T23:58:22Z"
      }
    ]
  }
}
```

### 3. Get Project Settings
```graphql
query GetProjectSettings {
  projectSettings {
    scope {
      enabled
      includePatterns
      excludePatterns
      useRegex
    }
    interception {
      enabled
      rules {
        id
        name
        enabled
        conditionType
        conditionValue
        actionType
        actionValue
      }
    }
  }
}
```

**Expected Response:**
```json
{
  "data": {
    "projectSettings": {
      "scope": {
        "enabled": false,
        "includePatterns": [],
        "excludePatterns": [],
        "useRegex": false
      },
      "interception": {
        "enabled": false,
        "rules": []
      }
    }
  }
}
```

## Agent Management Queries

### 4. List All Agents
```graphql
query GetAgents {
  agents {
    id
    name
    hostname
    status
    version
    lastHeartbeat
  }
}
```

**Expected Response:**
```json
{
  "data": {
    "agents": []
  }
}
```

## Traffic/Request Queries

### 5. List HTTP Transactions (Lightweight)
```graphql
query GetHttpTransactions {
  requests(agentId: null) {
    requestId
    method
    url
    status
    timestamp
    agentId
  }
}
```

**Expected Response (No Traffic):**
```json
{
  "data": {
    "requests": []
  }
}
```

**Expected Response (With Traffic):**
```json
{
  "data": {
    "requests": [
      {
        "requestId": "req_123456789",
        "method": "GET",
        "url": "http://httpbin.org/get",
        "status": 200,
        "timestamp": "2024-01-12T23:58:30Z",
        "agentId": "agent_001"
      }
    ]
  }
}
```

### 6. Get Request Detail (Heavyweight - includes body/headers)
```graphql
query GetRequestDetail($id: String!) {
  request(id: $id) {
    requestId
    method
    url
    status
    timestamp
    agentId
    requestHeaders
    requestBody
    responseHeaders
    responseBody
    duration
  }
}
```

**Variables:**
```json
{
  "id": "req_123456789"
}
```

**Expected Response:**
```json
{
  "data": {
    "request": {
      "requestId": "req_123456789",
      "method": "GET",
      "url": "http://httpbin.org/get",
      "status": 200,
      "timestamp": "2024-01-12T23:58:30Z",
      "agentId": "agent_001",
      "requestHeaders": "{\"User-Agent\": \"Proxxy-Integration-Test/1.0\", \"Host\": \"httpbin.org\"}",
      "requestBody": "",
      "responseHeaders": "{\"Content-Type\": \"application/json\", \"Content-Length\": \"294\"}",
      "responseBody": "{\"args\": {}, \"headers\": {\"Host\": \"httpbin.org\", \"User-Agent\": \"Proxxy-Integration-Test/1.0\"}, \"origin\": \"1.2.3.4\", \"url\": \"http://httpbin.org/get\"}",
      "duration": 150
    }
  }
}
```

### 7. System Metrics Query
```graphql
query GetSystemMetrics {
  systemMetrics {
    cpuUsage
    memoryUsage
    diskUsage
    networkConnections
    uptime
    timestamp
  }
}
```

## Project Management Mutations

### 8. Create Project
```graphql
mutation CreateProject($name: String!) {
  createProject(name: $name) {
    success
    message
  }
}
```

**Variables:**
```json
{
  "name": "test_project"
}
```

**Expected Response:**
```json
{
  "data": {
    "createProject": {
      "success": true,
      "message": "Project 'test_project' created"
    }
  }
}
```

### 9. Load Project
```graphql
mutation LoadProject($name: String!) {
  loadProject(name: $name) {
    success
    message
  }
}
```

**Variables:**
```json
{
  "name": "test_project"
}
```

**Expected Response:**
```json
{
  "data": {
    "loadProject": {
      "success": true,
      "message": "Project 'test_project' loaded with settings"
    }
  }
}
```

### 10. Delete Project
```graphql
mutation DeleteProject($name: String!) {
  deleteProject(name: $name) {
    success
    message
  }
}
```

**Variables:**
```json
{
  "name": "test_project"
}
```

**Expected Response:**
```json
{
  "data": {
    "deleteProject": {
      "success": true,
      "message": "Project 'test_project' deleted"
    }
  }
}
```

### 11. Unload Project
```graphql
mutation UnloadProject {
  unloadProject {
    success
    message
  }
}
```

**Expected Response:**
```json
{
  "data": {
    "unloadProject": {
      "success": true,
      "message": "Project unloaded and settings reset"
    }
  }
}
```

## Configuration Mutations

### 12. Toggle Interception
```graphql
mutation ToggleInterception($enabled: Boolean!) {
  toggleInterception(enabled: $enabled) {
    enabled
    rules {
      id
      name
      enabled
      conditionType
      conditionValue
      actionType
      actionValue
    }
  }
}
```

**Variables (Disable):**
```json
{
  "enabled": false
}
```

**Variables (Enable):**
```json
{
  "enabled": true
}
```

**Expected Response:**
```json
{
  "data": {
    "toggleInterception": {
      "enabled": false,
      "rules": []
    }
  }
}
```

### 13. Update Scope Configuration
```graphql
mutation UpdateScope($enabled: Boolean!, $includePatterns: [String!]!, $excludePatterns: [String!]!, $useRegex: Boolean!) {
  updateScope(enabled: $enabled, includePatterns: $includePatterns, excludePatterns: $excludePatterns, useRegex: $useRegex) {
    enabled
    includePatterns
    excludePatterns
    useRegex
  }
}
```

**Variables:**
```json
{
  "enabled": true,
  "includePatterns": ["*.example.com", "*.test.com"],
  "excludePatterns": ["*.ads.com"],
  "useRegex": false
}
```

**Expected Response:**
```json
{
  "data": {
    "updateScope": {
      "enabled": true,
      "includePatterns": ["*.example.com", "*.test.com"],
      "excludePatterns": ["*.ads.com"],
      "useRegex": false
    }
  }
}
```

## Request Replay Mutation

### 14. Replay Request
```graphql
mutation ReplayRequest($requestId: String!) {
  replayRequest(requestId: $requestId) {
    success
    message
    newRequestId
  }
}
```

**Variables:**
```json
{
  "requestId": "req_123456789"
}
```

**Expected Response:**
```json
{
  "data": {
    "replayRequest": {
      "success": true,
      "message": "Request replayed successfully",
      "newRequestId": "req_987654321"
    }
  }
}
```

## Complex Combined Query

### 15. Complete Status Query
```graphql
query CompleteStatus {
  hello
  projects {
    name
    isActive
    path
  }
  agents {
    id
    name
    status
    lastHeartbeat
  }
  requests(agentId: null) {
    requestId
    method
    url
    status
    timestamp
  }
  projectSettings {
    scope {
      enabled
      includePatterns
      excludePatterns
    }
    interception {
      enabled
      rules {
        id
        name
        enabled
      }
    }
  }
}
```

**Expected Response:**
```json
{
  "data": {
    "hello": "Hello from Proxxy!",
    "projects": [
      {
        "name": "integration_test_project",
        "isActive": true,
        "path": "workspace/integration_test_project.proxxy"
      }
    ],
    "agents": [],
    "requests": [],
    "projectSettings": {
      "scope": {
        "enabled": false,
        "includePatterns": [],
        "excludePatterns": []
      },
      "interception": {
        "enabled": false,
        "rules": []
      }
    }
  }
}
```

## Testing Instructions

### Using curl:
```bash
# Basic connectivity test
curl -X POST http://127.0.0.1:9090/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "query { hello }"}'

# List projects
curl -X POST http://127.0.0.1:9090/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "query { projects { name isActive path } }"}'

# Toggle interception off
curl -X POST http://127.0.0.1:9090/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "mutation { toggleInterception(enabled: false) { enabled } }"}'
```

### Using Python requests:
```python
import requests
import json

def test_graphql_query(query, variables=None):
    url = "http://127.0.0.1:9090/graphql"
    payload = {
        "query": query,
        "variables": variables or {}
    }
    
    response = requests.post(url, json=payload)
    return response.json()

# Test basic connectivity
result = test_graphql_query("query { hello }")
print(json.dumps(result, indent=2))

# Test project listing
result = test_graphql_query("""
query {
  projects {
    name
    isActive
    path
  }
}
""")
print(json.dumps(result, indent=2))
```

## Error Responses

### GraphQL Error Format:
```json
{
  "errors": [
    {
      "message": "Project 'nonexistent' not found",
      "locations": [
        {
          "line": 2,
          "column": 3
        }
      ],
      "path": ["loadProject"]
    }
  ],
  "data": null
}
```

### Common Error Cases:
1. **Invalid project name**: `Project 'invalid-name' not found`
2. **Database connection error**: `Failed to connect to database`
3. **Invalid GraphQL syntax**: `Syntax Error: Expected Name, found }`
4. **Missing required variables**: `Variable '$name' of required type 'String!' was not provided`

## Response Field Validation

### Critical Fields to Check:
1. **status**: Should NOT be null for completed requests
2. **responseHeaders**: Should NOT be null for captured responses
3. **responseBody**: Should NOT be null for captured responses
4. **projects[].isActive**: Should be true for loaded projects
5. **interception.enabled**: Should reflect the current state
6. **agents[].status**: Should show current agent status

These queries will help you thoroughly test the GraphQL API and verify that all response data is accessible and properly formatted.