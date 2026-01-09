# Proxxy API Dokümantasyonu

Bu klasör, Proxxy Orchestrator API'sinin kapsamlı dokümantasyonunu içerir.

## 📚 Dokümantasyon Dosyaları

### 1. [API Reference](./api-reference.md) ⭐
**En detaylı API dokümantasyonu**

Tüm REST ve GraphQL endpoint'lerinin tam referansı:
- ✅ Her endpoint için detaylı açıklamalar
- ✅ İstek/yanıt formatları ve örnekleri
- ✅ Path ve query parametreleri
- ✅ Hata kodları ve yanıtları
- ✅ GraphQL queries, mutations ve subscriptions
- ✅ WebSocket bağlantı detayları
- ✅ Veri modelleri ve TypeScript tipleri

**Kimler için:** Backend geliştiriciler, API entegrasyonu yapan herkes

---

### 2. [API Usage Examples](./api-examples.md) 💡
**Pratik kod örnekleri**

Farklı dillerde kullanıma hazır örnekler:
- ✅ Bash/cURL örnekleri
- ✅ JavaScript/TypeScript client'ları
- ✅ Python sync/async örnekleri
- ✅ React Hooks (useAgents, useTrafficStream, useMetrics)
- ✅ GraphQL subscription örnekleri
- ✅ Monitoring script'leri

**Kimler için:** Hızlı başlamak isteyen geliştiriciler, örnek kod arayanlar

---

### 3. [Backend API Specification](./backend-api.md) 🎯
**UI geliştirme için backend özellikleri**

Frontend geliştiriciler için özel hazırlanmış:
- ✅ GraphQL şema detayları
- ✅ UI component gereksinimleri
- ✅ Real-time subscription kullanımı
- ✅ Veritabanı şeması
- ✅ UI teknoloji önerileri
- ✅ Örnek UI flow'ları

**Kimler için:** Frontend/UI geliştiriciler

---

### 4. [Traffic Policy System](./traffic-policy.md) 🚦
**Dinamik trafik kuralları sistemi**

Runtime'da değiştirilebilir trafik politikaları:
- ✅ Policy engine mimarisi
- ✅ Kural tanımlama örnekleri
- ✅ Match & Replace sistemi
- ✅ Request interception

**Kimler için:** Proxy konfigürasyonu yapanlar, güvenlik testçileri

---

### 5. [Flow Engine](./flow-engine.md) 🔄
**Automation ve replay sistemi**

Zero-copy automation özellikleri:
- ✅ Visual & Protocol mode
- ✅ Login Sequence Recorder
- ✅ Self-healing selectors
- ✅ Performance optimizasyonları

**Kimler için:** Automation geliştiriciler, test mühendisleri

---

### 6. [Architecture](./architecture.md) 🏗️
**Sistem mimarisi**

Proxxy'nin genel mimarisi:
- ✅ Bileşen diyagramları
- ✅ İletişim protokolleri
- ✅ Veri akışı
- ✅ Deployment stratejileri

**Kimler için:** Sistem mimarları, DevOps mühendisleri

---

## 🚀 Hızlı Başlangıç

### 1. API'yi Keşfetmek İçin

```bash
# Orchestrator'ı başlat
cargo run -p orchestrator

# Tarayıcıda aç:
# - GraphQL Playground: http://localhost:9090/graphql
# - Swagger UI: http://localhost:9090/swagger-ui
```

### 2. İlk API Çağrınız

**REST API:**
```bash
curl http://localhost:9090/health
```

**GraphQL:**
```bash
curl -X POST http://localhost:9090/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ agents { id name status } }"}'
```

### 3. Kod Örneklerini Kullanma

[API Examples](./api-examples.md) dosyasından kopyala-yapıştır yapabilirsiniz:
- JavaScript client sınıfı
- Python API wrapper
- React hooks
- Monitoring script'leri

---

## 📖 Dokümantasyon Kullanım Kılavuzu

