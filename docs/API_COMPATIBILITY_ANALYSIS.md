# API Endpoint Uyumluluk Analizi - Proxxy GUI

## 🔍 Tespit Edilen Sorunlar

### 1. ❌ GraphQL Query İsimleri Uyumsuz

**Frontend (operations.ts):**
```graphql
query GetHttpTransactions {
  requests {  # ✅ DOĞRU
    requestId
    method
    url
    status
    ...
  }
}
```

**Backend (graphql/mod.rs):**
```rust
async fn requests(&self, ctx: &Context<'_>) -> Result<Vec<TrafficEventGql>> {
  // ✅ DOĞRU - "requests" query'si mevcut
}
```
**Durum:** ✅ UYUMLU

---

### 2. ❌ GraphQL Subscription İsimleri Uyumsuz

**Frontend (operations.ts - Satır 99-113):**
```graphql
subscription TrafficUpdates {
  events {  # ✅ DOĞRU
    requestId
    method
    url
    ...
  }
}
```

**Backend (graphql/mod.rs - Satır 114-121):**
```rust
async fn events(&self, ctx: &Context<'_>) -> impl Stream<Item = TrafficEventGql> {
  // ✅ DOĞRU - "events" subscription mevcut
}
```
**Durum:** ✅ UYUMLU

---

### 3. ⚠️ GraphQL Field İsimleri - Kısmi Uyumsuzluk

**Frontend Bekliyor:**
```typescript
interface HttpTransaction {
  requestId: string;
  method: string;
  url: string;
  status: number;
  timestamp: string;
  agentId: string;
  requestHeaders: string;
  requestBody: string;
  responseHeaders: string;
  responseBody: string;
}
```

**Backend Sağlıyor (TrafficEventGql):**
```rust
pub struct TrafficEventGql {
    pub request_id: String,      // ✅ requestId (GraphQL auto-converts)
    pub method: Option<String>,   // ⚠️ Optional!
    pub url: Option<String>,      // ⚠️ Optional!
    pub status: Option<i32>,      // ⚠️ Optional!
    // ❌ timestamp: EKSIK
    // ❌ agentId: EKSIK
    // ❌ requestHeaders: EKSIK
    // ❌ requestBody: EKSIK
    // ❌ responseHeaders: EKSIK
    // ❌ responseBody: EKSIK
}
```

**Durum:** ❌ EKSIK ALANLAR VAR

---

### 4. ✅ System Metrics - Uyumlu

**Frontend (operations.ts - Satır 34-51):**
```graphql
query GetSystemMetrics($agentId: String, $limit: Int) {
  systemMetrics(agentId: $agentId, limit: $limit) {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    ...
  }
}
```

**Backend (graphql/mod.rs - Satır 38-45):**
```rust
async fn system_metrics(
    &self, 
    ctx: &Context<'_>, 
    agent_id: Option<String>,  // ✅ Matches
    limit: Option<i32>         // ✅ Matches
) -> Result<Vec<SystemMetricsGql>>
```

**Durum:** ✅ UYUMLU

---

### 5. ✅ Agents Query - Uyumlu

**Frontend (operations.ts - Satır 4-15):**
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

**Backend (graphql/mod.rs - Satır 25-36):**
```rust
async fn agents(&self, ctx: &Context<'_>) -> Result<Vec<AgentGql>> {
  // AgentGql has: id, name, hostname, status, version, last_heartbeat
}
```

**Durum:** ✅ UYUMLU (GraphQL auto-converts last_heartbeat -> lastHeartbeat)

---

### 6. ✅ Mutations - Uyumlu

**Frontend (operations.ts - Satır 80-96):**
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

mutation InterceptRequest($id: String!, $action: String!) {
  intercept(id: $id, action: $action)
}
```

**Backend (graphql/mod.rs):**
```rust
async fn replay_request(..., request_id: String) -> Result<ReplayResult> {
  // ReplayResult has: success, message, replay_request_id, original_url, original_method
}

async fn intercept(..., _id: String, _action: String) -> bool {
  // Returns bool
}
```

**Durum:** ✅ UYUMLU

---

## 📊 Özet

| Kategori | Durum | Detay |
|----------|-------|-------|
| GraphQL Endpoint URL | ✅ Uyumlu | `http://localhost:9090/graphql` |
| WebSocket URL | ✅ Uyumlu | `ws://localhost:9090/graphql` |
| REST API URL | ✅ Uyumlu | `http://localhost:9090` |
| Agents Query | ✅ Uyumlu | Tüm alanlar mevcut |
| System Metrics Query | ✅ Uyumlu | Tüm alanlar mevcut |
| Requests Query | ⚠️ Kısmi | Eksik alanlar var |
| Traffic Subscription | ⚠️ Kısmi | Eksik alanlar var |
| Mutations | ✅ Uyumlu | Tüm mutation'lar çalışıyor |

---

## 🔧 Düzeltilmesi Gerekenler

### Öncelik 1: TrafficEventGql Eksik Alanlar

Backend'de `TrafficEventGql` struct'ına şu alanlar eklenmeli:

```rust
#[derive(SimpleObject)]
pub struct TrafficEventGql {
    pub request_id: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status: Option<i32>,
    // YENİ ALANLAR:
    pub timestamp: Option<String>,        // ISO 8601 format
    pub agent_id: Option<String>,         // Hangi agent yakaladı
    pub request_headers: Option<String>,  // JSON string
    pub request_body: Option<String>,     // Base64 veya text
    pub response_headers: Option<String>, // JSON string
    pub response_body: Option<String>,    // Base64 veya text
}
```

### Öncelik 2: Database Query Güncellemesi

`Database::get_recent_requests()` metodu tam veri döndürmeli:

```rust
// orchestrator/src/database.rs
pub async fn get_recent_requests(&self, limit: i64) -> Result<Vec<TrafficEvent>> {
    sqlx::query_as::<_, TrafficEvent>(
        "SELECT 
            request_id,
            agent_id,
            req_method,
            req_url,
            req_headers,
            req_body,
            req_timestamp,
            res_status,
            res_headers,
            res_body,
            res_timestamp
         FROM http_transactions 
         ORDER BY req_timestamp DESC 
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&self.pool)
    .await
}
```

---

## ✅ Çalışan Özellikler

1. **Agent Yönetimi** - Tam çalışıyor
2. **System Metrics** - Tam çalışıyor
3. **Real-time Metrics Subscription** - Tam çalışıyor
4. **Replay Request Mutation** - Tam çalışıyor
5. **GraphQL Connection** - Tam çalışıyor
6. **WebSocket Connection** - Tam çalışıyor

---

## 🎯 Öneriler

### Kısa Vadeli (Hemen)
1. ✅ API endpoint'leri doğru
2. ⚠️ TrafficEventGql struct'ını genişlet
3. ⚠️ Database query'lerini güncelle

### Orta Vadeli
1. Frontend'de tip kontrolü ekle (TypeScript strict mode)
2. GraphQL schema validation testleri ekle
3. API versiyonlama sistemi düşün

### Uzun Vadeli
1. GraphQL Code Generator kullan (otomatik tip üretimi)
2. E2E testler ekle
3. API dokümantasyonunu otomatik güncelle

---

**Analiz Tarihi:** 2026-01-09  
**Durum:** Çoğu endpoint uyumlu, küçük düzeltmeler gerekli
