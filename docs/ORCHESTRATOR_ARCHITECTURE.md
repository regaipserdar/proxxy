# Orchestrator Architecture & Data Flow

This document details the architecture of the **Proxxy Orchestrator**, specifically focusing on the recent optimizations (Lazy Loading), data flow, and exposed APIs.

## 1. System Overview

The Orchestrator acts as the central brain of the distributed proxy system. It handles:
1.  **Agent Communication (gRPC)**: Receiving traffic logs and system metrics from distributed agents.
2.  **Data Storage (SQLite)**: Persisting transactions, agent metadata, and metrics.
3.  **Frontend API (GraphQL)**: Serving data to the GUI with a performance-optimized "Lazy Loading" pattern.
4.  **Real-time Updates**: Broadcasting live events via GraphQL Subscriptions.

---

## 2. Recent Optimizations: Lazy Loading (GraphQL)

To solve performance issues when displaying thousands of HTTP transactions, we implemented a **Lazy Loading** pattern in the GraphQL layer.

### The Problem
Previously, fetching a list of requests (e.g., `reqs { id, url, body, headers }`) would force the server to serialise massive JSON objects and Base64-encode binary bodies for *every* item in the list, causing high CPU usage and network latency.

### The Solution
We split the `TrafficEvent` into "Lightweight" and "Heavyweight" fields.

*   **Lightweight (Always Available):** `id`, `method`, `url`, `status`, `timestamp`.
*   **Heavyweight (Computed on Demand):** `requestBody`, `requestHeaders`, `responseBody`, `responseHeaders`.

These heavyweight fields are implemented as **Complex Fields** in `async-graphql`. They are **only processed** if the client explicitly queries for them.

**Benefits:**
*   **List Views:** Query only lightweight fields. Zero cost for parsing/encoding bodies.
*   **Detail Views:** Query specific heavyweight fields for a single ID.

### GraphQL Schema Example

```graphql
type TrafficEventGql {
  # --- Lightweight Fields (Fast) ---
  requestId: String!
  method: String
  url: String
  status: Int
  timestamp: String
  agentId: String

  # --- Heavyweight Fields (Lazy Loaded) ---
  # Only computed if you ask for them!
  requestBody: String
  requestHeaders: String
  responseBody: String
  responseHeaders: String
}

type Query {
  # Optimized for list views (fetches last 50)
  requests: [TrafficEventGql!]!
  
  # Fetch full details for one request
  request(id: String!): TrafficEventGql
  
  systemMetrics(agentId: String, limit: Int): [SystemMetricsGql!]!
}
```

---

## 3. Data Flow

### A. Ingestion (Write Path)
**Source:** Proxy Agents via gRPC (`server.rs`)

1.  **Traffic Stream (`StreamTraffic`)**:
    *   Agent sends `TrafficEvent` via gRPC.
    *   **Structure**:
        ```protobuf
        message TrafficEvent {
          string request_id = 1;
          oneof event {
            HttpRequestData request = 2; // Method, URL, Headers, Body, TLS
            HttpResponseData response = 3; // Status, Headers, Body, TLS
            WebSocketFrame websocket = 4;
          }
        }
        ```
    *   Orchestrator immediately **Broadcasts** event to live subscribers (UI) via internal channel.
    *   Orchestrator **Asynchronously Saves** event to `http_transactions` table in SQLite.
        *   Requests and Responses typically arrive as separate events but share the same `request_id`.
        *   They are merged into a single database row.

2.  **Metrics Stream (`StreamMetrics`)**:
    *   Agent sends `SystemMetricsEvent` (CPU, RAM, Disk, Network).
    *   Orchestrator **Broadcasts** to UI.
    *   Orchestrator **Saves** to `system_metrics` table.

### B. Consumption (Read Path)
**Consumer:** Proxxy GUI via GraphQL (`graphql/mod.rs`)

1.  **Dashboard / List View**:
    *   **Query**:
        ```graphql
        query {
          requests {
            requestId
            method
            url
            status
          }
        }
        ```
    *   **Performance**: Extremely fast. Heavy body content is read from DB but **ignored** during GraphQL serialization.
    
