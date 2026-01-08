# Proxxy v1.1 - Backend API Specification for UI

## 📋 Overview
This document outlines all backend features available for the UI implementation. The backend provides both GraphQL and REST APIs for comprehensive proxy management.

---

## 🔌 API Endpoints

### Base URLs
- **GraphQL Endpoint:** `http://localhost:9090/graphql`
- **GraphQL Playground:** `http://localhost:9090/graphql` (Interactive)
- **REST API:** `http://localhost:9090/api/v1/*`
- **gRPC (Internal):** `http://localhost:50051`

---

## 📊 GraphQL API

### Queries

#### 1. **List Agents**
```graphql
query {
  agents {
    id
    name
    hostname
    status          # "Online" | "Offline"
    version
    lastHeartbeat   # ISO 8601 timestamp
  }
}
```

**Use Case:** Dashboard - Show all connected/disconnected agents

---

#### 2. **Get HTTP Transactions**
```graphql
query {
  httpTransactions(limit: 100) {
    requestId
    method
    url
    statusCode
    timestamp
    agentId
  }
}
```

**Use Case:** Traffic History - Display captured HTTP requests/responses

---

#### 3. **Get System Metrics**
```graphql
query {
  systemMetrics(agentId: "agent-123", limit: 60) {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    memoryTotalBytes
    networkRxBytes
    networkTxBytes
    diskReadBytes
    diskWriteBytes
    processCpuPercent
    processMemoryBytes
  }
}
```

**Use Case:** Agent Monitoring - Real-time system metrics charts

---

#### 4. **Get Current System Metrics**
```graphql
query {
  currentSystemMetrics(agentId: "agent-123") {
    cpuUsagePercent
    memoryUsedBytes
    # ... (same fields as systemMetrics)
  }
}
```

**Use Case:** Dashboard - Show current agent health status

---

### Mutations

#### 1. **Replay HTTP Request**
```graphql
mutation {
  replayRequest(requestId: "req-123") {
    success
    message
    replayRequestId
    originalUrl
    originalMethod
  }
}
```

**Use Case:** Repeater Tool - Re-send captured requests

---

#### 2. **Intercept Request** (Placeholder)
```graphql
mutation {
  intercept(id: "req-123", action: "drop") {
    success
  }
}
```

**Use Case:** Intercept Tool - Modify/drop requests in real-time

---

### Subscriptions

#### 1. **Real-time Traffic Updates**
```graphql
subscription {
  trafficUpdates {
    requestId
    method
    url
    statusCode
    timestamp
  }
}
```

**Use Case:** Live Traffic View - WebSocket stream of new requests

---

#### 2. **Real-time System Metrics**
```graphql
subscription {
  systemMetricsUpdates(agentId: "agent-123") {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    # ... (same fields)
  }
}
```

**Use Case:** Live Monitoring - Real-time charts

---

## 🎨 UI Components Needed

### 1. **Dashboard Page**
**Features:**
- [ ] Agent status cards (Online/Offline count)
- [ ] Recent traffic summary (total requests, methods breakdown)
- [ ] System health overview (CPU, Memory for all agents)
- [ ] Quick actions (Start/Stop agents, View logs)

**GraphQL Queries:**
- `agents` - Agent list with status
- `currentSystemMetrics` - Current health for each agent
- `httpTransactions(limit: 10)` - Recent traffic

---

### 2. **Agents Page**
**Features:**
- [ ] Agent list table (ID, Name, Status, Version, Last Seen)
- [ ] Agent detail view (System metrics, Configuration)
- [ ] Agent actions (Restart, Remove, Configure)
- [ ] Add new agent button

**GraphQL Queries:**
- `agents` - Full agent list
- `systemMetrics(agentId)` - Historical metrics for selected agent

**GraphQL Subscriptions:**
- `systemMetricsUpdates(agentId)` - Real-time metrics

---

### 3. **Traffic History Page**
**Features:**
- [ ] Request/Response list (Method, URL, Status, Time)
- [ ] Filters (Method, Status Code, Agent, Time Range)
- [ ] Search (URL, Headers, Body)
- [ ] Request detail view (Headers, Body, TLS info)
- [ ] Response detail view (Headers, Body)
- [ ] Export (JSON, HAR format)

**GraphQL Queries:**
- `httpTransactions(limit, filters)` - Traffic list

**GraphQL Subscriptions:**
- `trafficUpdates` - Live traffic stream

---

### 4. **Repeater Tool**
**Features:**
- [ ] Request editor (Method, URL, Headers, Body)
- [ ] Send button (triggers replay)
- [ ] Response viewer
- [ ] History of replayed requests
- [ ] Compare original vs replayed response

**GraphQL Queries:**
- `httpTransactions` - Get request to replay

**GraphQL Mutations:**
- `replayRequest(requestId)` - Trigger replay

---

### 5. **Intercept Tool** (Future)
**Features:**
- [ ] Live request stream
- [ ] Pause/Resume interception
- [ ] Modify request before forwarding
- [ ] Drop request
- [ ] Match & Replace rules

**GraphQL Mutations:**
- `intercept(id, action)` - Control interception

---

### 6. **System Metrics Page**
**Features:**
- [ ] Agent selector dropdown
- [ ] CPU usage chart (line chart, last 60 points)
- [ ] Memory usage chart
- [ ] Network I/O chart (RX/TX)
- [ ] Disk I/O chart
- [ ] Process metrics (CPU, Memory, Threads)
- [ ] Time range selector (1h, 6h, 24h)

**GraphQL Queries:**
- `systemMetrics(agentId, limit)` - Historical data

