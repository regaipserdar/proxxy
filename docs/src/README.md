# Proxxy - Dağıtık MITM Proxy Sistemi

Proxxy, HTTP/HTTPS trafiğini incelemek ve manipüle etmek için tasarlanmış, kurumsal düzeyde dağıtık bir Man-in-the-Middle (MITM) proxy çözümüdür. Merkezi bir Orchestrator, birden fazla Agent ve modern bir GUI'den oluşan modüler bir mimari sunar.

> **💡 Vizyon:** Bu stack, **Zero-Cost Abstractions** (Sıfır Maliyetli Soyutlamalar) ve **Memory Safety** (Bellek Güvenliği) prensiplerine dayanır. Python veya Node.js tabanlı rakiplerine (Puppeteer, Selenium) göre **10x daha az RAM** ve çok daha düşük latency hedefliyoruz.


## 🎯 Temel Özellikler

- **Dağıtık Mimari**: Birden fazla agent'ı merkezi bir noktadan yönetin
- **Gerçek Zamanlı Trafik İzleme**: HTTP/HTTPS isteklerini canlı olarak görüntüleyin
- **Dinamik Trafik Politikaları**: Runtime'da kurallar ekleyin/değiştirin (restart gerekmez)
- **Merkezi CA Yönetimi**: Tek bir Root CA sertifikası tüm agent'lar için
- **İstek Durdurma (Intercept)**: Kritik istekleri manuel onay bekletme
- **gRPC Sistem Metrikleri**: CPU, RAM, network kullanımını gerçek zamanlı izleme
- **GraphQL + REST API**: Esnek sorgulama ve entegrasyon
- **Modern GUI**: Tauri tabanlı masaüstü uygulaması
- **🚀 Flow Engine** (v1.2+): Zero-copy automation & replay system
  - **10x daha az RAM** (Python/Node.js rakiplerine göre)
  - **Visual & Protocol Mode**: Browser veya HTTP-only execution
  - **Self-Healing Selectors**: DOM değişikliklerine dayanıklı
  - **Login Sequence Recorder**: Otomatik akış kaydı
  - Detaylar: [`docs/src/flow-engine.md`](./docs/src/flow-engine.md)

## 🚀 Hızlı Başlangıç

### CLI Kullanımı

```bash
# Orchestrator başlatma
cargo run -p orchestrator -- --help
cargo run -p orchestrator -- --grpc-port 50051 --http-port 9090

# Proxy Agent başlatma
cargo run -p proxy-agent -- --help
cargo run -p proxy-agent -- --name "MyAgent" --listen-port 9095

# Database kontrolü
sqlite3 proxxy.db "SELECT id, name, status FROM agents;"
sqlite3 proxxy.db "SELECT COUNT(*) FROM http_transactions;"
```

**Not:** `cargo run` ile argüman geçmek için `--` kullanılır. Bu, cargo'ya "bundan sonraki argümanlar programa gidiyor" der.

## 📁 Proje Yapısı

```
proxxy/
├── orchestrator/          # Merkezi yönetim sunucusu
│   ├── src/
│   │   ├── main.rs       # CLI ve başlangıç noktası
│   │   ├── lib.rs        # REST/GraphQL API
│   │   ├── server.rs     # gRPC sunucu implementasyonu
│   │   ├── database.rs   # SQLite veritabanı işlemleri
│   │   └── session_manager.rs  # Agent oturum yönetimi
│   └── migrations/       # Veritabanı migration dosyaları
│
├── proxy-core/           # Paylaşılan MITM kütüphanesi
│   └── src/
│       ├── proxy.rs      # Hudsucker tabanlı proxy sunucusu
│       ├── ca.rs         # Sertifika otoritesi yönetimi
│       ├── policy.rs     # Runtime trafik politikaları
│       ├── controller.rs # İstek durdurma/devam ettirme
│       └── system_metrics.rs # Sistem metrikleri toplama
│
├── proxy-agent/          # Headless CLI agent
│   └── src/
│       ├── main.rs       # Agent başlangıç noktası
│       └── client.rs     # Orchestrator ile gRPC iletişimi
│
├── proxxy-gui/           # Tauri tabanlı masaüstü uygulaması
│   ├── src/              # React frontend
│   └── src-tauri/        # Tauri backend
│
├── proto/                # gRPC protokol tanımları
│   └── proxy.proto       # Agent-Orchestrator iletişim protokolü
│                         # (Traffic + System Metrics streaming)
│
└── docs/                 # Dokümantasyon
    └── TRAFFIC_POLICY.md # Trafik politikası sistemi detayları
```

## 🏗️ Mimari

### Sistem Bileşenleri

#### 1. **Orchestrator** - Merkezi Yönetim Sunucusu
- **gRPC Sunucusu** (`port 50051`): Agent'larla iletişim (traffic + metrics)
- **REST API** (`port 9090`): GUI için HTTP endpoint'leri
- **GraphQL API** (`port 9090/graphql`): Gelişmiş sorgular ve playground
- **SQLite Veritabanı**: Trafik logları, agent metadata ve sistem metrikleri
- **CA Yönetimi**: Root sertifika üretimi ve dağıtımı
- **Metrics Aggregation**: Tüm agent'lardan gelen sistem metriklerini toplama
- **Health Check Sistemi**: Agent'ların durumunu otomatik izleme

#### 2. **Proxy Core** - Paylaşılan Kütüphane
- **MITM Engine**: Hudsucker tabanlı HTTP/HTTPS proxy motoru
- **TLS Interceptor**: Dinamik sertifika üretimi (on-the-fly)
- **Policy Engine**: Runtime'da değiştirilebilir trafik kuralları
- **Request/Response Capture**: Tam HTTP transaction logging
- **System Metrics Collector**: CPU, RAM, network, disk kullanımı izleme
- **Match & Replace**: Otomatik içerik değiştirme

#### 3. **Proxy Agent** - Headless CLI Uygulaması
- **Lightweight Runner**: Uzak sunucularda çalışır
- **gRPC Client**: Orchestrator'a bağlanır ve komut alır
- **Traffic Streaming**: HTTP işlemlerini gerçek zamanlı akıtır
- **Metrics Streaming**: Sistem metriklerini gRPC ile sürekli gönderir
- **Memory-Only CA**: CA sertifikasını disk'e yazmaz (güvenlik)
- **Dynamic Configuration**: Orchestrator'dan gelen metric config'leri uygular

