// GraphQL Types for Proxxy GUI Integration

export interface Agent {
  id: string;
  name: string;
  hostname: string;
  status: string; // Backend returns string, not enum
  version: string;
  lastHeartbeat: string;
  cpuUsage?: number;
  memoryUsageMb?: number;
  uptimeSeconds?: number;
  publicIp?: string;
}

export interface HttpTransaction {
  requestId: string;
  method?: string;
  url?: string;
  status?: number;
  timestamp?: string;
  agentId?: string;
  requestHeaders?: string; // JSON string
  requestBody?: string;
  responseHeaders?: string; // JSON string
  responseBody?: string;
}

export interface SystemMetrics {
  agentId: string;
  timestamp: number; // Backend returns i64 timestamp
  cpuUsagePercent: number;
  memoryUsedBytes: string; // Backend returns as string
  memoryTotalBytes: string; // Backend returns as string
  networkRxBytesPerSec: string; // Backend field name
  networkTxBytesPerSec: string; // Backend field name
  diskReadBytesPerSec: string; // Backend field name
  diskWriteBytesPerSec: string; // Backend field name
  processCpuPercent: number;
  processMemoryBytes: string; // Backend returns as string
  processUptimeSeconds: number;
}

export interface TlsInfo {
  version: string;
  cipher: string;
  serverName?: string;
}

export interface ReplayResult {
  success: boolean;
  message: string;
  replayRequestId?: string; // Optional in backend
  originalUrl: string;
  originalMethod: string;
}

export interface TransactionFilters {
  method?: string;
  statusCode?: number;
  agentId?: string;
  timeRange?: [string, string];
  searchQuery?: string;
}

// GraphQL Operation Types
export interface GetAgentsQuery {
  agents: Agent[];
}

export interface GetHttpTransactionsQuery {
  requests: HttpTransaction[];
}

export interface GetSystemMetricsQuery {
  systemMetrics: SystemMetrics[];
}

export interface GetCurrentSystemMetricsQuery {
  currentSystemMetrics?: SystemMetrics;
}

export interface GetDashboardSummaryQuery {
  agents: Agent[];
  requests: HttpTransaction[];
}

export interface GetAgentDetailsQuery {
  agents: Agent[];
  currentSystemMetrics?: SystemMetrics;
}

export interface ReplayRequestMutation {
  replayRequest: ReplayResult;
}

export interface InterceptRequestMutation {
  intercept: boolean;
}

export interface TrafficUpdatesSubscription {
  events: HttpTransaction;
}

export interface SystemMetricsUpdatesSubscription {
  systemMetricsUpdates: SystemMetrics;
}