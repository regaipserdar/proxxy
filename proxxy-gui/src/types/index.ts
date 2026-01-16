import { Node } from 'reactflow';

// --- Flow Engine Types ---
export enum NodeType {
    TRIGGER = 'trigger',     // Proxy Listener, Manual Start
    MATCHER = 'matcher',     // Regex, Status Code, Header Check
    MODIFIER = 'modifier',   // Set Header, Replace Body
    REPEATER = 'repeater',   // Fuzzing, Loops
    SINK = 'sink'            // Log, Save, Final Request
}

export interface NodeConfig {
    method?: string;
    pattern?: string;
    headerKey?: string;
    headerValue?: string;
    targetUrl?: string;
    iterations?: number;
    port?: number | string;
}

export interface ProxxyNodeData {
    label: string;
    subLabel?: string;
    type: NodeType;
    config: NodeConfig;
    status?: 'idle' | 'intercepting' | 'processing' | 'error';
}

export type ProxxyNode = Node<ProxxyNodeData>;

export interface DebugLog {
    timestamp: string;
    message: string;
    type: 'info' | 'error' | 'debug';
}

// --- Agent & System Types ---
export interface AgentInfo {
    id: string;
    address: string;
    port: number;
    name?: string;
    status: 'Online' | 'Offline';
    last_heartbeat: string;
    version: string;
    capabilities: string[];
    cpuUsage?: number;
    memoryUsageMb?: number;
    uptimeSeconds?: number;
    publicIp?: string;
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
    cpu_usage?: number;
    memory_usage?: number;
    network_load?: number;
}

export interface SystemMetrics {
    total_requests: number;
    requests_per_second?: number;
    average_response_time_ms: number;
    error_rate: number;
    active_agents?: number;
}

// --- Traffic & Scope Types ---
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

export interface ScopeRule {
    id: string;
    ruleType: string;
    pattern: string;
    isRegex: boolean;
    enabled: boolean;
    createdAt: number;
}

export interface Project {
    id: string;
    name: string;
    status: 'active' | 'idle';
    ruleCount: number;
}
