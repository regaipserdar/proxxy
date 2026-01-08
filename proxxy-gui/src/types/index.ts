export interface AgentInfo {
    id: string;
    address: string;
    port: number;
    status: 'Online' | 'Offline';
    last_heartbeat: string;
    version: string;
    capabilities: string[];
}

export interface AgentsResponse {
    agents: AgentInfo[];
    total_count: number;
    online_count: number;
    offline_count: number;
}

export interface HealthStatus {
    status: string;
    uptime_seconds: number;
    database_connected: boolean;
}

export interface MetricsResponse {
    total_requests: number;
    average_response_time_ms: number;
    error_rate: number;
}

export interface TrafficResponse {
    transactions: HttpTransaction[];
    total_count: number;
}

export interface HttpTransaction {
    request_id: string;
    agent_id: string;
    method: string;
    url: string;
    status: number | null;
    timestamp: number;
}