#### 4. **Proxxy GUI** - Masaüstü Uygulaması
- **React + TypeScript**: Modern, responsive UI
- **Tauri Backend**: Hafif, güvenli cross-platform wrapper
- **Real-time Updates**: GraphQL subscriptions ile canlı trafik ve metrikler
- **Agent Management**: Agent'ları görüntüleme ve yönetme
- **System Metrics Dashboard**: CPU, RAM, network kullanım grafikleri
- **Policy Editor**: Drag-and-drop kural editörü (planlı)

### gRPC Protocol Yapısı

```protobuf
service ProxyService {
  // HTTP/HTTPS trafik streaming
  rpc StreamTraffic (stream TrafficEvent) returns (stream InterceptCommand);
  
  // Agent registration
  rpc RegisterAgent (RegisterAgentRequest) returns (RegisterAgentResponse);
  
  // Sistem metrikleri streaming (YENİ!)
  rpc StreamMetrics (stream SystemMetricsEvent) returns (stream MetricsCommand);
}

message SystemMetricsEvent {
  string agent_id = 1;
  int64 timestamp = 2;
  SystemMetrics metrics = 3;
}

message SystemMetrics {
  float cpu_usage_percent = 1;        // CPU kullanım yüzdesi
  uint64 memory_used_bytes = 2;       // Kullanılan RAM (bytes)
  uint64 memory_total_bytes = 3;      // Toplam RAM (bytes)
  NetworkMetrics network = 4;         // Network I/O istatistikleri
  DiskMetrics disk = 5;               // Disk I/O istatistikleri
  ProcessMetrics process = 6;         // Process-specific metrikler
}
```

### Port Haritası

| Port  | Bileşen                | Amaç                                    |
|-------|------------------------|-----------------------------------------|
| 50051 | Orchestrator gRPC      | Agent bağlantıları, trafik ve metrik streaming |
| 9090  | Orchestrator HTTP      | REST API + GraphQL + Swagger UI         |
| 9095  | Agent Proxy (varsayılan)| Tarayıcı/uygulama trafiği buraya gelir |
| 9091  | Agent Admin API        | Health check ve local metrics endpoint'leri |

## 🚀 Hızlı Başlangıç

### Gereksinimler

```bash
# Rust (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Protobuf Compiler (gRPC için)
# macOS:
brew install protobuf

# Ubuntu/Debian:
sudo apt install -y protobuf-compiler

# Arch Linux:
sudo pacman -S protobuf

# Node.js (18+, GUI için)
# https://nodejs.org/
```

### Kurulum

```bash
# 1. Projeyi klonlayın
git clone <repo-url>
cd proxxy

# 2. Tüm workspace'i derleyin
cargo build --release

# 3. GUI bağımlılıkları (opsiyonel)
cd proxxy-gui
npm install
```

## 📖 Kullanım Kılavuzu

### 1. Orchestrator'ı Başlatma

```bash
# Varsayılan ayarlarla başlat
cargo run -p orchestrator

# Özel konfigürasyon ile
cargo run -p orchestrator -- \
  --grpc-port 50051 \
  --http-port 9090 \
  --database-url sqlite:./my-proxxy.db \
  --health-check-interval 30 \
  --agent-timeout 300
```

**CLI Parametreleri:**

| Parametre                     | Açıklama                              | Varsayılan           |
|-------------------------------|---------------------------------------|----------------------|
| `--grpc-port <PORT>`          | gRPC sunucu portu                     | 50051                |
| `--http-port <PORT>`          | HTTP API portu                        | 9090                 |
| `--database-url <URL>`        | SQLite bağlantı URL'i                 | sqlite:./proxxy.db   |
| `--health-check-interval <SEC>`| Health check aralığı (saniye)        | 30                   |
| `--agent-timeout <SEC>`       | Agent timeout süresi (saniye)         | 300                  |

**Orchestrator Endpoint'leri:**

```bash
# gRPC Endpoint (Agent'lar için)
http://127.0.0.1:50051

# REST API
http://127.0.0.1:9090/health/detailed    # Sistem sağlık durumu
http://127.0.0.1:9090/agents             # Kayıtlı agent listesi
http://127.0.0.1:9090/metrics            # Trafik metrikleri (legacy)
http://127.0.0.1:9090/system-metrics     # Sistem metrikleri (tüm agent'lar)
http://127.0.0.1:9090/traffic            # Son HTTP işlemleri

# GraphQL Playground (Sistem metrikleri dahil)
http://127.0.0.1:9090/graphql

# Swagger UI (REST API Dokümantasyonu)
http://127.0.0.1:9090/swagger-ui
http://127.0.0.1:9090/api-docs/openapi.json
```

### 2. Proxy Agent'ı Başlatma

```bash
# Tek agent (varsayılan ayarlar + sistem metrikleri)
cargo run -p proxy-agent -- --name "Agent-1"

# İkinci agent (farklı portlar + özel metrik konfigürasyonu)
cargo run -p proxy-agent -- \
  --name "Agent-2" \
  --listen-port 9096 \
  --admin-port 9092 \
  --metrics-interval 10  # 10 saniye metrik toplama

# Uzak Orchestrator'a bağlanma (production metrikleri)
cargo run -p proxy-agent -- \
  --name "Production-Agent" \
  --orchestrator-url http://203.0.113.10:50051 \
  --listen-port 8080 \
  --admin-port 8091 \
  --metrics-interval 5 \
  --enable-detailed-metrics
```

**CLI Parametreleri:**

| Parametre                  | Açıklama                           | Varsayılan                   |
|----------------------------|------------------------------------|------------------------------|
| `--listen-addr <ADDR>`     | Proxy dinleme adresi               | 127.0.0.1                    |
| `--listen-port <PORT>`     | Proxy dinleme portu                | 9095                         |
| `--admin-port <PORT>`      | Admin API portu                    | 9091                         |
| `--orchestrator-url <URL>` | Orchestrator gRPC endpoint'i       | http://127.0.0.1:50051       |
| `--name <NAME>`            | Agent için friendly isim (opsiyonel)| -                            |
| `--metrics-interval <SEC>` | Sistem metrikleri toplama aralığı (saniye) | 5                    |
| `--enable-detailed-metrics`| Detaylı network/disk metrikleri    | false                        |

**Agent Admin API Endpoint'leri:**

