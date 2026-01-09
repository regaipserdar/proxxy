# Proxxy Orchestrator - API Reference

## 📖 Genel Bakış

Proxxy Orchestrator, dağıtık MITM proxy sisteminin merkezi yönetim sunucusudur. Bu dokümantasyon, tüm API endpoint'lerini, istek/yanıt formatlarını ve kullanım örneklerini detaylı olarak açıklar.

### API Tipleri

Orchestrator üç farklı API türü sunar:

1. **REST API** - HTTP tabanlı RESTful endpoint'ler
2. **GraphQL API** - Esnek sorgulama ve gerçek zamanlı güncellemeler
3. **gRPC API** - Agent'lar için yüksek performanslı iletişim (dahili)

---

## 🌐 Temel URL'ler

```
REST API:      http://localhost:9090/api/*
GraphQL:       http://localhost:9090/graphql
GraphiQL:      http://localhost:9090/graphql (Tarayıcıda)
Swagger UI:    http://localhost:9090/swagger-ui
OpenAPI Spec:  http://localhost:9090/api-docs/openapi.json
gRPC:          http://localhost:50051 (Dahili)
```

### Varsayılan Portlar

| Servis | Port | Açıklama |
|--------|------|----------|
| HTTP/REST | 9090 | REST API ve GraphQL endpoint'leri |
| gRPC | 50051 | Agent iletişimi için dahili gRPC sunucusu |

---

## 🔐 Kimlik Doğrulama

**Mevcut Durum:** API şu anda kimlik doğrulama gerektirmemektedir.

**Gelecek Planlama:**
- JWT token tabanlı kimlik doğrulama
- API key desteği
- Role-based access control (RBAC)

---

# 📡 REST API Endpoint'leri

## 1. Health & System Endpoints

### 1.1 Basic Health Check

**Endpoint:** `GET /health`

**Açıklama:** Temel sistem sağlık kontrolü

**Yanıt:**
```json
{
  "status": "healthy",
  "timestamp": "2026-01-09T14:49:45Z",
  "service": "orchestrator"
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/health
```

---

### 1.2 Detailed Health Status

**Endpoint:** `GET /api/health/detailed`

**Açıklama:** Detaylı sistem sağlık durumu ve çalışma süresi

**Yanıt:**
```json
{
  "status": "Healthy",
  "uptime_seconds": 3600,
  "database_connected": true
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/api/health/detailed
```

---

### 1.3 System Health

**Endpoint:** `GET /api/system/health`

**Açıklama:** Sistem geneli sağlık durumu ve agent istatistikleri

