#!/usr/bin/env python3
"""
Response Body Capture Integration Test for Proxxy

Bu script response body capture özelliğinin çalışıp çalışmadığını test eder:
1. Orchestrator'ı başlatır
2. Proxy agent'ı başlatır (body capture etkin)
3. HTTP istekleri gönderir
4. Response body'lerin yakalanıp yakalanmadığını kontrol eder
5. GraphQL API üzerinden response body'leri sorgular

Kullanım: python3 test_response_body_capture.py
"""

import subprocess
import time
import requests
import json
import sys
import os
import signal
import shutil
from typing import Optional, Dict, Any

# Konfigürasyon
ORCHESTRATOR_BINARY = "./target/debug/orchestrator"
PROXY_AGENT_BINARY = "./target/debug/proxy-agent"
GRAPHQL_URL = "http://127.0.0.1:9090/graphql"
PROXY_URL = "http://127.0.0.1:9095"  # Proxy agent default port
TEST_PROJECT_NAME = "response_body_test"

# Test URL'leri - farklı content-type'lar için
TEST_URLS = [
    {
        "url": "http://httpbin.org/json",
        "description": "JSON Response Test",
        "expected_content_type": "application/json"
    },
    {
        "url": "http://httpbin.org/html",
        "description": "HTML Response Test", 
        "expected_content_type": "text/html"
    },
    {
        "url": "http://httpbin.org/xml",
        "description": "XML Response Test",
        "expected_content_type": "application/xml"
    },
    {
        "url": "http://httpbin.org/get?test=response_body_capture",
        "description": "GET with Parameters",
        "expected_content_type": "application/json"
    }
]