**GraphQL Subscriptions:**
- `systemMetricsUpdates(agentId)` - Real-time updates

---

## 🗄️ Database Schema (SQLite)

### Tables

#### `agents`
```sql
id TEXT PRIMARY KEY
name TEXT NOT NULL
hostname TEXT NOT NULL
version TEXT NOT NULL
status TEXT NOT NULL  -- 'Online' | 'Offline'
last_heartbeat INTEGER NOT NULL
```

#### `http_transactions`
```sql
request_id TEXT PRIMARY KEY
agent_id TEXT NOT NULL
req_method TEXT NOT NULL
req_url TEXT NOT NULL
req_headers TEXT  -- JSON
req_body BLOB
req_timestamp INTEGER NOT NULL
tls_info TEXT  -- JSON
res_status INTEGER
res_headers TEXT  -- JSON
res_body BLOB
res_timestamp INTEGER
```

#### `system_metrics`
```sql
id INTEGER PRIMARY KEY AUTOINCREMENT
agent_id TEXT NOT NULL
timestamp INTEGER NOT NULL
cpu_usage_percent REAL NOT NULL
memory_used_bytes INTEGER NOT NULL
memory_total_bytes INTEGER NOT NULL
network_rx_bytes INTEGER NOT NULL
network_tx_bytes INTEGER NOT NULL
disk_read_bytes INTEGER NOT NULL
disk_write_bytes INTEGER NOT NULL
process_cpu_percent REAL NOT NULL
process_memory_bytes INTEGER NOT NULL
```

---

## 🔐 Authentication (Future)

Currently, the API is **unauthenticated**. For production:
- [ ] Add JWT authentication
- [ ] Add API key support
- [ ] Add role-based access control (RBAC)

---

## 📡 WebSocket Support

GraphQL subscriptions use WebSocket protocol:
- **Endpoint:** `ws://localhost:9090/graphql`
- **Protocol:** GraphQL over WebSocket
- **Libraries:** Apollo Client, urql, or graphql-ws

---

## 🎯 UI Technology Recommendations

### Frontend Framework
- **React** + TypeScript (Recommended)
- **Next.js** for SSR/SSG
- **Vite** for fast development

### GraphQL Client
- **Apollo Client** (Full-featured, caching)
- **urql** (Lightweight, flexible)
- **graphql-request** (Simple, no caching)

### UI Components
- **shadcn/ui** (Tailwind-based, customizable)
- **Ant Design** (Enterprise-ready)
- **Material-UI** (Google Material Design)

### Charts
- **Recharts** (React-friendly)
- **Chart.js** (Popular, flexible)
- **Apache ECharts** (Feature-rich)

### State Management
- **Zustand** (Simple, minimal)
- **Redux Toolkit** (Complex apps)
- **Jotai** (Atomic state)

---

## 🚀 Quick Start for UI Development

### 1. Start Backend
```bash
# Terminal 1: Start Orchestrator
cargo run -p orchestrator

# Terminal 2: Start Agent
cargo run -p proxy-agent -- --name "Dev-Agent"
```

### 2. Test GraphQL API
Open browser: `http://localhost:9090/graphql`

Try this query:
```graphql
query {
  agents {
    id
    name
    status
  }
}
```

### 3. Test Subscription
```graphql
subscription {
  systemMetricsUpdates(agentId: "your-agent-id") {
    cpuUsagePercent
    memoryUsedBytes
  }
}
```

---

## 📝 Example UI Flows

### Flow 1: View Live Traffic
1. User opens "Traffic History" page
2. UI subscribes to `trafficUpdates`
3. New requests appear in real-time
4. User clicks a request → Detail view opens
5. User clicks "Replay" → `replayRequest` mutation
6. Response appears in Repeater tool

### Flow 2: Monitor Agent Health
1. User opens "Agents" page
2. UI queries `agents` → Shows list
3. User selects an agent
4. UI subscribes to `systemMetricsUpdates(agentId)`
5. Charts update in real-time (CPU, Memory, Network)

### Flow 3: Replay Request
1. User opens "Traffic History"
2. User selects a request
3. Request details populate Repeater tool
4. User modifies headers/body (optional)
5. User clicks "Send"
6. UI calls `replayRequest(requestId)`
7. Response appears below

---

## ✅ Backend Features Ready for UI

- [x] Agent registration and status tracking
- [x] HTTP traffic capture (Request/Response)
- [x] System metrics collection (CPU, Memory, Network, Disk)
- [x] GraphQL API (Queries, Mutations, Subscriptions)
- [x] Request replay functionality
- [x] Real-time traffic streaming
- [x] Real-time metrics streaming
- [x] Database persistence (SQLite)
- [x] Agent disconnect handling
- [x] TLS certificate management

---

## 🎨 UI Design Priorities

### Phase 1: Core Features (MVP)
1. **Dashboard** - Agent status + Traffic summary
2. **Traffic History** - List view with filters
3. **Agents** - List view with status

### Phase 2: Advanced Features
4. **Repeater Tool** - Request replay
5. **System Metrics** - Charts and monitoring
6. **Request Detail** - Full request/response viewer

### Phase 3: Power Features
7. **Intercept Tool** - Real-time modification
8. **Search & Filters** - Advanced querying
9. **Export** - HAR, JSON export

---

## 🔗 Related Documentation

- [GraphQL Schema](./orchestrator/src/graphql/mod.rs)
- [Database Schema](./orchestrator/migrations/20240101_init.sql)
- [Proto Definitions](./proto/proxy.proto)
- [README](./README.md)

---

**Version:** 1.1.0  
**Last Updated:** 2026-01-08  
**Status:** ✅ Ready for UI Development