**Yanıt:**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "database_connected": true,
  "agents_online": 3,
  "agents_total": 5
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/api/system/health
```

---

### 1.4 Start System

**Endpoint:** `POST /api/system/start`

**Açıklama:** Proxy sistemini başlatır

**Yanıt:**
```json
{
  "status": "success",
  "message": "System is already running"
}
```

**Kullanım Örneği:**
```bash
curl -X POST http://localhost:9090/api/system/start
```

---

### 1.5 Stop System

**Endpoint:** `POST /api/system/stop`

**Açıklama:** Proxy sistemini durdurur

**Yanıt:**
```json
{
  "status": "success",
  "message": "System stop initiated"
}
```

**Kullanım Örneği:**
```bash
curl -X POST http://localhost:9090/api/system/stop
```

---

### 1.6 Restart System

**Endpoint:** `POST /api/system/restart`

**Açıklama:** Proxy sistemini yeniden başlatır

**Yanıt:**
```json
{
  "status": "success",
  "message": "System restart initiated"
}
```

**Kullanım Örneği:**
```bash
curl -X POST http://localhost:9090/api/system/restart
```

---

## 2. Agent Management Endpoints

### 2.1 List All Agents

**Endpoint:** `GET /api/agents`

**Açıklama:** Tüm kayıtlı proxy agent'larını listeler

**Yanıt:**
```json
{
  "agents": [
    {
      "id": "agent-001",
      "address": "192.168.1.100",
      "port": 8080,
      "status": "Online",
      "last_heartbeat": "2026-01-09T14:49:45Z",
      "version": "0.1.1",
      "capabilities": ["http", "https", "websocket"]
    },
    {
      "id": "agent-002",
      "address": "192.168.1.101",
      "port": 8080,
      "status": "Offline",
      "last_heartbeat": "2026-01-09T14:30:00Z",
      "version": "0.1.0",
      "capabilities": ["http", "https"]
    }
  ],
  "total_count": 2,
  "online_count": 1,
  "offline_count": 1
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/api/agents
```

---

### 2.2 Get Specific Agent

**Endpoint:** `GET /agents/{agent_id}`

**Açıklama:** Belirli bir agent'ın detaylı bilgilerini getirir

**Path Parameters:**
- `agent_id` (string, required): Agent'ın benzersiz kimliği

**Yanıt:**
```json
{
  "id": "agent-001",
  "address": "192.168.1.100",
  "port": 8080,
  "status": "Online",
  "last_heartbeat": "2026-01-09T14:49:45Z",
  "version": "0.1.1",
  "capabilities": ["http", "https", "websocket"]
}
```

**Hata Yanıtları:**
- `404 Not Found`: Agent bulunamadı

**Kullanım Örneği:**
```bash
curl http://localhost:9090/agents/agent-001
```

---

### 2.3 Register New Agent

**Endpoint:** `POST /agents`

**Açıklama:** Yeni bir proxy agent'ı sisteme kaydeder

**Request Body:**
```json
{
  "agent_id": "agent-003",
  "address": "192.168.1.102",
  "port": 8080,
  "version": "0.1.1",
  "capabilities": ["http", "https", "websocket"]
}
```

**Yanıt:**
```json
{
  "success": true,
  "message": "Agent registered successfully",
  "agent_id": "agent-003"
}
```

**Hata Yanıtları:**
- `500 Internal Server Error`: Veritabanı hatası

**Kullanım Örneği:**
```bash
curl -X POST http://localhost:9090/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-003",
    "address": "192.168.1.102",
    "port": 8080,
    "version": "0.1.1",
    "capabilities": ["http", "https", "websocket"]
  }'
