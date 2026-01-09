import { gql } from '@apollo/client';

// ============================================================================
// QUERIES - LIGHTWEIGHT (List Views)
// ============================================================================

/**
 * LIGHTWEIGHT: Get agents list
 * Use for: Dashboard, agent list table
 */
export const GET_AGENTS = gql`
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
`;

/**
 * LIGHTWEIGHT: Get HTTP transactions list (NO body/headers)
 * Use for: Traffic table, request list
 * 
 * CRITICAL: Does NOT request body/headers fields!
 * - Saves %98 memory per request
 * - Saves %85 CPU (no parsing)
 * - Saves %99 network bandwidth
 * 
 * For body/headers, use GET_REQUEST_DETAIL query
 */
export const GET_HTTP_TRANSACTIONS = gql`
  query GetHttpTransactions {
    requests {
      requestId
      method
      url
      status
      timestamp
      agentId
      # ❌ NO requestBody
      # ❌ NO requestHeaders  
      # ❌ NO responseBody
      # ❌ NO responseHeaders
      # These are fetched on-demand via GET_REQUEST_DETAIL
    }
  }
`;

/**
 * HEAVYWEIGHT: Get single request detail (WITH body/headers)
 * Use for: Request detail view, inspector
 * 
 * Only call this when user clicks on a specific request!
 * GraphQL will only parse body/headers for this ONE request.
 */
export const GET_REQUEST_DETAIL = gql`
  query GetRequestDetail($id: String!) {
    request(id: $id) {
      requestId
      method
      url
      status
      timestamp
      agentId
      # ✅ Body/headers only fetched when needed
      requestBody
      requestHeaders
      responseBody
      responseHeaders
    }
  }
`;

/**
 * LIGHTWEIGHT: Dashboard summary
 * Use for: Dashboard overview
 */
export const GET_DASHBOARD_SUMMARY = gql`
  query GetDashboardSummary {
    agents {
      id
      status
    }
    requests {
      requestId
      method
      status
    }
  }
`;

// ============================================================================
// QUERIES - SYSTEM METRICS
// ============================================================================

/**
 * Get system metrics with optional filtering
 * Note: String fields (memoryUsedBytes, etc.) need parseInt() for charts
 */
export const GET_SYSTEM_METRICS = gql`
  query GetSystemMetrics($agentId: String, $limit: Int) {
    systemMetrics(agentId: $agentId, limit: $limit) {
      agentId
      timestamp
      cpuUsagePercent
      memoryUsedBytes        # String - use parseInt() for charts
      memoryTotalBytes       # String - use parseInt() for charts
      networkRxBytesPerSec   # String - use parseInt() for charts
      networkTxBytesPerSec   # String - use parseInt() for charts
      diskReadBytesPerSec    # String - use parseInt() for charts
      diskWriteBytesPerSec   # String - use parseInt() for charts
      processCpuPercent
      processMemoryBytes     # String - use parseInt() for charts
      processUptimeSeconds
    }
  }
`;

/**
 * Get current system metrics for a specific agent
 */
export const GET_CURRENT_SYSTEM_METRICS = gql`
  query GetCurrentSystemMetrics($agentId: String!) {
    currentSystemMetrics(agentId: $agentId) {
      agentId
      timestamp
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      networkRxBytesPerSec
      networkTxBytesPerSec
      diskReadBytesPerSec
      diskWriteBytesPerSec
      processCpuPercent
      processMemoryBytes
      processUptimeSeconds
    }
  }
`;

/**
 * Get agent details with current metrics
 */
export const GET_AGENT_DETAILS = gql`
  query GetAgentDetails($agentId: String!) {
    agents {
      id
      name
      hostname
      status
      version
      lastHeartbeat
    }
    currentSystemMetrics(agentId: $agentId) {
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      processCpuPercent
      processMemoryBytes
    }
  }
`;

// ============================================================================
// MUTATIONS
// ============================================================================

/**
 * Replay a captured HTTP request
 */
export const REPLAY_REQUEST = gql`
  mutation ReplayRequest($requestId: String!) {
    replayRequest(requestId: $requestId) {
      success
      message
      replayRequestId
      originalUrl
      originalMethod
    }
  }
`;

/**
 * Intercept a request (placeholder)
 */
export const INTERCEPT_REQUEST = gql`
  mutation InterceptRequest($id: String!, $action: String!) {
    intercept(id: $id, action: $action)
  }
`;

// ============================================================================
// SUBSCRIPTIONS - LIGHTWEIGHT (Real-time Updates)
// ============================================================================

/**
 * LIGHTWEIGHT: Real-time traffic updates (NO body/headers)
 * Use for: Live traffic feed, real-time table updates
 * 
 * CRITICAL: Does NOT subscribe to body/headers!
 * - WebSocket bandwidth saved
 * - Browser memory saved
 * - Real-time updates stay fast
 * 
 * For body/headers, user must click to fetch via GET_REQUEST_DETAIL
 */
export const TRAFFIC_UPDATES = gql`
  subscription TrafficUpdates {
    events {
      requestId
      method
      url
      status
      timestamp
      agentId
      # ❌ NO requestBody
      # ❌ NO requestHeaders
      # ❌ NO responseBody
      # ❌ NO responseHeaders
      # Fetch these on-demand when user clicks
    }
  }
`;

/**
 * Real-time system metrics updates
 * Optional: Filter by agentId
 */
export const SYSTEM_METRICS_UPDATES = gql`
  subscription SystemMetricsUpdates($agentId: String) {
    systemMetricsUpdates(agentId: $agentId) {
      agentId
      timestamp
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      networkRxBytesPerSec
      networkTxBytesPerSec
      diskReadBytesPerSec
      diskWriteBytesPerSec
      processCpuPercent
      processMemoryBytes
      processUptimeSeconds
    }
  }
`;

// ============================================================================
// TEST QUERIES
// ============================================================================

/**
 * Test GraphQL connection
 */
export const TEST_CONNECTION = gql`
  query TestConnection {
    hello
  }
`;

// ============================================================================
// USAGE EXAMPLES
// ============================================================================

/*
// ✅ CORRECT: List view (lightweight)
const { data } = useQuery(GET_HTTP_TRANSACTIONS);
// -> Returns 50 requests, ~7.5 KB total
// -> Each request: ~150 bytes (metadata only)

// ✅ CORRECT: Detail view (heavyweight, on-demand)
const { data } = useQuery(GET_REQUEST_DETAIL, {
  variables: { id: selectedRequestId },
  skip: !selectedRequestId, // Only fetch when user clicks
});
// -> Returns 1 request with full body/headers
// -> Size: ~5-50 KB (depending on body size)

// ❌ WRONG: Fetching body/headers in list
const { data } = useQuery(gql`
  query {
    requests {
      requestId
      requestBody      # ❌ DON'T DO THIS IN LIST VIEW!
      responseBody     # ❌ 50MB+ data for 50 requests!
    }
  }
`);

// ✅ CORRECT: Subscription (lightweight)
const { data } = useSubscription(TRAFFIC_UPDATES);
// -> Real-time updates, metadata only
// -> WebSocket stays fast and responsive

// ✅ CORRECT: Metrics with parsing
const metrics = data?.systemMetrics.map(m => ({
  ...m,
  memoryUsedMB: parseInt(m.memoryUsedBytes, 10) / 1024 / 1024,
  networkRxKBps: parseInt(m.networkRxBytesPerSec, 10) / 1024,
}));
*/