```bash
# Health Check
curl http://127.0.0.1:9091/health
# Çıktı: {"status": "ok"}

# Local Metrics (proxy-specific)
curl http://127.0.0.1:9091/metrics
# Çıktı: {"total_requests": 42, "active_connections": 3}

# System Metrics (local snapshot)
curl http://127.0.0.1:9091/system
# Çıktı: {
#   "cpu_usage_percent": 15.7,
#   "memory_used_bytes": 2147483648,
#   "memory_total_bytes": 8589934592,
#   "network_rx_bytes_per_sec": 1048576,
#   "network_tx_bytes_per_sec": 524288,
#   "process_cpu_percent": 2.3,
#   "process_memory_bytes": 67108864
# }
```

**Admin Port ve Sistem Metrikleri:**

Admin Port, her agent'ın kendi health check ve metrics bilgilerini sunduğu dahili bir HTTP endpoint'idir. **Yeni sistem metrikleri özelliği ile:**

- **Local System Snapshot**: `/system` endpoint'i ile anlık sistem durumu
- **gRPC Streaming**: Sürekli metrikler Orchestrator'a gRPC ile gönderilir
- **Kubernetes/Docker Health Checks**: Container orchestration için liveness/readiness probe
- **Monitoring (Prometheus/Grafana)**: Metrics scraping
- **Load Balancer Health Checks**: HAProxy, Nginx health check endpoint'leri

```yaml
# Kubernetes örneği (sistem metrikleri ile)
livenessProbe:
  httpGet:
    path: /health
    port: 9091
  initialDelaySeconds: 10
  periodSeconds: 5

# Sistem metrikleri için ek probe
readinessProbe:
  httpGet:
    path: /system
    port: 9091
  initialDelaySeconds: 5
  periodSeconds: 10
```

### 3. GUI'yi Başlatma

```bash
cd proxxy-gui

# İlk seferinde bağımlılıkları kur
npm install

# Geliştirme modunda çalıştır (sistem metrikleri dashboard dahil)
npm run tauri dev

# Production build
npm run tauri build
```

GUI otomatik olarak `http://127.0.0.1:9090` adresindeki Orchestrator'a bağlanır ve sistem metriklerini gerçek zamanlı görüntüler.

## 📊 Sistem Metrikleri Özellikleri

### gRPC Streaming Metrikleri

Her proxy agent sürekli olarak sistem metriklerini Orchestrator'a gönderir:

```bash
# Agent'dan Orchestrator'a sürekli akan metrikler:
SystemMetricsEvent {
  agent_id: "agent-1",
  timestamp: 1704738420,
  metrics: {
    cpu_usage_percent: 15.7,
    memory_used_bytes: 2147483648,
    memory_total_bytes: 8589934592,
    network: {
      rx_bytes_per_sec: 1048576,
      tx_bytes_per_sec: 524288
    },
    disk: {
      read_bytes_per_sec: 65536,
      write_bytes_per_sec: 32768
    },
    process: {
      cpu_usage_percent: 2.3,
      memory_bytes: 67108864,
      uptime_seconds: 3600
    }
  }
}
```

### GraphQL Sistem Metrikleri

```graphql
# Tüm agent'ların mevcut sistem durumu
query {
  agents {
    id
    name
    status
    currentMetrics {
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      networkRxBytesPerSec
      networkTxBytesPerSec
      processCpuPercent
      processMemoryBytes
    }
  }
}

# Belirli bir agent'ın metrik geçmişi
query {
  agent(id: "agent-1") {
    name
    metricsHistory(limit: 60) {
      timestamp
      cpuUsagePercent
      memoryUsedBytes
      networkRxBytesPerSec
    }
  }
}

# Real-time metrik güncellemeleri
subscription {
  systemMetricsUpdated {
    agentId
    cpuUsagePercent
    memoryUsedBytes
    networkRxBytesPerSec
  }
}
```

## 🔧 Kullanım Senaryoları

### Senaryo 1: Yerel Tek Agent Test (Sistem Metrikleri ile)

**Amaç**: Hızlı development/testing için minimal setup + metrik izleme

```bash
# Terminal 1: Orchestrator
cargo run -p orchestrator

# Terminal 2: Agent (sistem metrikleri aktif)
cargo run -p proxy-agent -- --name "Dev-Agent" --metrics-interval 5

# Terminal 3: GUI (sistem metrikleri dashboard ile)
cd proxxy-gui && npm run tauri dev

# Terminal 4: Test traffic + metrik izleme
curl -x http://127.0.0.1:9095 http://example.com

# Terminal 5: Sistem metriklerini kontrol et
curl http://127.0.0.1:9091/system | jq
```

### Senaryo 2: Çoklu Agent Yerel Setup (Sistem Metrikleri ile)

**Amaç**: Load balancing, multi-region testleri + merkezi metrik toplama

```bash
# Terminal 1: Orchestrator
cargo run -p orchestrator

# Terminal 2: Agent 1 (US Region) - Yüksek frekanslı metrikler
cargo run -p proxy-agent -- \
  --name "US-East-Agent" \
  --listen-port 9095 \
  --admin-port 9091 \
  --metrics-interval 3 \
  --enable-detailed-metrics

# Terminal 3: Agent 2 (EU Region) - Standart metrikler
cargo run -p proxy-agent -- \
  --name "EU-West-Agent" \
  --listen-port 9096 \
  --admin-port 9092 \
  --metrics-interval 5

# Terminal 4: Agent 3 (Asia Region) - Düşük frekanslı metrikler
cargo run -p proxy-agent -- \
  --name "Asia-South-Agent" \
  --listen-port 9097 \
  --admin-port 9093 \
  --metrics-interval 10
```

**Farklı agent'ları test etme ve metrik karşılaştırma:**
```bash
# US Agent üzerinden (yüksek load)
curl -x http://127.0.0.1:9095 http://api.example.com

# EU Agent üzerinden (orta load)
curl -x http://127.0.0.1:9096 http://api.example.com

# Sistem metriklerini karşılaştır
curl http://127.0.0.1:9091/system | jq '.cpu_usage_percent'  # US Agent
curl http://127.0.0.1:9092/system | jq '.cpu_usage_percent'  # EU Agent
curl http://127.0.0.1:9093/system | jq '.cpu_usage_percent'  # Asia Agent

# Orchestrator'dan tüm agent metrikleri
curl http://127.0.0.1:9090/system-metrics | jq
```

