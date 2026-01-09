# API Kullanım Örnekleri

Bu dokümantasyon, Proxxy Orchestrator API'sini kullanarak yaygın görevlerin nasıl gerçekleştirileceğini gösteren pratik örnekler içerir.

---

## 📋 İçindekiler

1. [REST API Örnekleri](#rest-api-örnekleri)
2. [GraphQL Örnekleri](#graphql-örnekleri)
3. [JavaScript/TypeScript Örnekleri](#javascripttypescript-örnekleri)
4. [Python Örnekleri](#python-örnekleri)
5. [React Hooks Örnekleri](#react-hooks-örnekleri)

---

# REST API Örnekleri

## 1. Sistem Durumunu Kontrol Etme

### Bash/cURL
```bash
#!/bin/bash

# Basit health check
curl http://localhost:9090/health

# Detaylı health check
curl http://localhost:9090/api/health/detailed

# Sistem sağlığı ve agent istatistikleri
curl http://localhost:9090/api/system/health | jq '.'
```

### JavaScript (Fetch API)
```javascript
async function checkHealth() {
  try {
    const response = await fetch('http://localhost:9090/api/system/health');
    const data = await response.json();
    
    console.log('System Status:', data.status);
    console.log('Uptime:', data.uptime_seconds, 'seconds');
    console.log('Agents Online:', data.agents_online, '/', data.agents_total);
    
    return data;
  } catch (error) {
    console.error('Health check failed:', error);
  }
}

checkHealth();
```

### Python
```python
import requests
import json

def check_health():
    try:
        response = requests.get('http://localhost:9090/api/system/health')
        data = response.json()
        
        print(f"System Status: {data['status']}")
        print(f"Uptime: {data['uptime_seconds']} seconds")
        print(f"Agents Online: {data['agents_online']}/{data['agents_total']}")
        
        return data
    except Exception as e:
        print(f"Health check failed: {e}")

check_health()
```

---

## 2. Agent Listesini Alma

### Bash/cURL
```bash
#!/bin/bash

# Tüm agent'ları listele
curl http://localhost:9090/api/agents | jq '.'

# Sadece online agent'ları göster
curl http://localhost:9090/api/agents | jq '.agents[] | select(.status == "Online")'

# Agent sayılarını göster
curl http://localhost:9090/api/agents | jq '{total: .total_count, online: .online_count, offline: .offline_count}'
```

### JavaScript
```javascript
async function listAgents() {
  const response = await fetch('http://localhost:9090/api/agents');
  const data = await response.json();
  
  console.log(`Total Agents: ${data.total_count}`);
  console.log(`Online: ${data.online_count}, Offline: ${data.offline_count}`);
  
  data.agents.forEach(agent => {
    console.log(`\n${agent.id}:`);
    console.log(`  Status: ${agent.status}`);
    console.log(`  Address: ${agent.address}:${agent.port}`);
    console.log(`  Version: ${agent.version}`);
    console.log(`  Last Heartbeat: ${agent.last_heartbeat}`);
  });
  
  return data;
}

listAgents();
```

### Python
```python
import requests
from datetime import datetime

def list_agents():
    response = requests.get('http://localhost:9090/api/agents')
    data = response.json()
    
    print(f"Total Agents: {data['total_count']}")
    print(f"Online: {data['online_count']}, Offline: {data['offline_count']}\n")
    
    for agent in data['agents']:
        print(f"{agent['id']}:")
        print(f"  Status: {agent['status']}")
        print(f"  Address: {agent['address']}:{agent['port']}")
        print(f"  Version: {agent['version']}")
        print(f"  Last Heartbeat: {agent['last_heartbeat']}\n")
    
    return data

list_agents()
```

---

## 3. Yeni Agent Kaydetme

### Bash/cURL
```bash
#!/bin/bash

curl -X POST http://localhost:9090/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-prod-01",
    "address": "192.168.1.100",
    "port": 8080,
    "version": "0.1.1",
    "capabilities": ["http", "https", "websocket"]
  }' | jq '.'
```

### JavaScript
```javascript
async function registerAgent(agentConfig) {
  const response = await fetch('http://localhost:9090/agents', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(agentConfig),
  });
  
  const data = await response.json();
  
  if (data.success) {
    console.log('✅ Agent registered successfully:', data.agent_id);
  } else {
    console.error('❌ Registration failed:', data.message);
  }
  
  return data;
}

// Kullanım
registerAgent({
  agent_id: 'agent-prod-01',
  address: '192.168.1.100',
  port: 8080,
  version: '0.1.1',
  capabilities: ['http', 'https', 'websocket']
});
```

### Python
```python
import requests

def register_agent(agent_config):
    response = requests.post(
        'http://localhost:9090/agents',
        json=agent_config
    )
    data = response.json()
    
    if data['success']:
        print(f"✅ Agent registered successfully: {data['agent_id']}")
    else:
        print(f"❌ Registration failed: {data['message']}")
    
    return data

# Kullanım
register_agent({
    'agent_id': 'agent-prod-01',
    'address': '192.168.1.100',
    'port': 8080,
    'version': '0.1.1',
    'capabilities': ['http', 'https', 'websocket']
})
```

---

## 4. Trafik Verilerini Alma

### Bash/cURL
```bash
#!/bin/bash

# Son 50 HTTP işlemini getir
curl http://localhost:9090/api/traffic/recent | jq '.transactions[] | {method, url, status}'

# Belirli bir metodu filtrele
curl http://localhost:9090/api/traffic/recent | jq '.transactions[] | select(.method == "POST")'

# Hata olan istekleri göster (4xx, 5xx)
curl http://localhost:9090/api/traffic/recent | jq '.transactions[] | select(.status >= 400)'
```

### JavaScript
```javascript
async function getRecentTraffic() {
  const response = await fetch('http://localhost:9090/api/traffic/recent');
  const data = await response.json();
  
  console.log(`Total Transactions: ${data.total_count}\n`);
  
  data.transactions.forEach(tx => {
    const statusEmoji = tx.status >= 400 ? '❌' : '✅';
    console.log(`${statusEmoji} ${tx.method} ${tx.url}`);
    console.log(`   Status: ${tx.status}, Agent: ${tx.agent_id}`);
    console.log(`   Time: ${new Date(tx.timestamp * 1000).toISOString()}\n`);
  });
  
  return data;
}

getRecentTraffic();
```

### Python
```python
import requests
from datetime import datetime

def get_recent_traffic():
    response = requests.get('http://localhost:9090/api/traffic/recent')
    data = response.json()
    
    print(f"Total Transactions: {data['total_count']}\n")
    
    for tx in data['transactions']:
        status_emoji = '❌' if tx['status'] >= 400 else '✅'
        print(f"{status_emoji} {tx['method']} {tx['url']}")
        print(f"   Status: {tx['status']}, Agent: {tx['agent_id']}")
        
        timestamp = datetime.fromtimestamp(tx['timestamp'])
        print(f"   Time: {timestamp.isoformat()}\n")
    
    return data

get_recent_traffic()
```

---

## 5. Metrikleri Alma

### Bash/cURL
```bash
#!/bin/bash

# Sistem geneli metrikler
curl http://localhost:9090/metrics | jq '.'

# Belirli bir agent'ın metrikleri
AGENT_ID="agent-001"
curl http://localhost:9090/metrics/$AGENT_ID | jq '.'

# Hata oranını hesapla
curl http://localhost:9090/metrics | jq '.error_rate * 100 | "\(.)%"'
```

### JavaScript
```javascript
async function getMetrics(agentId = null) {
  const url = agentId 
    ? `http://localhost:9090/metrics/${agentId}`
    : 'http://localhost:9090/metrics';
    
  const response = await fetch(url);
  const data = await response.json();
  
  if (agentId) {
    console.log(`Metrics for Agent: ${data.agent_id}`);
    console.log(`Requests Handled: ${data.requests_handled}`);
    console.log(`Avg Response Time: ${data.average_response_time_ms.toFixed(2)}ms`);
    console.log(`Error Rate: ${(data.error_rate * 100).toFixed(2)}%`);
    console.log(`Memory Usage: ${(data.memory_usage_mb).toFixed(2)}MB`);
    console.log(`CPU Usage: ${data.cpu_usage_percent.toFixed(2)}%`);
  } else {
    console.log('System-Wide Metrics:');
    console.log(`Total Requests: ${data.total_requests}`);
    console.log(`Avg Response Time: ${data.average_response_time_ms.toFixed(2)}ms`);
    console.log(`Error Rate: ${(data.error_rate * 100).toFixed(2)}%`);
  }
  
  return data;
}

// Sistem metrikleri
getMetrics();

// Agent metrikleri
getMetrics('agent-001');
```

---

# GraphQL Örnekleri

## 1. Agent Listesini Sorgulama

### GraphQL Query
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

### JavaScript (Fetch)
```javascript
async function queryAgents() {
  const query = `
    query {
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
  
  const response = await fetch('http://localhost:9090/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ query }),
  });
  
  const { data } = await response.json();
  return data.agents;
}

queryAgents().then(agents => {
  console.log('Agents:', agents);
});
```

### Python (requests)
```python
import requests

def query_agents():
    query = """
    query {
      agents {
        id
        name
        hostname
        status
        version
        lastHeartbeat
      }
    }
    """
    
    response = requests.post(
        'http://localhost:9090/graphql',
        json={'query': query}
    )
    
    data = response.json()
    return data['data']['agents']

agents = query_agents()
print('Agents:', agents)
```

---

## 2. HTTP İşlemlerini Sorgulama

### GraphQL Query
```graphql
query GetTransactions($limit: Int!) {
  httpTransactions(limit: $limit) {
    requestId
    method
    url
    statusCode
    timestamp
    agentId
  }
}
```

### JavaScript
```javascript
async function queryTransactions(limit = 50) {
  const query = `
    query GetTransactions($limit: Int!) {
      httpTransactions(limit: $limit) {
        requestId
        method
        url
        statusCode
        timestamp
        agentId
      }
    }
  `;
  
  const response = await fetch('http://localhost:9090/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query,
      variables: { limit },
    }),
  });
  
  const { data } = await response.json();
  return data.httpTransactions;
}

queryTransactions(100).then(transactions => {
  console.log(`Found ${transactions.length} transactions`);
  transactions.forEach(tx => {
    console.log(`${tx.method} ${tx.url} - ${tx.statusCode}`);
  });
});
```

---

## 3. İstek Tekrarlama (Mutation)

### GraphQL Mutation
```graphql
mutation ReplayRequest($requestId: String!) {
  replayRequest(requestId: $requestId) {
    success
    message
    replayRequestId
    originalUrl
    originalMethod
  }
}
```

### JavaScript
```javascript
async function replayRequest(requestId) {
  const mutation = `
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
  
  const response = await fetch('http://localhost:9090/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query: mutation,
      variables: { requestId },
    }),
  });
  
  const { data } = await response.json();
  
  if (data.replayRequest.success) {
    console.log('✅ Request replayed successfully');
    console.log('Original:', data.replayRequest.originalMethod, data.replayRequest.originalUrl);
    console.log('Replay ID:', data.replayRequest.replayRequestId);
  } else {
    console.error('❌ Replay failed:', data.replayRequest.message);
  }
  
  return data.replayRequest;
}

replayRequest('req-12345');
```

---

## 4. Gerçek Zamanlı Trafik İzleme (Subscription)

### GraphQL Subscription
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

### JavaScript (graphql-ws)
```javascript
import { createClient } from 'graphql-ws';

const client = createClient({
  url: 'ws://localhost:9090/graphql',
});

const subscription = client.subscribe(
  {
    query: `
      subscription {
        trafficUpdates {
          requestId
          method
          url
          statusCode
          timestamp
        }
      }
    `,
  },
  {
    next: (data) => {
      const tx = data.data.trafficUpdates;
      console.log(`🔔 New Request: ${tx.method} ${tx.url} - ${tx.statusCode}`);
    },
    error: (error) => {
      console.error('❌ Subscription error:', error);
    },
    complete: () => {
      console.log('✅ Subscription completed');
    },
  }
);

// Aboneliği iptal etmek için
// subscription.unsubscribe();
```

---

## 5. Gerçek Zamanlı Metrik İzleme

### GraphQL Subscription
```graphql
subscription SystemMetrics($agentId: String!) {
  systemMetricsUpdates(agentId: $agentId) {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    networkRxBytes
    networkTxBytes
  }
}
```

### JavaScript
```javascript
import { createClient } from 'graphql-ws';

function subscribeToMetrics(agentId) {
  const client = createClient({
    url: 'ws://localhost:9090/graphql',
  });

  return client.subscribe(
    {
      query: `
        subscription SystemMetrics($agentId: String!) {
          systemMetricsUpdates(agentId: $agentId) {
            agentId
            timestamp
            cpuUsagePercent
            memoryUsedBytes
            networkRxBytes
            networkTxBytes
          }
        }
      `,
      variables: { agentId },
    },
    {
      next: (data) => {
        const metrics = data.data.systemMetricsUpdates;
        console.log(`📊 Metrics Update for ${metrics.agentId}:`);
        console.log(`   CPU: ${metrics.cpuUsagePercent.toFixed(2)}%`);
        console.log(`   Memory: ${(metrics.memoryUsedBytes / 1024 / 1024).toFixed(2)}MB`);
        console.log(`   Network RX: ${(metrics.networkRxBytes / 1024).toFixed(2)}KB`);
        console.log(`   Network TX: ${(metrics.networkTxBytes / 1024).toFixed(2)}KB`);
      },
      error: (error) => console.error('Error:', error),
      complete: () => console.log('Subscription completed'),
    }
  );
}

const unsubscribe = subscribeToMetrics('agent-001');
```

---

# JavaScript/TypeScript Örnekleri

## 1. API Client Sınıfı

```typescript
class ProxxyClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://localhost:9090') {
    this.baseUrl = baseUrl;
  }
  
  // Health Check
  async checkHealth(): Promise<HealthStatus> {
    const response = await fetch(`${this.baseUrl}/api/system/health`);
    return response.json();
  }
  
  // List Agents
  async listAgents(): Promise<AgentsResponse> {
    const response = await fetch(`${this.baseUrl}/api/agents`);
    return response.json();
  }
  
  // Get Specific Agent
  async getAgent(agentId: string): Promise<AgentInfo> {
    const response = await fetch(`${this.baseUrl}/agents/${agentId}`);
    if (!response.ok) {
      throw new Error(`Agent not found: ${agentId}`);
    }
    return response.json();
  }
  
  // Register Agent
  async registerAgent(config: RegisterAgentRequest): Promise<RegisterAgentResponse> {
    const response = await fetch(`${this.baseUrl}/agents`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return response.json();
  }
  
  // Get Recent Traffic
  async getRecentTraffic(): Promise<TrafficResponse> {
    const response = await fetch(`${this.baseUrl}/api/traffic/recent`);
    return response.json();
  }
  
  // Get Metrics
  async getMetrics(agentId?: string): Promise<MetricsResponse> {
    const url = agentId 
      ? `${this.baseUrl}/metrics/${agentId}`
      : `${this.baseUrl}/metrics`;
    const response = await fetch(url);
    return response.json();
  }
  
  // GraphQL Query
  async graphql<T>(query: string, variables?: any): Promise<T> {
    const response = await fetch(`${this.baseUrl}/graphql`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, variables }),
    });
    const { data, errors } = await response.json();
    if (errors) {
      throw new Error(errors[0].message);
    }
    return data;
  }
}