```

---

## 3. Traffic Data Endpoints

### 3.1 Get Recent Traffic

**Endpoint:** `GET /api/traffic/recent`

**Açıklama:** Son HTTP trafiğini getirir (maksimum 50 kayıt)

**Yanıt:**
```json
{
  "transactions": [
    {
      "request_id": "req-12345",
      "agent_id": "agent-001",
      "method": "GET",
      "url": "https://api.example.com/users",
      "status": 200,
      "timestamp": 1704815385
    },
    {
      "request_id": "req-12346",
      "agent_id": "agent-001",
      "method": "POST",
      "url": "https://api.example.com/login",
      "status": 401,
      "timestamp": 1704815390
    }
  ],
  "total_count": 2
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/api/traffic/recent
```

---

### 3.2 List Traffic with Filters

**Endpoint:** `GET /traffic`

**Açıklama:** Trafik verilerini filtrelerle listeler

**Query Parameters:**
- `agent_id` (string, optional): Belirli bir agent'ın trafiği
- `limit` (integer, optional): Maksimum kayıt sayısı (varsayılan: 100)

**Yanıt:**
```json
{
  "traffic_data": [],
  "total_count": 0
}
```

**Not:** Bu endpoint şu anda boş veri döndürür, veritabanı entegrasyonu devam ediyor.

**Kullanım Örneği:**
```bash
# Tüm trafik
curl http://localhost:9090/traffic?limit=50

# Belirli agent'ın trafiği
curl http://localhost:9090/traffic?agent_id=agent-001&limit=100
```

---

### 3.3 Get Agent-Specific Traffic

**Endpoint:** `GET /traffic/{agent_id}`

**Açıklama:** Belirli bir agent'ın trafik verilerini getirir

**Path Parameters:**
- `agent_id` (string, required): Agent'ın benzersiz kimliği

**Yanıt:**
```json
{
  "traffic_data": [],
  "total_count": 0
}
```

**Hata Yanıtları:**
- `404 Not Found`: Agent bulunamadı

**Kullanım Örneği:**
```bash
curl http://localhost:9090/traffic/agent-001
```

---

## 4. Metrics Endpoints

### 4.1 System-Wide Metrics

**Endpoint:** `GET /metrics`

**Açıklama:** Sistem geneli metrikler ve istatistikler

**Yanıt:**
```json
{
  "total_requests": 15234,
  "average_response_time_ms": 245.7,
  "error_rate": 0.023
}
```

**Metrik Açıklamaları:**
- `total_requests`: Toplam işlenen HTTP isteği sayısı
- `average_response_time_ms`: Ortalama yanıt süresi (milisaniye)
- `error_rate`: Hata oranı (4xx ve 5xx yanıtlar / toplam istekler)

**Kullanım Örneği:**
```bash
curl http://localhost:9090/metrics
```

---

### 4.2 Agent-Specific Metrics

**Endpoint:** `GET /metrics/{agent_id}`

**Açıklama:** Belirli bir agent'ın metriklerini getirir

**Path Parameters:**
- `agent_id` (string, required): Agent'ın benzersiz kimliği

**Yanıt:**
```json
{
  "agent_id": "agent-001",
  "timestamp": "2026-01-09T14:49:45Z",
  "requests_handled": 5432,
  "average_response_time_ms": 198.5,
  "error_rate": 0.015,
  "memory_usage_mb": 256,
  "cpu_usage_percent": 12.5
}
```

**Hata Yanıtları:**
- `404 Not Found`: Agent bulunamadı

**Kullanım Örneği:**
```bash
curl http://localhost:9090/metrics/agent-001
```

---

## 5. Root Endpoint

### 5.1 API Documentation

**Endpoint:** `GET /`

**Açıklama:** API dokümantasyonu ve mevcut endpoint'lerin listesi

**Yanıt:**
```json
{
  "service": "Distributed MITM Proxxy Orchestrator",
  "version": "0.1.1",
  "status": "running",
  "endpoints": [
    {
      "path": "/",
      "method": "GET",
      "description": "This welcome page with API documentation"
    },
    {
      "path": "/health",
      "method": "GET",
      "description": "Basic health check"
    },
    {
      "path": "/health/detailed",
      "method": "GET",
      "description": "Detailed system health status"
    },
    {
      "path": "/agents",
      "method": "GET",
      "description": "List all registered proxy agents"
    },
    {
      "path": "/agents",
      "method": "POST",
      "description": "Register a new proxy agent"
    },
    {
      "path": "/agents/{agent_id}",
      "method": "GET",
      "description": "Get information about a specific agent"
    },
    {
      "path": "/traffic",
      "method": "GET",
      "description": "Get recent traffic data (query params: agent_id, limit)"
    },
    {
      "path": "/traffic/{agent_id}",
      "method": "GET",
      "description": "Get traffic data for a specific agent"
    },
    {
      "path": "/metrics",
      "method": "GET",
      "description": "Get system-wide metrics"
    },
    {
      "path": "/metrics/{agent_id}",
      "method": "GET",
      "description": "Get metrics for a specific agent"
    }
  ]
}
```

**Kullanım Örneği:**
```bash
curl http://localhost:9090/
```

---

# 🎨 GraphQL API

## GraphQL Playground

GraphQL Playground'a tarayıcınızdan erişebilirsiniz:
```
http://localhost:9090/graphql
```

Bu interaktif arayüz, sorguları test etmenizi, şemayı keşfetmenizi ve dokümantasyonu görüntülemenizi sağlar.

---

## Queries

### 1. List Agents

**Açıklama:** Tüm kayıtlı agent'ları listeler

**Query:**
```graphql
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
```

**Yanıt:**
```json
{
  "data": {
    "agents": [
      {
        "id": "agent-001",
        "name": "Dev-Agent-1",
        "hostname": "laptop.local",
        "status": "Online",
        "version": "0.1.1",
        "lastHeartbeat": "2026-01-09T14:49:45Z"
      }
    ]
  }
}
```

---

### 2. Get HTTP Transactions

**Açıklama:** HTTP işlemlerini getirir

**Query:**
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

**Parameters:**
- `limit` (Int, optional): Maksimum kayıt sayısı

**Yanıt:**
```json
{
  "data": {
    "httpTransactions": [
      {
        "requestId": "req-12345",
        "method": "GET",
        "url": "https://api.example.com/users",
        "statusCode": 200,
        "timestamp": "2026-01-09T14:49:45Z",
        "agentId": "agent-001"
      }
    ]
  }
}
```

---

### 3. Get System Metrics

**Açıklama:** Sistem metriklerini getirir

**Query:**
```graphql
query {
  systemMetrics(agentId: "agent-001", limit: 60) {
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

**Parameters:**
- `agentId` (String, required): Agent kimliği
- `limit` (Int, optional): Maksimum kayıt sayısı

**Yanıt:**
```json
{
  "data": {
    "systemMetrics": [
      {
        "agentId": "agent-001",
        "timestamp": "2026-01-09T14:49:45Z",
        "cpuUsagePercent": 12.5,
        "memoryUsedBytes": 268435456,
        "memoryTotalBytes": 8589934592,
        "networkRxBytes": 1024000,
        "networkTxBytes": 512000,
        "diskReadBytes": 2048000,
        "diskWriteBytes": 1024000,
        "processCpuPercent": 8.2,
        "processMemoryBytes": 134217728
      }
    ]
  }
}
```

---

### 4. Get Current System Metrics

**Açıklama:** Anlık sistem metriklerini getirir

**Query:**
```graphql
query {
  currentSystemMetrics(agentId: "agent-001") {
    cpuUsagePercent
    memoryUsedBytes
    memoryTotalBytes
    networkRxBytes
    networkTxBytes
  }
}
```

**Parameters:**
- `agentId` (String, required): Agent kimliği

---

## Mutations

### 1. Replay HTTP Request

**Açıklama:** Yakalanmış bir HTTP isteğini tekrar gönderir

**Mutation:**
```graphql
mutation {
  replayRequest(requestId: "req-12345") {
    success
    message
    replayRequestId
    originalUrl
    originalMethod
  }
}
```

**Parameters:**
- `requestId` (String, required): Tekrar gönderilecek isteğin kimliği

**Yanıt:**
```json
{
  "data": {
    "replayRequest": {
      "success": true,
      "message": "Request replayed successfully",
      "replayRequestId": "req-12346",
      "originalUrl": "https://api.example.com/users",
      "originalMethod": "GET"
    }
  }
}
```

---

### 2. Intercept Request

**Açıklama:** İsteği gerçek zamanlı olarak müdahale eder (gelecek özellik)

**Mutation:**
```graphql
mutation {
  intercept(id: "req-12345", action: "drop") {
    success
  }
}
```

**Parameters:**
- `id` (String, required): İstek kimliği
- `action` (String, required): Eylem ("drop", "modify", "forward")

---

## Subscriptions

### 1. Real-time Traffic Updates

**Açıklama:** Gerçek zamanlı trafik güncellemeleri

**Subscription:**
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

**WebSocket Yanıtı:**
```json
{
  "data": {
    "trafficUpdates": {
      "requestId": "req-12347",
      "method": "POST",
      "url": "https://api.example.com/login",
      "statusCode": 200,
      "timestamp": "2026-01-09T14:50:00Z"
    }
  }
}
```

---

### 2. Real-time System Metrics

**Açıklama:** Gerçek zamanlı sistem metrikleri

**Subscription:**
```graphql
subscription {
  systemMetricsUpdates(agentId: "agent-001") {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    networkRxBytes
    networkTxBytes
  }
}
```

**Parameters:**
- `agentId` (String, required): Agent kimliği

**WebSocket Yanıtı:**
```json
{
  "data": {
    "systemMetricsUpdates": {
      "agentId": "agent-001",
      "timestamp": "2026-01-09T14:50:00Z",
      "cpuUsagePercent": 13.2,
      "memoryUsedBytes": 270532608,
      "networkRxBytes": 1025024,
      "networkTxBytes": 513024
    }
  }
}
```

---

# 🔧 gRPC API (Dahili)

gRPC API, agent'lar ile orchestrator arasındaki dahili iletişim için kullanılır. UI geliştirmesi için genellikle gerekli değildir.

## Proto Tanımları

Proto dosyası: `proto/proxy.proto`

### Servisler

1. **RegisterAgent** - Agent kaydı
2. **Heartbeat** - Agent sağlık kontrolü
3. **SubmitTrafficData** - Trafik verisi gönderimi
4. **SubmitMetrics** - Metrik gönderimi
5. **GetConfiguration** - Yapılandırma alma

---

# 📊 Veri Modelleri

## AgentInfo

```typescript
interface AgentInfo {
  id: string;              // Benzersiz agent kimliği
  address: string;         // IP adresi
  port: number;            // Port numarası
  status: string;          // "Online" | "Offline"
  last_heartbeat: string;  // ISO 8601 timestamp
  version: string;         // Agent versiyonu
  capabilities: string[];  // Yetenekler ["http", "https", "websocket"]
}
```

## HttpTransaction

```typescript
interface HttpTransaction {
  request_id: string;      // Benzersiz istek kimliği
  agent_id: string;        // Agent kimliği
  method: string;          // HTTP metodu (GET, POST, vb.)
  url: string;             // Tam URL
  status: number | null;   // HTTP durum kodu
  timestamp: number;       // Unix timestamp
}
```

## SystemMetrics

```typescript
interface SystemMetrics {
  agent_id: string;
  timestamp: string;
  cpu_usage_percent: number;
  memory_used_bytes: number;
  memory_total_bytes: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  disk_read_bytes: number;
  disk_write_bytes: number;
  process_cpu_percent: number;
  process_memory_bytes: number;
}
```

---

# 🚀 Hızlı Başlangıç

## 1. Orchestrator'ı Başlatma

```bash
# Varsayılan ayarlarla
cargo run -p orchestrator

# Özel portlarla
cargo run -p orchestrator -- --grpc-port 50051 --http-port 9090

# Özel veritabanı ile
cargo run -p orchestrator -- --database-url sqlite:./my-proxy.db
```

## 2. API'yi Test Etme

### REST API
```bash
# Health check
curl http://localhost:9090/health

# Agent listesi
curl http://localhost:9090/api/agents

# Metrikler
curl http://localhost:9090/metrics
```

### GraphQL
Tarayıcıda açın: `http://localhost:9090/graphql`

Test sorgusu:
```graphql
query {
  agents {
    id
    name
    status
  }
}
```

---

# 🎯 Kullanım Senaryoları

## Senaryo 1: Canlı Trafik İzleme

1. GraphQL Playground'u açın
2. Subscription başlatın:
```graphql
subscription {
  trafficUpdates {
    requestId
    method
    url
    statusCode
  }
}
```
3. Yeni istekler gerçek zamanlı olarak görünecektir

## Senaryo 2: Agent Sağlığını İzleme

1. Agent listesini alın:
```bash
curl http://localhost:9090/api/agents
```

2. Belirli bir agent'ın metriklerini izleyin:
```bash
curl http://localhost:9090/metrics/agent-001
```

3. Gerçek zamanlı metrikler için GraphQL subscription kullanın:
```graphql
subscription {
  systemMetricsUpdates(agentId: "agent-001") {
    cpuUsagePercent
    memoryUsedBytes
  }
}
```

## Senaryo 3: İstek Tekrarlama

1. Trafik geçmişini görüntüleyin:
```bash
curl http://localhost:9090/api/traffic/recent
```

2. Bir isteği tekrarlayın:
```graphql
mutation {
  replayRequest(requestId: "req-12345") {
    success
    message
    replayRequestId
  }
}
```

---

# 🔍 Hata Kodları

## HTTP Durum Kodları

| Kod | Açıklama | Örnek Durum |
|-----|----------|-------------|
| 200 | OK | İstek başarılı |
| 404 | Not Found | Agent veya kaynak bulunamadı |
| 500 | Internal Server Error | Veritabanı hatası, sunucu hatası |

## GraphQL Hataları

GraphQL hataları `errors` dizisinde döner:

```json
{
  "errors": [
    {
      "message": "Agent not found",
      "locations": [{"line": 2, "column": 3}],
      "path": ["agent"]
    }
  ],
  "data": null
}
```

---

# 📚 İleri Seviye Konular

## CORS Yapılandırması

API, tüm origin'lerden gelen isteklere izin verir (permissive CORS). Üretim ortamında bunu kısıtlamanız önerilir.

## Rate Limiting

Şu anda rate limiting yoktur. Üretim ortamında eklenmesi önerilir.

## WebSocket Bağlantıları

GraphQL subscriptions WebSocket protokolü kullanır:
- **Endpoint:** `ws://localhost:9090/graphql`
- **Protocol:** graphql-ws

### JavaScript Örneği

```javascript
import { createClient } from 'graphql-ws';

const client = createClient({
  url: 'ws://localhost:9090/graphql',
});

const unsubscribe = client.subscribe(
  {
    query: `
      subscription {
        trafficUpdates {
          requestId
          method
          url
        }
      }
    `,
  },
  {
    next: (data) => console.log('New traffic:', data),
    error: (error) => console.error('Error:', error),
    complete: () => console.log('Done'),
  }
);
```

---

# 🛠️ Geliştirme Araçları

## Swagger UI

OpenAPI dokümantasyonunu görüntülemek için:
```
http://localhost:9090/swagger-ui
```

## GraphiQL Playground

GraphQL sorgularını test etmek için:
```
http://localhost:9090/graphql
```

## cURL Örnekleri

### POST İsteği
```bash
curl -X POST http://localhost:9090/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "test-agent",
    "address": "127.0.0.1",
    "port": 8080,
    "version": "0.1.1",
    "capabilities": ["http", "https"]
  }'
```

### GraphQL Sorgusu
```bash
curl -X POST http://localhost:9090/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ agents { id name status } }"
  }'
```

---

# 📖 İlgili Dokümantasyon

- [Backend API Specification](./backend-api.md) - UI için backend özellikleri
- [Architecture](./architecture.md) - Sistem mimarisi
- [Traffic Policy](./traffic-policy.md) - Trafik politikaları
- [Flow Engine](./flow-engine.md) - Flow engine dokümantasyonu

---

# 📝 Sürüm Geçmişi

## v0.1.1 (Mevcut)
- ✅ REST API endpoint'leri
- ✅ GraphQL API (queries, mutations, subscriptions)
- ✅ Agent yönetimi
- ✅ Trafik yakalama
- ✅ Sistem metrikleri
- ✅ OpenAPI/Swagger dokümantasyonu
- ✅ CORS desteği

## Gelecek Sürümler
- 🔜 Kimlik doğrulama (JWT)
- 🔜 Rate limiting
- 🔜 Gelişmiş filtreleme
- 🔜 HAR export
- 🔜 WebSocket intercept

---

**Son Güncelleme:** 2026-01-09  
**Versiyon:** 0.1.1  
**Durum:** ✅ Aktif Geliştirme