### Senaryo 3: Production Deployment (Uzak Sunucular)

**Amaç**: Gerçek kurumsal ortamda dağıtık proxy

```bash
# Sunucu 1: Orchestrator (Public IP: 203.0.113.10)
cargo run -p orchestrator --release -- \
  --grpc-port 50051 \
  --http-port 9090 \
  --database-url sqlite:/var/lib/proxxy/data.db

# Sunucu 2: AWS Worker Agent
cargo run -p proxy-agent --release -- \
  --name "AWS-US-East-1" \
  --orchestrator-url http://203.0.113.10:50051 \
  --listen-port 8080 \
  --admin-port 8081

# Sunucu 3: Azure Worker Agent
cargo run -p proxy-agent --release -- \
  --name "Azure-West-EU" \
  --orchestrator-url http://203.0.113.10:50051 \
  --listen-port 8080 \
  --admin-port 8081
```

### Senaryo 4: Docker Compose Deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  orchestrator:
    build: ./orchestrator
    ports:
      - "50051:50051"
      - "9090:9090"
    volumes:
      - proxxy-data:/data
    environment:
      - DATABASE_URL=sqlite:/data/proxxy.db

  agent-1:
    build: ./proxy-agent
    depends_on:
      - orchestrator
    environment:
      - ORCHESTRATOR_URL=http://orchestrator:50051
      - AGENT_NAME=Docker-Agent-1
    ports:
      - "9095:9095"
      - "9091:9091"

  agent-2:
    build: ./proxy-agent
    depends_on:
      - orchestrator
    environment:
      - ORCHESTRATOR_URL=http://orchestrator:50051
      - AGENT_NAME=Docker-Agent-2
    ports:
      - "9096:9096"
      - "9092:9092"

volumes:
  proxxy-data:
```

## 🌐 Proxy Kullanımı ve CA Sertifikası

### Tarayıcıyı Yapılandırma

Agent başladıktan sonra, tarayıcınızı proxy kullanacak şekilde ayarlayın:

**Manuel Proxy Ayarları:**
```
HTTP Proxy:  127.0.0.1:9095
HTTPS Proxy: 127.0.0.1:9095
SOCKS Proxy: (yok)
Bypass list: localhost, 127.0.0.1
```

**Chrome/Chromium (Linux/Mac):**
```bash
google-chrome --proxy-server="http://127.0.0.1:9095"
```

**Firefox (about:preferences):**
```
Network Settings → Manual proxy configuration
HTTP Proxy: 127.0.0.1, Port: 9095
✓ Also use this proxy for HTTPS
```

**curl ile test:**
```bash
# HTTP
curl -x http://127.0.0.1:9095 http://example.com

# HTTPS (CA sertifikası gerekli)
curl -x http://127.0.0.1:9095 --cacert ./certs/ca.crt https://example.com
```

### CA Sertifikasını Yükleme

**Adım 1: Sertifikayı Bul**

Orchestrator başladığında `./certs/ca.crt` dosyası otomatik oluşturulur.

```bash
ls -la ./certs/
# ca.crt  - Root CA sertifikası (tarayıcıya yüklenecek)
# ca.key  - Private key (GİZLİ, paylaşma!)
```

**Adım 2: Sisteme Yükle**

**macOS (Keychain):**
```bash
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain ./certs/ca.crt
```

**Ubuntu/Debian:**
```bash
sudo cp ./certs/ca.crt /usr/local/share/ca-certificates/proxxy-ca.crt
sudo update-ca-certificates
```

**Arch Linux:**
```bash
sudo trust anchor --store ./certs/ca.crt
```

**Windows:**
```powershell
# PowerShell (Yönetici olarak çalıştır)
Import-Certificate -FilePath ".\certs\ca.crt" -CertStoreLocation Cert:\LocalMachine\Root
```

**Firefox (Özel):**

Firefox sistem sertifikalarını kullanmaz, kendi sertifika deposu var:

1. `about:preferences#privacy` → Security → Certificates → View Certificates
2. "Authorities" sekmesi → Import
3. `ca.crt` dosyasını seç
4. ✓ "Trust this CA to identify websites"

**Adım 3: Doğrulama**

```bash
# Chrome DevTools
# Network → Bir HTTPS sitesine git → Connection → Certificate Issuer
# Görmelisin: "Proxxy Root CA"

# Firefox
# URL yanındaki kilit ikonu → Connection → More Information → View Certificate
# Issuer: Proxxy Root CA
```

## 📊 Veritabanı Şeması

Orchestrator SQLite kullanır ve otomatik migration yapar. Veritabanı dosyası varsayılan olarak `./proxxy.db` konumundadır.

### `agents` Tablosu