2.  **Inspector / Detail View**:
    *   **Query**:
        ```graphql
        query {
            request(id: "123") {
                requestBody
                requestHeaders
                responseBody
                responseHeaders
            }
        }
        ```
    *   **Performance**: Slower (ms), but only runs for one item. Performs Base64 encoding/JSON serialization only for the requested ID.

---

## 4. API Reference

### Served (Outputs)
These are the APIs you can use to interact with the system.

*   **GraphQL API (`http://localhost:9090/graphql`)**:
    *   The primary interface for the frontend.
    *   Explore using the built-in Playground (visit URL in browser).
    *   Supports Queries, Mutations (e.g., Replay), and Subscriptions (Real-time).

*   **REST API**:
    *   Legacy/Simple endpoints.
    *   Mostly used for health checks (`/health/detailed`).

*   **gRPC Server (`localhost:50051`)**:
    *   Only for Agents.
    *   Defines `StreamTraffic` and `StreamMetrics` RPCs.

### Collected (Inputs)
What data do we actually have visibility into?

#### 1. HTTP Request Data
*   **Method**: GET, POST, PUT, DELETE, etc.
*   **URL**: Full URL including query parameters.
*   **Headers**: All HTTP request headers (User-Agent, Cookie, Authorization, etc.).
*   **Body**: Full binary body (up to configured limits).
*   **TLS Info**: Cypress suite, TLS version (e.g., TLS 1.3).

#### 2. HTTP Response Data
*   **Status Code**: 200, 404, 500, etc.
*   **Headers**: All response headers (Set-Cookie, Content-Type, etc.).
*   **Body**: Full binary response body.

#### 3. System Metrics
*   **CPU**: Usage percentage (System & Process).
*   **Memory**: Used vs Total bytes.
*   **Network**: Rx/Tx bytes per second.
*   **Disk**: Read/Write bytes per second, available space.

---

## 5. Capabilities (What can we do?)

### 1. Visualization
*   **Traffic Inspector**: View full details of any HTTP transaction.
*   **Live Tail**: Watch traffic flow in real-time multiple agents.
*   **System Dashboard**: Monitor health and resource usage of all connected agents.

### 2. Actionable Features
*   **Replay Request**:
    *   **Action**: You can select any past request and click "Replay".
    *   **Mechanism**:
        1.  Orchestrator fetches the original request details from DB.
        2.  It sends an `InterceptCommand` (specifically `ExecuteRequest`) to the original Agent via gRPC.
        3.  The Agent executes the HTTP request using its local network context.
        4.  The result is streamed back as a new `TrafficEvent`.
    *   **Use Case**: Debugging webhooks, testing API endpoints, retry logic verification.

*   **Interception (Planned)**:
    *   Ability to pause requests, edit headers/body on the fly, and then forward or drop.
    *   *Note: Backend proto supports `drop` and `modify` but UI/Logic implementation is in progress.*

---

## 6. Resilience & Reconnection Handling

Typically, distributed systems must handle network partitions. Here is how Proxxy handles it:

### Agent Behavior on Disconnection
When the connection to the Orchestrator is lost:
1.  **No Buffering (Current Implementation)**:
    *   Currently, if the gRPC stream is broken, the Agent tries to `reconnect` immediately.
    *   Any traffic events generated *during* the downtime are **dropped/lost** because the internal channel (`tx_stream`) will fail to send.
    *   The Agent prioritizes staying alive and handling traffic locally even if it cannot report it.
    *   *Reference: `proxy-agent/src/client.rs` -> `client.stream_traffic(req)` loop.*

2.  **Automatic Reconnection**:
    *   The Agent enters a retry loop with exponential backoff (starting at 1s, up to 60s).
    *   Once connected, it re-registers (`RegisterAgent`) and establishes new streams.

### Future Improvement: Local Buffering
To prevent data loss, a future update should implement a **Ring Buffer**:
*   Store up to N (e.g., 1000) events in memory when disconnected.
*   Flush them to the Orchestrator upon reconnection.
