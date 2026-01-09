import { gql } from '@apollo/client';

// Queries
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

export const GET_HTTP_TRANSACTIONS = gql`
  query GetHttpTransactions {
    requests {
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
    }
  }
`;

export const GET_SYSTEM_METRICS = gql`
  query GetSystemMetrics($agentId: String, $limit: Int) {
    systemMetrics(agentId: $agentId, limit: $limit) {
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

// Test query for connection verification
export const TEST_CONNECTION = gql`
  query TestConnection {
    hello
  }
`;

// Mutations
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

export const INTERCEPT_REQUEST = gql`
  mutation InterceptRequest($id: String!, $action: String!) {
    intercept(id: $id, action: $action)
  }
`;

// Subscriptions
export const TRAFFIC_UPDATES = gql`
  subscription TrafficUpdates {
    events {
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
    }
  }
`;

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

// Additional utility queries for dashboard and monitoring
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