// Kullanım
const client = new ProxxyClient();

// Health check
const health = await client.checkHealth();
console.log('System status:', health.status);

// List agents
const agents = await client.listAgents();
console.log('Total agents:', agents.total_count);

// GraphQL query
const data = await client.graphql(`
  query {
    agents {
      id
      name
      status
    }
  }
`);
```

## 2. TypeScript Tip Tanımları

```typescript
interface HealthStatus {
  status: string;
  uptime_seconds: number;
  database_connected: boolean;
  agents_online?: number;
  agents_total?: number;
}

interface AgentInfo {
  id: string;
  address: string;
  port: number;
  status: 'Online' | 'Offline';
  last_heartbeat: string;
  version: string;
  capabilities: string[];
}

interface AgentsResponse {
  agents: AgentInfo[];
  total_count: number;
  online_count: number;
  offline_count: number;
}

interface RegisterAgentRequest {
  agent_id: string;
  address: string;
  port: number;
  version: string;
  capabilities: string[];
}

interface RegisterAgentResponse {
  success: boolean;
  message: string;
  agent_id: string;
}

interface HttpTransaction {
  request_id: string;
  agent_id: string;
  method: string;
  url: string;
  status: number | null;
  timestamp: number;
}

interface TrafficResponse {
  transactions: HttpTransaction[];
  total_count: number;
}

