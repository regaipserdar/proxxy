#!/usr/bin/env python3
"""
Test HTTP istekleri gönderen script
Proxy üzerinden testphp.vulnweb.com'a çeşitli HTTP metodları ile istek gönderir
"""

import requests
import time
import json

# Proxy ayarları
PROXY_URL = "http://127.0.0.1:9095"  # Proxy agent 9095 portunda çalışıyor
TARGET_BASE = "http://testphp.vulnweb.com"

proxies = {
    'http': PROXY_URL,
    'https': PROXY_URL
}

def send_request(method, path, data=None, headers=None):
    """HTTP isteği gönder"""
    url = f"{TARGET_BASE}{path}"
    
    default_headers = {
        "User-Agent": "Proxxy-Test-Client/1.0",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.5",
        "Accept-Encoding": "gzip, deflate",
        "Connection": "keep-alive",
        "Upgrade-Insecure-Requests": "1"
    }
    
    if headers:
        default_headers.update(headers)
    
    try:
        print(f"🚀 Gönderiliyor: {method} {url}")
        
        response = requests.request(
            method=method,
            url=url,
            proxies=proxies,
            headers=default_headers,
            data=data,
            timeout=15,
            allow_redirects=True
        )
        
        print(f"✅ Yanıt: {response.status_code} - {len(response.content)} bytes")
        return response
        
    except Exception as e:
        print(f"❌ Hata: {e}")
        return None

def main():
    print("🎯 testphp.vulnweb.com'a test istekleri gönderiliyor...")
    print("=" * 60)
    
    # Test istekleri
    test_requests = [
        # GET istekleri
        ("GET", "/", None, None),
        ("GET", "/artists.php", None, None),
        ("GET", "/categories.php", None, None),
        ("GET", "/pictures/", None, None),
        ("GET", "/login.php", None, None),
        
        # POST istekleri
        ("POST", "/login.php", "uname=test&pass=test123", {"Content-Type": "application/x-www-form-urlencoded"}),
        ("POST", "/search.php", "searchFor=test", {"Content-Type": "application/x-www-form-urlencoded"}),
        
        # PUT isteği
        ("PUT", "/test-endpoint", '{"test": "data"}', {"Content-Type": "application/json"}),
        
        # DELETE isteği  
        ("DELETE", "/test-resource", None, None),
        
        # OPTIONS isteği
        ("OPTIONS", "/", None, None),
        
        # HEAD isteği
        ("HEAD", "/", None, None),
        
        # PATCH isteği
        ("PATCH", "/test-resource", '{"update": "value"}', {"Content-Type": "application/json"}),
    ]
    
    for i, (method, path, data, headers) in enumerate(test_requests, 1):
        print(f"\n[{i}/{len(test_requests)}] ", end="")
        send_request(method, path, data, headers)
        time.sleep(1)  # İstekler arası kısa bekleme
    
    print(f"\n{'='*60}")
    print("✅ Tüm test istekleri gönderildi!")
    print("📊 GraphQL ile logları kontrol edebilirsiniz:")
    print("   curl -X POST http://127.0.0.1:9090/graphql \\")
    print('     -H "Content-Type: application/json" \\')
    print('     -d \'{"query": "query { requests(agentId: null) { requestId method url status timestamp } }"}\'')

if __name__ == "__main__":
    main()