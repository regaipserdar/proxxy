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
      cpuUsage
      memoryUsageMb
      uptimeSeconds
      publicIp
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
  query GetHttpTransactions($agentId: String, $limit: Int, $offset: Int) {
    requests(agentId: $agentId, limit: $limit, offset: $offset) {
      requestId
      method
      url
      status
      timestamp
      agentId
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

export const GET_TRANSACTION_DETAILS = GET_REQUEST_DETAIL;

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
      publicIp
    }
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

// ============================================================================
// QUERIES - PROJECT MANAGEMENT
// ============================================================================

export const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      name
      path
      sizeBytes
      lastModified
      isActive
    }
  }
`;

export const GET_CA_CERT = gql`
  query GetCaCert {
    caCertPem
  }
`;

export const CREATE_PROJECT = gql`
  mutation CreateProject($name: String!) {
    createProject(name: $name) {
      success
      message
    }
  }
`;

export const LOAD_PROJECT = gql`
  mutation LoadProject($name: String!) {
    loadProject(name: $name) {
      success
      message
    }
  }
`;

export const DELETE_PROJECT = gql`
  mutation DeleteProject($name: String!) {
    deleteProject(name: $name) {
      success
      message
    }
  }
`;

export const EXPORT_PROJECT = gql`
  mutation ExportProject($name: String!, $outputPath: String!) {
    exportProject(name: $name, outputPath: $outputPath) {
      success
      message
    }
  }
`;

export const IMPORT_PROJECT = gql`
  mutation ImportProject($proxxyPath: String!, $projectName: String) {
    importProject(proxxyPath: $proxxyPath, projectName: $projectName) {
      success
      message
    }
  }
`;

export const UNLOAD_PROJECT = gql`
  mutation UnloadProject {
    unloadProject {
      success
      message
    }
  }
`;

// ============================================================================
// TARGET SCOPE OPERATIONS
// ============================================================================

export const GET_SCOPE_RULES = gql`
  query GetScopeRules {
    scopeRules {
      id
      ruleType
      pattern
      isRegex
      enabled
      createdAt
    }
  }
`;

export const ADD_SCOPE_RULE = gql`
  mutation AddScopeRule($ruleType: String!, $pattern: String!, $isRegex: Boolean!) {
    addScopeRule(ruleType: $ruleType, pattern: $pattern, isRegex: $isRegex) {
      id
      ruleType
      pattern
      isRegex
      enabled
      createdAt
    }
  }
`;

export const DELETE_SCOPE_RULE = gql`
  mutation DeleteScopeRule($id: String!) {
    deleteScopeRule(id: $id)
  }