interface MetricsResponse {
  total_requests: number;
  average_response_time_ms: number;
  error_rate: number;
}

interface AgentMetrics extends MetricsResponse {
  agent_id: string;
  timestamp: string;
  requests_handled: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
}
```

---

# React Hooks Örnekleri

## 1. useAgents Hook

```typescript
import { useState, useEffect } from 'react';

interface UseAgentsResult {
  agents: AgentInfo[];
  loading: boolean;
  error: Error | null;
  refresh: () => void;
}

function useAgents(): UseAgentsResult {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  const fetchAgents = async () => {
    try {
      setLoading(true);
      const response = await fetch('http://localhost:9090/api/agents');
      const data = await response.json();
      setAgents(data.agents);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };
  
  useEffect(() => {
    fetchAgents();
    
    // Poll every 5 seconds
    const interval = setInterval(fetchAgents, 5000);
    return () => clearInterval(interval);
  }, []);
  
  return { agents, loading, error, refresh: fetchAgents };
}

// Kullanım
function AgentsList() {
  const { agents, loading, error, refresh } = useAgents();
  
  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  
  return (
    <div>
      <button onClick={refresh}>Refresh</button>
      <ul>
        {agents.map(agent => (
          <li key={agent.id}>
            {agent.id} - {agent.status}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

## 2. useTrafficStream Hook (WebSocket)

```typescript
import { useState, useEffect } from 'react';
import { createClient } from 'graphql-ws';

function useTrafficStream() {
  const [traffic, setTraffic] = useState<HttpTransaction[]>([]);
  const [connected, setConnected] = useState(false);
  
  useEffect(() => {
    const client = createClient({
      url: 'ws://localhost:9090/graphql',
    });
    
    const subscription = client.subscribe(
      {
        query: `
          subscription {
            trafficUpdates {
              requestId
              method
              url
              statusCode
              timestamp
            }
          }
        `,
      },
      {
        next: (data) => {
          setConnected(true);
          setTraffic(prev => [data.data.trafficUpdates, ...prev].slice(0, 100));
        },
        error: () => setConnected(false),
        complete: () => setConnected(false),
      }
    );
    
    return () => subscription.unsubscribe();
  }, []);
  
  return { traffic, connected };
}

// Kullanım
function TrafficMonitor() {
  const { traffic, connected } = useTrafficStream();
  
  return (
    <div>
      <div>Status: {connected ? '🟢 Connected' : '🔴 Disconnected'}</div>
      <ul>
        {traffic.map(tx => (
          <li key={tx.requestId}>
            {tx.method} {tx.url} - {tx.statusCode}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

## 3. useMetrics Hook

```typescript
function useMetrics(agentId?: string, refreshInterval = 5000) {
  const [metrics, setMetrics] = useState<MetricsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const fetchMetrics = async () => {
      const url = agentId 
        ? `http://localhost:9090/metrics/${agentId}`
        : 'http://localhost:9090/metrics';
        
      const response = await fetch(url);
      const data = await response.json();
      setMetrics(data);
      setLoading(false);
    };
    
    fetchMetrics();
    const interval = setInterval(fetchMetrics, refreshInterval);
    
    return () => clearInterval(interval);
  }, [agentId, refreshInterval]);
  
  return { metrics, loading };
}

// Kullanım
function MetricsDisplay({ agentId }: { agentId?: string }) {
  const { metrics, loading } = useMetrics(agentId);
  
  if (loading) return <div>Loading...</div>;
  if (!metrics) return null;
  
  return (
    <div>
      <h3>Metrics</h3>
      <p>Total Requests: {metrics.total_requests}</p>
      <p>Avg Response Time: {metrics.average_response_time_ms.toFixed(2)}ms</p>
      <p>Error Rate: {(metrics.error_rate * 100).toFixed(2)}%</p>
    </div>
  );
}
```

---

# Python Örnekleri

## 1. Python API Client

```python
import requests
from typing import Optional, List, Dict, Any
from dataclasses import dataclass

@dataclass
class AgentInfo:
    id: str
    address: str
    port: int
    status: str
    last_heartbeat: str
    version: str
    capabilities: List[str]

class ProxxyClient:
    def __init__(self, base_url: str = "http://localhost:9090"):
        self.base_url = base_url
        self.session = requests.Session()
    
    def check_health(self) -> Dict[str, Any]:
        """Check system health"""
        response = self.session.get(f"{self.base_url}/api/system/health")
        response.raise_for_status()
        return response.json()
    
    def list_agents(self) -> List[AgentInfo]:
        """List all agents"""
        response = self.session.get(f"{self.base_url}/api/agents")
        response.raise_for_status()
        data = response.json()
        
        return [
            AgentInfo(**agent)
            for agent in data['agents']
        ]
    
    def get_agent(self, agent_id: str) -> AgentInfo:
        """Get specific agent"""
        response = self.session.get(f"{self.base_url}/agents/{agent_id}")
        response.raise_for_status()
        return AgentInfo(**response.json())
    
    def register_agent(self, agent_config: Dict[str, Any]) -> Dict[str, Any]:
        """Register new agent"""
        response = self.session.post(
            f"{self.base_url}/agents",
            json=agent_config
        )
        response.raise_for_status()
        return response.json()
    
    def get_recent_traffic(self) -> List[Dict[str, Any]]:
        """Get recent traffic"""
        response = self.session.get(f"{self.base_url}/api/traffic/recent")
        response.raise_for_status()
        return response.json()['transactions']
    
    def get_metrics(self, agent_id: Optional[str] = None) -> Dict[str, Any]:
        """Get metrics"""
        url = f"{self.base_url}/metrics"
        if agent_id:
            url += f"/{agent_id}"
        
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()
    
    def graphql(self, query: str, variables: Optional[Dict] = None) -> Dict[str, Any]:
        """Execute GraphQL query"""
        response = self.session.post(
            f"{self.base_url}/graphql",
            json={'query': query, 'variables': variables}
        )
        response.raise_for_status()
        
        data = response.json()
        if 'errors' in data:
            raise Exception(data['errors'][0]['message'])
        
        return data['data']

# Kullanım
client = ProxxyClient()

# Health check
health = client.check_health()
print(f"System status: {health['status']}")

# List agents
agents = client.list_agents()
for agent in agents:
    print(f"{agent.id}: {agent.status}")

# Get metrics
metrics = client.get_metrics()
print(f"Total requests: {metrics['total_requests']}")
```

## 2. Async Python Client

```python
import aiohttp
import asyncio
from typing import Optional, List, Dict, Any

class AsyncProxxyClient:
    def __init__(self, base_url: str = "http://localhost:9090"):
        self.base_url = base_url
    
    async def check_health(self) -> Dict[str, Any]:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{self.base_url}/api/system/health") as response:
                return await response.json()
    
    async def list_agents(self) -> List[Dict[str, Any]]:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{self.base_url}/api/agents") as response:
                data = await response.json()
                return data['agents']
    
    async def get_metrics(self, agent_id: Optional[str] = None) -> Dict[str, Any]:
        url = f"{self.base_url}/metrics"
        if agent_id:
            url += f"/{agent_id}"
        
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                return await response.json()

# Kullanım
async def main():
    client = AsyncProxxyClient()
    
    # Paralel istekler
    health, agents, metrics = await asyncio.gather(
        client.check_health(),
        client.list_agents(),
        client.get_metrics()
    )
    
    print(f"Health: {health['status']}")
    print(f"Agents: {len(agents)}")
    print(f"Total Requests: {metrics['total_requests']}")

asyncio.run(main())
```

---

# Monitoring Script Örneği

## Bash Monitoring Script

```bash
#!/bin/bash

# Proxxy Orchestrator Monitoring Script

API_URL="http://localhost:9090"
REFRESH_INTERVAL=5

echo "🚀 Proxxy Orchestrator Monitor"
echo "================================"
echo ""

while true; do
    clear
    echo "📊 System Status - $(date)"
    echo "================================"
    
    # Health Check
    HEALTH=$(curl -s "$API_URL/api/system/health")
    STATUS=$(echo $HEALTH | jq -r '.status')
    UPTIME=$(echo $HEALTH | jq -r '.uptime_seconds')
    AGENTS_ONLINE=$(echo $HEALTH | jq -r '.agents_online')
    AGENTS_TOTAL=$(echo $HEALTH | jq -r '.agents_total')
    
    echo "Status: $STATUS"
    echo "Uptime: $UPTIME seconds"
    echo "Agents: $AGENTS_ONLINE/$AGENTS_TOTAL online"
    echo ""
    
    # Metrics
    echo "📈 Metrics"
    echo "================================"
    METRICS=$(curl -s "$API_URL/metrics")
    TOTAL_REQUESTS=$(echo $METRICS | jq -r '.total_requests')
    AVG_RESPONSE=$(echo $METRICS | jq -r '.average_response_time_ms')
    ERROR_RATE=$(echo $METRICS | jq -r '.error_rate * 100')
    
    echo "Total Requests: $TOTAL_REQUESTS"
    echo "Avg Response Time: ${AVG_RESPONSE}ms"
    echo "Error Rate: ${ERROR_RATE}%"
    echo ""
    
    # Recent Traffic
    echo "🚦 Recent Traffic (Last 5)"
    echo "================================"
    curl -s "$API_URL/api/traffic/recent" | jq -r '.transactions[:5][] | "\(.method) \(.url) - \(.status)"'
    
    echo ""
    echo "Refreshing in $REFRESH_INTERVAL seconds... (Ctrl+C to exit)"
    sleep $REFRESH_INTERVAL
done
```

---

**Son Güncelleme:** 2026-01-09  
**Versiyon:** 0.1.1