### Senaryo 1: "API'yi öğrenmek istiyorum"
1. [API Reference](./api-reference.md) - Tüm endpoint'leri inceleyin
2. [API Examples](./api-examples.md) - Örnekleri çalıştırın
3. GraphQL Playground'da deneyler yapın

### Senaryo 2: "Frontend geliştiriyorum"
1. [Backend API Specification](./backend-api.md) - UI gereksinimleri
2. [API Examples](./api-examples.md) - React hooks bölümü
3. GraphQL subscriptions ile real-time özellikler

### Senaryo 3: "Proxy konfigürasyonu yapıyorum"
1. [Traffic Policy System](./traffic-policy.md) - Kural sistemi
2. [API Reference](./api-reference.md) - Policy endpoint'leri
3. [API Examples](./api-examples.md) - Policy yönetimi örnekleri

### Senaryo 4: "Test automation yazıyorum"
1. [Flow Engine](./flow-engine.md) - Automation özellikleri
2. [API Examples](./api-examples.md) - Automation örnekleri
3. [API Reference](./api-reference.md) - Replay endpoint'leri

---

## 🔍 Endpoint Hızlı Referans

### Health & System
- `GET /health` - Basit health check
- `GET /api/health/detailed` - Detaylı sistem durumu
- `GET /api/system/health` - Sistem + agent istatistikleri

### Agents
- `GET /api/agents` - Tüm agent'ları listele
- `GET /agents/{id}` - Belirli agent bilgisi
- `POST /agents` - Yeni agent kaydet

### Traffic
- `GET /api/traffic/recent` - Son HTTP işlemleri
- `GET /traffic?agent_id={id}` - Agent'a özel trafik

### Metrics
- `GET /metrics` - Sistem metrikleri
- `GET /metrics/{agent_id}` - Agent metrikleri

### GraphQL
- `POST /graphql` - GraphQL sorguları
- `GET /graphql` - GraphiQL Playground

---

## 🛠️ Geliştirme Araçları

### Interaktif Dokümantasyon
- **GraphiQL**: http://localhost:9090/graphql
- **Swagger UI**: http://localhost:9090/swagger-ui

### API Test Araçları
- **Postman**: OpenAPI spec'i import edin
- **Insomnia**: GraphQL endpoint'i ekleyin
- **cURL**: Bash örneklerini kullanın

### Client Kütüphaneleri
- **JavaScript**: [API Examples](./api-examples.md#javascript-client-class)
- **Python**: [API Examples](./api-examples.md#python-api-client)
- **React**: [API Examples](./api-examples.md#react-hooks)

---

## 📝 Katkıda Bulunma

Dokümantasyonu geliştirmek için:

1. Hata bulduysanız issue açın
2. Yeni örnek eklemek için PR gönderin
3. Eksik bölümleri tamamlayın

### Dokümantasyon Standartları
- ✅ Her endpoint için örnek istek/yanıt
- ✅ Hata durumları açıklanmalı
- ✅ Kod örnekleri çalışır durumda olmalı
- ✅ TypeScript tipleri güncel tutulmalı

---

## 🔗 İlgili Kaynaklar

### Kod Kaynakları
- [orchestrator/src/http.rs](../../orchestrator/src/http.rs) - REST API implementasyonu
- [orchestrator/src/lib.rs](../../orchestrator/src/lib.rs) - GraphQL şema
- [proto/proxy.proto](../../proto/proxy.proto) - gRPC protokol tanımları

### Harici Dokümantasyon
- [Axum Documentation](https://docs.rs/axum/) - Web framework
- [async-graphql](https://async-graphql.github.io/) - GraphQL kütüphanesi
- [Tonic](https://docs.rs/tonic/) - gRPC framework

---

**Son Güncelleme:** 2026-01-09  
**Versiyon:** 0.1.1  
**Durum:** ✅ Aktif Geliştirme

---

## 📞 Destek

Sorularınız için:
- GitHub Issues
- Dokümantasyon: Bu klasördeki dosyalar
- Canlı örnekler: GraphQL Playground ve Swagger UI