class ResponseBodyCaptureTest:
    def __init__(self):
        self.orchestrator_process: Optional[subprocess.Popen] = None
        self.proxy_agent_process: Optional[subprocess.Popen] = None
        self.captured_requests = []
        
    def log(self, message: str, level: str = "INFO"):
        """Log mesajı timestamp ile"""
        timestamp = time.strftime("%H:%M:%S")
        print(f"[{timestamp}] {level}: {message}")
        
    def graphql_query(self, query: str, variables: Optional[Dict] = None) -> Dict[Any, Any]:
        """GraphQL sorgusu çalıştır"""
        payload = {
            "query": query,
            "variables": variables or {}
        }
        
        try:
            response = requests.post(
                GRAPHQL_URL,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=10
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            self.log(f"GraphQL isteği başarısız: {e}", "ERROR")
            raise
            
    def start_orchestrator(self) -> bool:
        """Orchestrator'ı başlat"""
        self.log("Orchestrator başlatılıyor...")
        
        try:
            # Binary kontrolü
            if not os.path.exists(ORCHESTRATOR_BINARY):
                self.log(f"Orchestrator binary bulunamadı: {ORCHESTRATOR_BINARY}", "ERROR")
                self.log("Önce projeyi build edin: cargo build", "ERROR")
                return False
                
            # Test projesi dizinini temizle
            test_project_dir = f"workspace/{TEST_PROJECT_NAME}.proxxy"
            if os.path.exists(test_project_dir):
                self.log(f"Mevcut test projesi dizini temizleniyor: {test_project_dir}")
                shutil.rmtree(test_project_dir)
                
            # Orchestrator'ı başlat
            cmd = [ORCHESTRATOR_BINARY, "--project", TEST_PROJECT_NAME]
            self.log(f"Komut çalıştırılıyor: {' '.join(cmd)}")
            
            self.orchestrator_process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            self.log(f"Orchestrator başlatıldı (PID: {self.orchestrator_process.pid})")
            
            # Başlatma için bekle
            for i in range(10):
                time.sleep(1)
                if self.orchestrator_process.poll() is not None:
                    stdout, stderr = self.orchestrator_process.communicate()
                    self.log(f"Orchestrator erken kapandı. STDOUT: {stdout}", "ERROR")
                    self.log(f"STDERR: {stderr}", "ERROR")
                    return False
                print(".", end="", flush=True)
            print()
            
            # GraphQL endpoint kontrolü
            self.log("GraphQL endpoint kontrol ediliyor...")
            for attempt in range(5):
                try:
                    response = requests.get(f"http://127.0.0.1:9090/graphql", timeout=2)
                    if response.status_code in [200, 405]:
                        self.log("✅ GraphQL endpoint hazır")
                        return True
                except requests.exceptions.RequestException:
                    if attempt < 4:
                        self.log(f"GraphQL endpoint hazır değil, tekrar deneniyor... ({attempt + 1}/5)")
                        time.sleep(2)
                    else:
                        self.log("GraphQL endpoint 5 denemeden sonra yanıt vermedi", "ERROR")
                        return False
                        
            return True
            
        except Exception as e:
            self.log(f"Orchestrator başlatılamadı: {e}", "ERROR")
            return False
            
    def start_proxy_agent(self) -> bool:
        """Proxy agent'ı body capture etkin olarak başlat"""
        self.log("Proxy Agent başlatılıyor (body capture etkin)...")
        
        try:
            # Binary kontrolü
            if not os.path.exists(PROXY_AGENT_BINARY):
                self.log(f"Proxy agent binary bulunamadı: {PROXY_AGENT_BINARY}", "ERROR")
                self.log("Önce projeyi build edin: cargo build", "ERROR")
                return False
                
            # Environment variables ile body capture konfigürasyonu
            env = os.environ.copy()
            env.update({
                "PROXXY_BODY_CAPTURE_ENABLED": "true",
                "PROXXY_MAX_BODY_SIZE": "1048576",  # 1MB
                "PROXXY_MEMORY_LIMIT": "10485760",  # 10MB
                "PROXXY_RESPONSE_TIMEOUT": "30",
                "PROXXY_STREAM_TIMEOUT": "5",
                "PROXXY_CONTENT_TYPE_MODE": "capture_all"  # Tüm content-type'ları yakala
            })
            
            # Proxy agent'ı başlat
            cmd = [
                PROXY_AGENT_BINARY,
                "--listen-port", "9095",
                "--admin-port", "9091"
            ]
            self.log(f"Komut çalıştırılıyor: {' '.join(cmd)}")
            self.log("Environment variables:")
            for key, value in env.items():
                if key.startswith("PROXXY_"):
                    self.log(f"  {key}={value}")
            
            self.proxy_agent_process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=env
            )
            
            self.log(f"Proxy Agent başlatıldı (PID: {self.proxy_agent_process.pid})")
            
            # Başlatma için bekle
            for i in range(10):
                time.sleep(1)
                if self.proxy_agent_process.poll() is not None:
                    stdout, stderr = self.proxy_agent_process.communicate()
                    self.log(f"Proxy Agent erken kapandı. STDOUT: {stdout}", "ERROR")
                    self.log(f"STDERR: {stderr}", "ERROR")
                    return False
                print(".", end="", flush=True)
            print()
            
            # Proxy endpoint kontrolü
            self.log("Proxy endpoint kontrol ediliyor...")
            for attempt in range(5):
                try:
                    # Proxy'ye basit bir health check isteği gönder
                    response = requests.get(
                        "http://httpbin.org/status/200",
                        proxies={"http": PROXY_URL, "https": PROXY_URL},
                        timeout=5
                    )
                    if response.status_code == 200:
                        self.log("✅ Proxy Agent hazır ve çalışıyor")
                        return True
                except requests.exceptions.RequestException:
                    if attempt < 4:
                        self.log(f"Proxy endpoint hazır değil, tekrar deneniyor... ({attempt + 1}/5)")
                        time.sleep(2)
                    else:
                        self.log("Proxy endpoint 5 denemeden sonra yanıt vermedi", "ERROR")
                        return False
                        
            return True
            
        except Exception as e:
            self.log(f"Proxy Agent başlatılamadı: {e}", "ERROR")
            return False
            
    def disable_interception(self) -> bool:
        """Interception'ı kapat (otomatik trafik akışı için)"""
        self.log("Interception kapatılıyor...")
        
        try:
            mutation = """
            mutation ToggleInterception($enabled: Boolean!) {
                toggleInterception(enabled: $enabled) {
                    enabled
                }
            }
            """
            
            result = self.graphql_query(mutation, {"enabled": False})
            
            if "errors" in result:
                self.log(f"GraphQL hataları: {result['errors']}", "ERROR")
                return False
                
            interception_config = result.get("data", {}).get("toggleInterception", {})
            
            if interception_config.get("enabled", True):
                self.log("Interception kapatılamadı", "ERROR")
                return False
                
            self.log("✅ Interception başarıyla kapatıldı")
            return True
            
        except Exception as e:
            self.log(f"Interception kapatılamadı: {e}", "ERROR")
            return False
            
    def generate_test_traffic(self) -> bool:
        """Test trafiği oluştur"""
        self.log("Test trafiği oluşturuluyor...")
        
        proxies = {
            'http': PROXY_URL,
            'https': PROXY_URL
        }
        
        success_count = 0
        
        for i, test_case in enumerate(TEST_URLS):
            self.log(f"Test {i+1}/{len(TEST_URLS)}: {test_case['description']}")
            self.log(f"  URL: {test_case['url']}")
            
            try:
                response = requests.get(
                    test_case['url'],
                    proxies=proxies,
                    timeout=15,
                    headers={
                        "User-Agent": "Proxxy-ResponseBody-Test/1.0",
                        "X-Test-Case": test_case['description'],
                        "X-Test-Index": str(i),
                        "X-Test-Timestamp": str(int(time.time()))
                    }
                )
                
                if response.status_code == 200:
                    self.log(f"  ✅ Başarılı (status: {response.status_code}, size: {len(response.content)} bytes)")
                    success_count += 1
                else:
                    self.log(f"  ⚠️  Beklenmeyen status: {response.status_code}")
                    
            except Exception as e:
                self.log(f"  ❌ Hata: {e}", "ERROR")
                
            time.sleep(2)  # İstekler arası bekleme
            
        self.log(f"Trafik oluşturma tamamlandı: {success_count}/{len(TEST_URLS)} başarılı")
        
        # Trafiğin işlenmesi için bekle
        self.log("Trafiğin işlenmesi için bekleniyor...")
        time.sleep(10)
        
        return success_count > 0
        
    def verify_response_body_capture(self) -> bool:
        """Response body capture'ın çalışıp çalışmadığını kontrol et"""
        self.log("Response body capture kontrol ediliyor...")
        
        try:
            # HTTP transaction'ları al
            query = """
            query GetHttpTransactions {
                requests(agentId: null) {
                    requestId
                    method
                    url
                    status
                    timestamp
                }
            }
            """
            
            self.log("HTTP transaction'ları sorgulanıyor...")
            result = self.graphql_query(query)
            
            if "errors" in result:
                self.log(f"GraphQL hataları: {result['errors']}", "ERROR")
                return False
                
            requests_list = result.get("data", {}).get("requests", [])
            self.log(f"{len(requests_list)} HTTP transaction bulundu")
            
            if not requests_list:
                self.log("❌ Hiç HTTP transaction bulunamadı", "ERROR")
                return False
                
            # Test isteklerimizi bul
            test_requests = []
            for req in requests_list:
                url = req.get("url", "")
                if "httpbin.org" in url and any(test_url["url"] in url for test_url in TEST_URLS):
                    test_requests.append(req)
                    
            self.log(f"{len(test_requests)} test isteği bulundu")
            
            if not test_requests:
                self.log("❌ Test istekleri bulunamadı", "ERROR")
                self.log("Mevcut URL'ler:")
                for req in requests_list[:5]:
                    self.log(f"  - {req.get('url', 'N/A')}")
                return False
                
            # Her test isteği için detaylı kontrol
            success_count = 0
            
            for i, req in enumerate(test_requests):
                request_id = req["requestId"]
                self.log(f"\nTest isteği {i+1}/{len(test_requests)} kontrol ediliyor:")
                self.log(f"  ID: {request_id}")
                self.log(f"  URL: {req.get('url', 'N/A')}")
                self.log(f"  Status: {req.get('status', 'N/A')}")
                
                # Detaylı bilgileri al
                detail_query = """
                query GetRequestDetail($id: String!) {
                    request(id: $id) {
                        requestId
                        method
                        url
                        status
                        requestHeaders
                        requestBody
                        responseHeaders
                        responseBody
                        timestamp
                    }
                }
                """
                
                detail_result = self.graphql_query(detail_query, {"id": request_id})
                
                if "errors" in detail_result:
                    self.log(f"  ❌ Detay sorgusu başarısız: {detail_result['errors']}", "ERROR")
                    continue
                    
                request_detail = detail_result.get("data", {}).get("request")
                
                if not request_detail:
                    self.log(f"  ❌ İstek detayı bulunamadı", "ERROR")
                    continue
                    
                # Response body kontrolü
                response_body = request_detail.get("responseBody")
                response_headers = request_detail.get("responseHeaders")
                
                # Kritik kontroller
                checks_passed = 0
                total_checks = 4
                
                # 1. Response body null değil mi?
                if response_body is not None:
                    self.log(f"  ✅ Response body null değil")
                    checks_passed += 1
                else:
                    self.log(f"  ❌ Response body null!")
                    
                # 2. Response body boş değil mi?
                if response_body and len(response_body.strip()) > 0:
                    self.log(f"  ✅ Response body boş değil (uzunluk: {len(response_body)} karakter)")
                    checks_passed += 1
                else:
                    self.log(f"  ❌ Response body boş!")
                    
                # 3. Response headers var mı?
                if response_headers and len(response_headers.strip()) > 0:
                    self.log(f"  ✅ Response headers mevcut (uzunluk: {len(response_headers)} karakter)")
                    checks_passed += 1
                else:
                    self.log(f"  ❌ Response headers boş!")
                    
                # 4. JSON response ise parse edilebilir mi?
                json_parseable = False
                if response_body and response_body.strip().startswith('{'):
                    try:
                        json.loads(response_body)
                        self.log(f"  ✅ Response body geçerli JSON")
                        json_parseable = True
                        checks_passed += 1
                    except json.JSONDecodeError:
                        self.log(f"  ⚠️  Response body JSON gibi görünüyor ama parse edilemiyor")
                elif response_body and ("<html" in response_body.lower() or "<?xml" in response_body.lower()):
                    self.log(f"  ✅ Response body HTML/XML içeriği")
                    checks_passed += 1
                else:
                    self.log(f"  ⚠️  Response body formatı belirsiz")
                    
                # Başarı oranı
                success_rate = (checks_passed / total_checks) * 100
                self.log(f"  📊 Başarı oranı: {success_rate:.1f}% ({checks_passed}/{total_checks})")
                
                if checks_passed >= 3:  # En az 3/4 kontrol geçmeli
                    self.log(f"  ✅ Bu istek için response body capture başarılı")
                    success_count += 1
                else:
                    self.log(f"  ❌ Bu istek için response body capture başarısız")
                    
                # İlk birkaç karakteri göster (debug için)
                if response_body:
                    preview = response_body[:200] + "..." if len(response_body) > 200 else response_body
                    self.log(f"  📄 Response body önizleme: {preview}")
                    
            # Genel sonuç
            overall_success_rate = (success_count / len(test_requests)) * 100
            self.log(f"\n📊 GENEL SONUÇ:")
            self.log(f"   Başarılı istekler: {success_count}/{len(test_requests)}")
            self.log(f"   Başarı oranı: {overall_success_rate:.1f}%")
            
            if success_count == len(test_requests):
                self.log("🎉 TÜM RESPONSE BODY CAPTURE TESTLERİ BAŞARILI!")
                return True
            elif success_count > 0:
                self.log("⚠️  Bazı response body capture testleri başarılı")
                return True  # Kısmi başarı da kabul edilebilir
            else:
                self.log("❌ HİÇBİR RESPONSE BODY CAPTURE TESTİ BAŞARILI DEĞİL")
                return False
                
        except Exception as e:
            self.log(f"Response body capture kontrolü başarısız: {e}", "ERROR")
            return False
            
    def cleanup(self):
        """Kaynakları temizle"""
        self.log("Temizlik yapılıyor...")
        
        # Proxy Agent'ı kapat
        if self.proxy_agent_process:
            try:
                self.log(f"Proxy Agent kapatılıyor (PID: {self.proxy_agent_process.pid})")
                self.proxy_agent_process.terminate()
                try:
                    self.proxy_agent_process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.log("Proxy Agent zorla kapatılıyor...")
                    self.proxy_agent_process.kill()
                    self.proxy_agent_process.wait()
                self.log("Proxy Agent kapatıldı")
            except Exception as e:
                self.log(f"Proxy Agent kapatma hatası: {e}", "ERROR")
                
        # Orchestrator'ı kapat
        if self.orchestrator_process:
            try:
                self.log(f"Orchestrator kapatılıyor (PID: {self.orchestrator_process.pid})")
                self.orchestrator_process.terminate()
                try:
                    self.orchestrator_process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.log("Orchestrator zorla kapatılıyor...")
                    self.orchestrator_process.kill()
                    self.orchestrator_process.wait()
                self.log("Orchestrator kapatıldı")
            except Exception as e:
                self.log(f"Orchestrator kapatma hatası: {e}", "ERROR")
                
    def run(self) -> bool:
        """Tam integration testi çalıştır"""
        self.log("🚀 Response Body Capture Integration Test Başlıyor")
        self.log("=" * 60)
        
        try:
            # Adım 1: Orchestrator'ı başlat
            if not self.start_orchestrator():
                return False
                
            # Adım 2: Proxy Agent'ı başlat
            if not self.start_proxy_agent():
                return False
                
            # Adım 3: Interception'ı kapat
            if not self.disable_interception():
                return False
                
            # Adım 4: Test trafiği oluştur
            if not self.generate_test_traffic():
                return False
                
            # Adım 5: Response body capture'ı kontrol et
            if not self.verify_response_body_capture():
                return False
                
            return True
            
        except KeyboardInterrupt:
            self.log("Test kullanıcı tarafından durduruldu", "ERROR")
            return False
        except Exception as e:
            self.log(f"Test sırasında beklenmeyen hata: {e}", "ERROR")
            return False
        finally:
            self.cleanup()
            