| Kolon           | Tip      | Açıklama                                    |
|-----------------|----------|---------------------------------------------|
| `id`            | TEXT PK  | Agent UUID (gRPC'den gelen)                 |
| `name`          | TEXT     | Friendly isim ("Production-Agent")          |
| `hostname`      | TEXT     | Agent'ın çalıştığı sunucu hostname'i        |
| `version`       | TEXT     | Agent versiyonu (semantic versioning)       |
| `status`        | TEXT     | Online / Offline / Disconnected             |
| `last_heartbeat`| INTEGER  | Unix timestamp (son görülme zamanı)         |
| `created_at`    | INTEGER  | Unix timestamp (ilk kayıt zamanı)           |

**Örnek Satır:**
```sql
INSERT INTO agents VALUES (
  'a1b2c3d4-5678-90ab-cdef-1234567890ab',
  'AWS-US-East-1',
  'ip-172-31-45-67.ec2.internal',
  '1.0.0',
  'Online',
  1704738420,
  1704738000
);
```

### `http_transactions` Tablosu

| Kolon            | Tip       | Açıklama                                  |
|------------------|-----------|-------------------------------------------|
| `request_id`     | TEXT PK   | İstek UUID                                |
| `agent_id`       | TEXT FK   | Hangi agent yakaladı (agents.id)          |
| `req_method`     | TEXT      | HTTP metodu (GET, POST, etc.)             |
| `req_url`        | TEXT      | Tam URL (https://example.com/api/users)   |
| `req_headers`    | TEXT JSON | Request header'ları (JSON serialized)     |
| `req_body`       | BLOB      | Request body (binary safe)                |
| `req_timestamp`  | INTEGER   | Unix timestamp (istek zamanı)             |
| `res_status`     | INTEGER   | HTTP status code (200, 404, etc.)         |
| `res_headers`    | TEXT JSON | Response header'ları                      |
| `res_body`       | BLOB      | Response body                             |
| `res_timestamp`  | INTEGER   | Unix timestamp (yanıt zamanı)             |
| `duration_ms`    | INTEGER   | İstek süresi (milisaniye)                 |
| `tls_info`       | TEXT JSON | TLS handshake bilgileri (cipher, version) |

### `system_metrics` Tablosu (YENİ!)

| Kolon                    | Tip     | Açıklama                                    |
|--------------------------|---------|---------------------------------------------|
| `id`                     | INTEGER | Primary key (auto increment)               |
| `agent_id`               | TEXT FK | Hangi agent'tan geldi (agents.id)          |
| `timestamp`              | INTEGER | Unix timestamp (metrik zamanı)             |
| `cpu_usage_percent`      | REAL    | CPU kullanım yüzdesi (0-100)               |
| `memory_used_bytes`      | INTEGER | Kullanılan RAM (bytes)                      |
| `memory_total_bytes`     | INTEGER | Toplam RAM (bytes)                          |
| `network_rx_bytes`       | INTEGER | Network alınan bytes (toplam)              |
| `network_tx_bytes`       | INTEGER | Network gönderilen bytes (toplam)          |
| `network_rx_bytes_per_sec` | INTEGER | Network alım hızı (bytes/saniye)          |
| `network_tx_bytes_per_sec` | INTEGER | Network gönderim hızı (bytes/saniye)      |
| `disk_read_bytes`        | INTEGER | Disk okuma bytes (toplam)                  |
| `disk_write_bytes`       | INTEGER | Disk yazma bytes (toplam)                  |
| `disk_read_bytes_per_sec` | INTEGER | Disk okuma hızı (bytes/saniye)            |
| `disk_write_bytes_per_sec` | INTEGER | Disk yazma hızı (bytes/saniye)           |
| `process_cpu_percent`    | REAL    | Process CPU kullanımı (0-100)              |
| `process_memory_bytes`   | INTEGER | Process RAM kullanımı (bytes)              |
| `process_uptime_seconds` | INTEGER | Process çalışma süresi (saniye)            |
| `created_at`             | INTEGER | Kayıt oluşturma zamanı (Unix timestamp)    |

**Örnek Sorgular:**
```sql
-- En yavaş 10 istek
SELECT req_url, duration_ms 
FROM http_transactions 
ORDER BY duration_ms DESC 
LIMIT 10;

-- Belirli bir agent'ın trafiği
SELECT COUNT(*) as total_requests
FROM http_transactions
WHERE agent_id = 'a1b2c3d4-5678-90ab-cdef-1234567890ab';

-- Agent'ların son 1 saatteki ortalama CPU kullanımı
SELECT 
  a.name,
  AVG(sm.cpu_usage_percent) as avg_cpu,
  AVG(sm.memory_used_bytes / 1024.0 / 1024.0) as avg_memory_mb
FROM agents a
JOIN system_metrics sm ON a.id = sm.agent_id
WHERE sm.timestamp > strftime('%s', 'now', '-1 hour')
GROUP BY a.id, a.name
ORDER BY avg_cpu DESC;

-- Yüksek CPU kullanımı olan anlar
SELECT 
  agent_id,
  datetime(timestamp, 'unixepoch') as time,
  cpu_usage_percent,
  memory_used_bytes / 1024.0 / 1024.0 as memory_mb
FROM system_metrics
WHERE cpu_usage_percent > 80
ORDER BY timestamp DESC
LIMIT 20;

-- Network I/O en yüksek olan agent'lar
SELECT 
  agent_id,
  MAX(network_rx_bytes_per_sec + network_tx_bytes_per_sec) as max_network_io,
  AVG(network_rx_bytes_per_sec + network_tx_bytes_per_sec) as avg_network_io
FROM system_metrics
WHERE timestamp > strftime('%s', 'now', '-1 hour')
GROUP BY agent_id
ORDER BY max_network_io DESC;
```

### Migration Sistemi

Veritabanı versiyonu otomatik takip edilir. `migrations/` klasöründeki SQL dosyaları sırayla uygulanır:

```
migrations/
├── 001_initial_schema.sql
├── 002_add_tls_info.sql
├── 003_agent_metadata.sql
└── 004_system_metrics.sql    # YENİ!
```

## 🎯 Traffic Policy Sistemi

Proxxy'nin en güçlü özelliği, **runtime'da değiştirilebilen trafik politikalarıdır**. Agent'ı yeniden başlatmadan kurallar ekleyebilir, değiştirebilir veya silebilirsiniz.

### Mimari: Static Config vs Dynamic Policy

```rust
// STATIC: Agent başlangıcında bir kere ayarlanır
ProxyStartupConfig {
    listen_address: "127.0.0.1",
    listen_port: 9095,
    orchestrator_endpoint: "http://127.0.0.1:50051",
    admin_port: 9091,
}

// DYNAMIC: Runtime'da Orchestrator'dan güncellenir
Arc<RwLock<TrafficPolicy>> {
    scope: ScopeConfig { ... },           // Hangi domainler
    interception_rules: Vec<Rule>,         // Ne yapılacak
    match_replace_rules: Vec<Replace>,     // Otomatik değişiklikler
}
```

### Temel Bileşenler

#### 1. Scope Configuration

Hangi domainlerin yakalanacağını belirler:

```rust
use proxy_core::{ScopeConfig, OutOfScopeAction};

let scope = ScopeConfig {
    // Dahil edilecek domainler (wildcard destekli)
    include: vec![
        "*.target.com".to_string(),
        "api.example.com".to_string(),
        "staging.*.internal".to_string(),
    ],
    
    // Hariç tutulacak domainler
    exclude: vec![
        "*.google-analytics.com".to_string(),
        "tracking.ads.com".to_string(),
    ],
    
    // Scope dışı trafik için aksiyon
    out_of_scope_action: OutOfScopeAction::Pass,
};
```

**Out-of-Scope Actions:**

| Aksiyon   | Davranış                                  | Kullanım                          |
|-----------|-------------------------------------------|-----------------------------------|
| `Pass`    | Trafiği işlemeden geçir                   | Normal proxy davranışı            |
| `LogOnly` | DB'ye kaydet ama UI'da gösterme           | Bant genişliği tasarrufu          |
| `Drop`    | TCP RST gönder, bağlantıyı kes            | Firewall testi, stealth mod       |

#### 2. Interception Rules

İstekleri yakalama ve işleme kuralları:

```rust
use proxy_core::{InterceptionRule, RuleCondition, RuleAction};

// Örnek 1: Admin paneli engelle
let block_admin = InterceptionRule {
    id: "block-admin".to_string(),
    name: "Block Admin Panel".to_string(),
    enabled: true,
    conditions: vec![
        RuleCondition::UrlContains("/admin".to_string()),
        RuleCondition::Method("POST".to_string()),
    ],
    action: RuleAction::Block {
        reason: "Admin access not allowed".to_string(),
    },
};

// Örnek 2: Login isteklerini durdur (manuel onay bekle)
let pause_login = InterceptionRule {
    id: "pause-login".to_string(),
    name: "Intercept Login Requests".to_string(),
    enabled: true,
    conditions: vec![
        RuleCondition::UrlRegex(r"/api/v1/login$".to_string()),
        RuleCondition::HasHeader("Authorization".to_string()),
    ],
    action: RuleAction::Pause,
};

// Örnek 3: Belirli istekleri yavaşlat
let slow_api = InterceptionRule {
    id: "slow-api".to_string(),
    name: "Simulate Slow API".to_string(),
    enabled: true,
    conditions: vec![
        RuleCondition::UrlContains("/api/slow".to_string()),
    ],
    action: RuleAction::Delay(2000), // 2 saniye
};
```

**Mevcut Condition Türleri:**

```rust
pub enum RuleCondition {
    UrlContains(String),              // Substring arama
    UrlRegex(String),                 // Regex pattern
    Method(String),                   // GET, POST, PUT, DELETE, etc.
    HasHeader(String),                // Header var mı?
    HeaderValueMatch { key, regex },  // Header değeri regex match
    BodyRegex(String),                // Request body'de arama
    Port(u16),                        // Port numarası
}
```

**Mevcut Action Türleri:**

```rust
pub enum RuleAction {
    Pause,                           // UI'da manuel onay bekle
    Block { reason: String },        // HTTP 403 dön
    Drop,                            // TCP RST gönder
    Delay(u64),                      // N milisaniye bekle
    InjectHeader { key, value },     // Header ekle/değiştir
    ModifyBody { find, replace },    // Body'de replace
}
```

#### 3. Match & Replace Rules

Otomatik içerik değiştirme:

```rust
use proxy_core::{MatchReplaceRule, MatchLocation};

// Örnek 1: Authorization token'ı redact et
let redact_token = MatchReplaceRule {
    enabled: true,
    match_regex: r"Authorization: Bearer (\S+)".to_string(),
    replace_string: "Authorization: Bearer [REDACTED]".to_string(),
    location: MatchLocation::RequestHeader,
};

// Örnek 2: API key'leri gizle
let hide_keys = MatchReplaceRule {
    enabled: true,
    match_regex: r#""api_key"\s*:\s*"([^"]+)""#.to_string(),
    replace_string: r#""api_key": "***HIDDEN***""#.to_string(),
    location: MatchLocation::RequestBody,
};

// Örnek 3: Response'ta email adreslerini maskele
let mask_emails = MatchReplaceRule {
    enabled: true,
    match_regex: r"([a-zA-Z0-9._%+-]+)@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,})".to_string(),
    replace_string: "***@$2".to_string(),
    location: MatchLocation::ResponseBody,
};
```

**Mevcut Location Türleri:**

```rust
pub enum MatchLocation {
    RequestHeader,    // Request header'ları
    RequestBody,      // Request body
    ResponseHeader,   // Response header'ları
    ResponseBody,     // Response body
}
```

### Runtime Policy Update

Policy güncellemeleri Orchestrator'dan gRPC ile gelir:

```rust
// Orchestrator UI'dan gönderilen yeni policy
let new_policy = TrafficPolicy {
    scope: ScopeConfig {
        include: vec!["*.target.com".to_string()],
        exclude: vec![],
        out_of_scope_action: OutOfScopeAction::Drop,
    },
    interception_rules: vec![pause_login],
    match_replace_rules: vec![redact_token],
};

// Agent otomatik uygular (RwLock ile thread-safe)
*policy.write().unwrap() = new_policy;
```

### Block vs Drop Karşılaştırması

| Özellik          | Block (403)                          | Drop (TCP RST)                       |
|------------------|--------------------------------------|--------------------------------------|
| **Davranış**     | HTTP 403 Forbidden döner             | Bağlantıyı aniden keser              |
| **Kullanıcı Görür**| "403 Forbidden" hata sayfası       | "Connection reset" / Timeout         |
| **Logging**      | Full HTTP transaction loglanır       | Partial log (sadece istek)           |
| **Use Case**     | Polite rejection, test environment   | Firewall testing, stealth mode       |
| **Güvenlik**     | Açık reddedilme                      | Gizli engelleme (port kapalı gibi)   |

**Örnek Kullanımlar:**

```rust
// Geliştirme ortamında (Block)
RuleAction::Block { 
    reason: "This endpoint is deprecated, use /v2/api instead".to_string() 
}

// Production firewall testi (Drop)
RuleAction::Drop  // Hiçbir cevap yok, sessizce kes
```

### Detaylı Dokümantasyon

Traffic Policy sisteminin tüm detayları için: **[docs/TRAFFIC_POLICY.md](docs/TRAFFIC_POLICY.md)**

## 🐛 Sorun Giderme

### "Address already in use" Hatası

**Problem:** Port zaten kullanılıyor.

```bash
# Hangi process kullanıyor?
lsof -i :9095  # macOS/Linux
netstat -ano | findstr :9095  # Windows
netstat -ano | findstr :9095  # Windows

# Farklı port kullan
cargo run -p proxy-agent -- --listen-port 9096 --admin-port 9092
```

### "Failed to connect to Orchestrator" Hatası

**Problem:** Agent Orchestrator'a bağlanamıyor.

```bash
# 1. Orchestrator çalışıyor mu?
curl http://127.0.0.1:9090/health/detailed

# 2. gRPC portu açık mı?
telnet 127.0.0.1 50051

# 3. Firewall kontrolü (Linux)
sudo iptables -L -n | grep 50051

# 4. Doğru URL'i kullan
cargo run -p proxy-agent -- --orchestrator-url http://127.0.0.1:50051
```

### "Database error" Hatası

**Problem:** Veritabanı bozuk veya migration hatası.

```bash
# Veritabanını sıfırla (DİKKAT: Tüm veri silinir!)
rm proxxy.db
cargo run -p orchestrator  # Otomatik yeniden oluşturulur

# Veritabanını backup'la
cp proxxy.db proxxy-backup.db

# Migration log'larını kontrol et
cargo run -p orchestrator 2>&1 | grep migration
```

### "GUI bağlanamıyor" Hatası

**Problem:** GUI, Orchestrator API'sine erişemiyor.

```bash
# HTTP API çalışıyor mu?
curl http://127.0.0.1:9090/health/detailed

# CORS sorunu var mı?
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: content-type" \
     -X OPTIONS \
     http://127.0.0.1:9090/agents

# GUI'nin config dosyasını kontrol et
cd proxxy-gui
cat src/config.ts  # API_BASE_URL doğru mu?
```

### "CA certificate not trusted" Hatası

**Problem:** Tarayıcı CA sertifikasını güvenmiyor.

```bash
# Sertifika yüklü mü kontrol et (macOS)
security find-certificate -c "Proxxy Root CA" -a

# Sertifika yüklü mü kontrol et (Linux)
ls /usr/local/share/ca-certificates/ | grep proxxy

# Firefox özel sertifika deposu
# about:preferences#privacy → Certificates → View Certificates → Authorities
# "Proxxy Root CA" arayın

# Tarayıcıyı yeniden başlat
# Sertifika yüklemeden sonra mutlaka!
```

### "Agent disconnected" Mesajı

**Problem:** Agent bağlantısı koptu.

```bash
# Agent health check
curl http://127.0.0.1:9091/health

# Orchestrator log'ları
cargo run -p orchestrator 2>&1 | grep -i "agent\|disconnect"

# Network timeout kontrolü
# Orchestrator --agent-timeout değerini artır
cargo run -p orchestrator -- --agent-timeout 600

# Agent heartbeat interval kontrolü (kod içinde)
# proxy-agent/src/client.rs → HEARTBEAT_INTERVAL
```

## 📚 Ek Kaynaklar

### Dokümantasyonlar

- **Traffic Policy Sistemi**: [docs/TRAFFIC_POLICY.md](docs/TRAFFIC_POLICY.md)
- **gRPC Protokolü**: [proto/proxy.proto](proto/proxy.proto)
- **GraphQL Schema**: `http://127.0.0.1:9090/graphql` (GraphiQL Playground)
- **REST API Dokümantasyonu**: `http://127.0.0.1:9090/swagger-ui`

### API Örnekleri

**REST API Kullanımı:**

```bash
# Tüm agent'ları listele (sistem metrikleri dahil)
curl http://127.0.0.1:9090/agents | jq

# Son 10 HTTP işlemi
curl http://127.0.0.1:9090/traffic?limit=10 | jq

# Trafik metrikleri (legacy)
curl http://127.0.0.1:9090/metrics | jq

# Sistem metrikleri (tüm agent'lar)
curl http://127.0.0.1:9090/system-metrics | jq

# Belirli bir agent'ın sistem metrikleri
curl "http://127.0.0.1:9090/system-metrics?agent_id=agent-1" | jq
```

**GraphQL Sorguları (Sistem Metrikleri ile):**

```graphql
# Tüm agent'lar, trafikler ve sistem metrikleri
query {
  agents {
    id
    name
    status
    lastHeartbeat
    currentMetrics {
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      networkRxBytesPerSec
      networkTxBytesPerSec
      processCpuPercent
      processMemoryBytes
    }
    transactions(limit: 5) {
      requestId
      method
      url
      status
      durationMs
    }
  }
}

# Belirli bir agent'ın detaylı sistem metrikleri
query {
  agent(id: "agent-1") {
    name
    hostname
    status
    currentMetrics {
      cpuUsagePercent
      memoryUsedBytes
      memoryTotalBytes
      networkRxBytesPerSec
      networkTxBytesPerSec
      processCpuPercent
      processMemoryBytes
      processUptimeSeconds
    }
    metricsHistory(limit: 60) {
      timestamp
      cpuUsagePercent
      memoryUsedBytes
      networkRxBytesPerSec
    }
  }
}

# Real-time sistem metrikleri subscription
subscription {
  systemMetricsUpdated(agentId: "agent-1") {
    agentId
    cpuUsagePercent
    memoryUsedBytes
    networkRxBytesPerSec
    networkTxBytesPerSec
    processCpuPercent
  }
}

# Tüm agent'ların sistem durumu
query {
  systemMetrics {
    agentId
    timestamp
    cpuUsagePercent
    memoryUsedBytes
    memoryTotalBytes
    networkRxBytesPerSec
    networkTxBytesPerSec
  }
}
```

**Sistem Metrikleri Örnekleri:**

```bash
# Agent'ların CPU kullanımını karşılaştır
curl http://127.0.0.1:9090/system-metrics | jq '.[] | {agent_id, cpu_usage_percent}'

# Yüksek memory kullanımı olan agent'ları bul
curl http://127.0.0.1:9090/system-metrics | jq '.[] | select(.memory_usage_percent > 80)'

# Network I/O en yüksek olan agent
curl http://127.0.0.1:9090/system-metrics | jq 'max_by(.network_rx_bytes_per_sec + .network_tx_bytes_per_sec)'

# Son 1 saatteki ortalama CPU kullanımı
curl "http://127.0.0.1:9090/system-metrics?since=1h" | jq 'map(.cpu_usage_percent) | add / length'
```

## 🔨 Geliştirme

# SQL
📊 SQLite Veritabanını Kontrol Etme
Yöntem 1: Komut Satırı (sqlite3)
bash
# Tüm tabloları listele
sqlite3 proxxy.db ".tables"
# Agents tablosunu görüntüle
sqlite3 proxxy.db "SELECT * FROM agents;"
# Daha okunaklı format (column mode)
sqlite3 proxxy.db -column -header "SELECT * FROM agents;"
# Traffic sayısı
sqlite3 proxxy.db "SELECT COUNT(*) as total FROM http_transactions;"
# Son 10 HTTP isteği
sqlite3 proxxy.db -column -header "
SELECT 
  req_method, 
  req_url, 
  res_status,
  datetime(req_timestamp, 'unixepoch') as timestamp
FROM http_transactions 
ORDER BY req_timestamp DESC 
LIMIT 10;"
# Agent başına trafik istatistikleri
sqlite3 proxxy.db -column -header "
SELECT 
  a.name,
  COUNT(h.request_id) as total_requests,
  AVG(h.duration_ms) as avg_duration_ms
FROM agents a
LEFT JOIN http_transactions h ON a.id = h.agent_id
GROUP BY a.id, a.name;"
Yöntem 2: İnteraktif Mod
bash
# SQLite shell'e gir
sqlite3 proxxy.db
# İçinde şu komutları kullan:
.tables                    # Tabloları listele
.schema agents             # Tablo yapısını gör
.mode column               # Column formatında göster
.headers on                # Header'ları göster
SELECT * FROM agents;      # Tüm agent'lar
# Çıkmak için
.quit
Yöntem 3: GUI Araçlar
DB Browser for SQLite (Ücretsiz):

bash
# macOS
brew install --cask db-browser-for-sqlite
# Sonra aç
open -a "DB Browser for SQLite" proxxy.db
Kullanışlı Sorgular
sql
-- 1. Aktif agent'lar
SELECT 
  name, 
  hostname,
  status,
  datetime(last_heartbeat, 'unixepoch') as last_seen
FROM agents 
WHERE status = 'Online';
-- 2. En yavaş 10 istek
SELECT 
  req_method,
  req_url,
  duration_ms,
  res_status
FROM http_transactions 
WHERE duration_ms IS NOT NULL
ORDER BY duration_ms DESC 
LIMIT 10;
-- 3. HTTP metod dağılımı
SELECT 
  req_method,
  COUNT(*) as count
FROM http_transactions
GROUP BY req_method
ORDER BY count DESC;
-- 4. Status code dağılımı
SELECT 
  res_status,
  COUNT(*) as count
FROM http_transactions
WHERE res_status IS NOT NULL
GROUP BY res_status
ORDER BY res_status;
-- 5. Son 1 saatteki trafik
SELECT COUNT(*) as requests_last_hour
FROM http_transactions
WHERE req_timestamp > strftime('%s', 'now', '-1 hour');
-- 6. En çok istenen domainler
SELECT 
  SUBSTR(req_url, 1, INSTR(SUBSTR(req_url, 9), '/') + 8) as domain,
  COUNT(*) as requests
FROM http_transactions
GROUP BY domain
ORDER BY requests DESC
LIMIT 10;
Hızlı Kontrol Script'i
bash
#!/bin/bash
# check-db.sh
echo "=== PROXXY DATABASE STATUS ==="
echo ""
echo "📊 Agents:"
sqlite3 proxxy.db -column -header "
SELECT name, status, datetime(last_heartbeat, 'unixepoch') as last_seen 
FROM agents;"
echo ""
echo "📈 Traffic Stats:"
sqlite3 proxxy.db -column -header "
SELECT 
  COUNT(*) as total_requests,
  AVG(duration_ms) as avg_duration_ms,
  MIN(duration_ms) as min_duration_ms,
  MAX(duration_ms) as max_duration_ms
FROM http_transactions
WHERE duration_ms IS NOT NULL;"
echo ""
echo "🔢 HTTP Methods:"
sqlite3 proxxy.db -column -header "
SELECT req_method, COUNT(*) as count 
FROM http_transactions 
GROUP BY req_method 
ORDER BY count DESC;"

### Test Çalıştırma

```bash
# Tüm testler
cargo test --workspace

# Belirli bir paket
cargo test -p proxy-core
cargo test -p orchestrator

# Belirli bir test
cargo test -p proxy-core test_scope_config_wildcard

# Verbose output
cargo test -- --nocapture

# Test coverage (cargo-tarpaulin gerekli)
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

### Code Quality

```bash
# Linting
cargo clippy --workspace -- -D warnings

# Formatting
cargo fmt --workspace --check  # Sadece kontrol
cargo fmt --workspace           # Otomatik düzelt

# Audit (güvenlik açıkları)
cargo audit

# Outdated dependencies
cargo outdated
```

### Build Variants

```bash
# Debug build (hızlı compile, yavaş runtime)
cargo build

# Release build (optimize edilmiş)
cargo build --release

# Profiling build
cargo build --profile profiling

# Size-optimized build
cargo build --release --profile=release-lto
```

### Profiling

```bash
# CPU profiling (cargo-flamegraph gerekli)
cargo install flamegraph
cargo flamegraph -p orchestrator

# Memory profiling (valgrind gerekli)
cargo build
valgrind --leak-check=full ./target/debug/orchestrator

# Benchmarking (criterion)
cargo bench --workspace
```

## 🚀 Production Deployment

### Systemd Service (Linux)

```ini
# /etc/systemd/system/proxxy-orchestrator.service
[Unit]
Description=Proxxy Orchestrator
After=network.target

[Service]
Type=simple
User=proxxy
WorkingDirectory=/opt/proxxy
ExecStart=/opt/proxxy/orchestrator \
  --grpc-port 50051 \
  --http-port 9090 \
  --database-url sqlite:/var/lib/proxxy/data.db
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
# Service'i etkinleştir
sudo systemctl daemon-reload
sudo systemctl enable proxxy-orchestrator
sudo systemctl start proxxy-orchestrator
sudo systemctl status proxxy-orchestrator
```

### Docker Deployment

```dockerfile
# Dockerfile.orchestrator
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p orchestrator

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/orchestrator /usr/local/bin/
EXPOSE 50051 9090
CMD ["orchestrator"]
```

```bash
# Build
docker build -f Dockerfile.orchestrator -t proxxy-orchestrator:latest .

# Run
docker run -d \
  -p 50051:50051 \
  -p 9090:9090 \
  -v proxxy-data:/data \
  --name orchestrator \
  proxxy-orchestrator:latest
```

### Monitoring Stack

```yaml
# Prometheus scrape config
scrape_configs:
  - job_name: 'proxxy-agents'
    static_configs:
      - targets: 
          - 'agent1:9091'
          - 'agent2:9092'
          - 'agent3:9093'
    metrics_path: '/metrics'
```

## 📝 Lisans

[Lisans bilgisi buraya eklenecek]

## 🤝 Katkıda Bulunma

Pull request'ler memnuniyetle karşılanır! Büyük değişiklikler için önce bir issue açarak ne değiştirmek istediğinizi tartışalım.

---

**İletişim**: [Proje maintainer bilgileri]