`;

export const TOGGLE_SCOPE_RULE = gql`
  mutation ToggleScopeRule($id: String!, $enabled: Boolean!) {
    toggleScopeRule(id: $id, enabled: $enabled)
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

export const DELETE_REQUESTS_BY_HOST = gql`
  mutation DeleteRequestsByHost($host: String!) {
    deleteRequestsByHost(host: $host)
  }
`;

// ============================================================================
// REPEATER OPERATIONS
// ============================================================================

export const GET_REPEATER_TABS = gql`
  query GetRepeaterTabs {
    repeaterTabs {
      id
      name
      targetAgentId
      validationStatus
      isActive
      requestTemplate {
        method
        url
        body
        headers
      }
    }
  }
`;

export const EXECUTE_REPEATER_REQUEST = gql`
  mutation ExecuteRepeaterRequest($input: ExecuteRepeaterRequestInput!) {
    executeRepeaterRequest(input: $input) {
      id
      tabId
      agentId
      statusCode
      durationMs
      executedAt
      error
      requestData {
        method
        url
        body
        headers
      }
      responseData {
        statusCode
        body
        headers
        bodyLength
      }
    }
  }
`;

export const CREATE_REPEATER_TAB = gql`
  mutation CreateRepeaterTab($input: CreateRepeaterTabInput!) {
    createRepeaterTab(input: $input) {
      id
      name
      targetAgentId
    }
  }
`;

export const UPDATE_REPEATER_TAB = gql`
  mutation UpdateRepeaterTab($id: String!, $input: UpdateRepeaterTabInput!) {
    updateRepeaterTab(id: $id, input: $input) {
      id
      name
      targetAgentId
    }
  }
`;

export const DELETE_REPEATER_TAB = gql`
  mutation DeleteRepeaterTab($id: String!) {
    deleteRepeaterTab(id: $id)
  }
`;

export const GET_REPEATER_HISTORY = gql`
  query GetRepeaterHistory($tabId: String!, $limit: Int) {
    repeaterHistory(tabId: $tabId, limit: $limit) {
      id
      tabId
      agentId
      statusCode
      durationMs
      executedAt
      error
      requestData {
        method
        url
        body
        headers
      }
      responseData {
        statusCode
        body
        headers
        bodyLength
      }
    }
  }
`;

// ============================================================================
// SUBSCRIPTIONS - LIGHTWEIGHT (Real-time Updates)
// ============================================================================

// ============================================================================
// FLOW RECORDER OPERATIONS
// ============================================================================

export const GET_FLOW_PROFILES = gql`
  query GetFlowProfiles($status: String, $limit: Int) {
    flowProfiles(status: $status, limit: $limit) {
      id
      name
      flowType
      startUrl
      status
      stepCount
      createdAt
      updatedAt
      agentId
    }
  }
`;

export const GET_FLOW_PROFILE = gql`
  query GetFlowProfile($id: String!) {
    flowProfile(id: $id) {
      id
      name
      flowType
      startUrl
      steps
      meta
      status
      stepCount
      createdAt
      updatedAt
      agentId
    }
  }
`;

export const GET_FLOW_PROFILE_WITH_TRAFFIC = gql`
  query GetFlowProfileWithTraffic($id: String!) {
    flowProfileWithTraffic(id: $id) {
      profile {
        id
        name
        flowType
        startUrl
        steps
        meta
        status
        stepCount
        createdAt
        updatedAt
        agentId
      }
      traffic {
        id
        requestId
        method
        path
        host
        statusCode
        timestamp
      }
    }
  }
`;

export const GET_FLOW_EXECUTIONS = gql`
  query GetFlowExecutions($profileId: String!, $limit: Int) {
    flowExecutions(profileId: $profileId, limit: $limit) {
      id
      profileId
      agentId
      startedAt
      completedAt
      status
      errorMessage
      stepsCompleted
      totalSteps
      durationMs
    }
  }
`;

export const CREATE_FLOW_PROFILE = gql`
  mutation CreateFlowProfile($input: CreateFlowProfileInput!) {
    createFlowProfile(input: $input) {
      success
      message
      profileId
    }
  }
`;

export const UPDATE_FLOW_PROFILE = gql`
  mutation UpdateFlowProfile($id: String!, $input: UpdateFlowProfileInput!) {
    updateFlowProfile(id: $id, input: $input) {
      success
      message
      profileId
    }
  }
`;

export const DELETE_FLOW_PROFILE = gql`
  mutation DeleteFlowProfile($id: String!) {
    deleteFlowProfile(id: $id) {
      success
      message
      profileId
    }
  }
`;

export const START_FLOW_RECORDING = gql`
  mutation StartFlowRecording($input: StartRecordingInput!) {
    startFlowRecording(input: $input) {
      success
      message
      profileId
    }
  }
`;

export const STOP_FLOW_RECORDING = gql`
  mutation StopFlowRecording($profileId: String!, $input: StopRecordingInput!) {
    stopFlowRecording(profileId: $profileId, input: $input) {
      success
      message
      profileId
    }
  }
`;

export const REPLAY_FLOW = gql`
  mutation ReplayFlow($input: ReplayFlowInput!) {
    replayFlow(input: $input) {
      success
      executionId
      error
      sessionCookies
    }
  }
`;

export const GET_RECORDING_STATE = gql`
  query GetRecordingState {
    getRecordingState {
      profileId
      state
      eventCount
      currentUrl
      startedAt
      error
    }
  }
`;

// Debug mutations for browser testing
export const DEBUG_LAUNCH_BROWSER = gql`
  mutation DebugLaunchBrowser($startUrl: String!) {
    debugLaunchBrowser(startUrl: $startUrl) {
      success
      message
    }
  }
`;

export const DEBUG_CLOSE_BROWSER = gql`
  mutation DebugCloseBrowser {
    debugCloseBrowser {
      success
      message
    }
  }
`;

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
  subscription TrafficUpdates($agentId: String) {
    events(agentId: $agentId) {
      requestId
      method
      url
      status
      timestamp
      agentId
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