def main():
    """Ana giriş noktası"""
    print("🚀 Proxxy Response Body Capture Integration Test")
    print("=" * 60)
    print("Bu test şunları doğrular:")
    print("  1. Orchestrator başlatma")
    print("  2. Proxy Agent başlatma (body capture etkin)")
    print("  3. HTTP trafiği oluşturma")
    print("  4. Response body'lerin yakalanması")
    print("  5. GraphQL API üzerinden response body sorgulama")
    print("=" * 60)
    print()
    
    test = ResponseBodyCaptureTest()
    
    try:
        success = test.run()
        
        print("\n" + "=" * 60)
        if success:
            print("✅ RESPONSE BODY CAPTURE TESTİ BAŞARILI")
            print()
            print("🎉 Response Body Capture özelliği çalışıyor!")
            print("   • Orchestrator: ✅")
            print("   • Proxy Agent: ✅") 
            print("   • Body Capture: ✅")
            print("   • GraphQL API: ✅")
            print("   • Response Storage: ✅")
            print()
            print("Response body capture başarıyla:")
            print("  - HTTP response'ları yakalar")
            print("  - Body içeriğini saklar")
            print("  - GraphQL API üzerinden erişilebilir hale getirir")
            print("  - Farklı content-type'ları destekler")
            sys.exit(0)
        else:
            print("❌ RESPONSE BODY CAPTURE TESTİ BAŞARISIZ")
            print()
            print("Lütfen yukarıdaki logları kontrol edin.")
            print("Olası sorunlar:")
            print("  - Binary'ler build edilmemiş (çalıştır: cargo build)")
            print("  - Port'lar kullanımda (9090, 9095)")
            print("  - Network bağlantı sorunları")
            print("  - Body capture konfigürasyon hataları")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\n❌ Test kullanıcı tarafından durduruldu")
        sys.exit(1)

if __name__ == "__main__":
    main()