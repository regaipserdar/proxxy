#!/usr/bin/env python3
"""
🔥 Ultimate Integration Test for Proxxy
Features:
- Robust Startup (Port Checks)
- Full Traffic Stress Test (100+ endpoints)
- Deep Content Verification (Data Integrity)
"""

import subprocess
import time
import requests
import json
import sys
import os
import shutil
import random
import urllib3
import socket
from typing import Optional, Dict, Any

urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# --- CONFIGURATION ---
ORCHESTRATOR_BINARY = "./target/release/orchestrator"
PROXY_AGENT_BINARY = "./target/release/proxy-agent"
TEST_SERVER_BINARY = "./target/release/test_server"

TEST_SERVER_URL = "http://127.0.0.1:8000"
GRAPHQL_URL = "http://127.0.0.1:9090/graphql"
PROXY_URL = "http://127.0.0.1:8080"
TEST_PROJECT_NAME = "integration_test_project"

ECHO_PAYLOAD = {"test_id": random.randint(1000, 9999), "data": "INTEGRITY_CHECK"}

# --- SENARYOLAR ---
SCENARIOS = [
    # 1. KRİTİK TESTLER (Veri Bütünlüğü)
    ("GET", "/api/json", None),
    ("GET", "/api/xml", None),
    ("GET", "/api/large", None),
    ("POST", "/api/echo", ECHO_PAYLOAD),
    
    # 2. GENEL TESTLER (Stress)
    ("GET", "/", None),
    ("GET", "/robots.txt", None),
    ("GET", "/.env", None),
    ("POST", "/login", {"u":"admin", "p":"123"}),
    ("GET", "/vuln/xss?q=script", None),
    ("POST", "/vuln/xxe", "<root>test</root>"),
    ("GET", "/api/admin/users", None),
    ("GET", "/config.json", None),
    ("GET", "/vuln/sqli?id=1'", None),
    ("GET", "/vuln/lfi?file=../../passwd", None),
]

class IntegrationTest:
    def __init__(self):
        self.procs = []
        self.id_matrix = {}
        
    def log(self, msg, level="INFO"):
        print(f"[{time.strftime('%H:%M:%S')}] {level}: {msg}")

    def graphql(self, query, variables=None):
        try:
            res = requests.post(GRAPHQL_URL, json={"query": query, "variables": variables or {}}, timeout=10)
            return res.json()
        except Exception as e:
            self.log(f"GraphQL Error: {e}", "ERROR")
            return None

    def check_port(self, port):
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        result = sock.connect_ex(('127.0.0.1', port))
        sock.close()
        return result == 0

    # --- PHASE 0: STARTUP ---
    def start_services(self):
        self.log("🚀 Starting Services...")
        
        if os.path.exists(f"workspace/{TEST_PROJECT_NAME}.proxxy"):
            shutil.rmtree(f"workspace/{TEST_PROJECT_NAME}.proxxy")

        # 1. Orchestrator
        self.log("   Starting Orchestrator...")
        self.procs.append(subprocess.Popen(
            [ORCHESTRATOR_BINARY, "--project", TEST_PROJECT_NAME],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL 
        ))
        
        for _ in range(15):
            if self.check_port(9090) and self.check_port(50051): break
            time.sleep(1)
        else:
            self.log("❌ Orchestrator failed to bind ports", "ERROR"); return False

        # 2. Test Server
        self.log("   Starting Test Server...")
        self.procs.append(subprocess.Popen(
            [TEST_SERVER_BINARY],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        ))
        time.sleep(2)

        # 3. Proxy Agent
        self.log("   Starting Proxy Agent...")
        self.procs.append(subprocess.Popen(
            [PROXY_AGENT_BINARY, "--orchestrator-url", "http://127.0.0.1:50051", "--listen-port", "8080"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        ))

        for _ in range(10):
            if self.check_port(8080): break
            time.sleep(1)
        else:
            self.log("❌ Proxy Agent failed to bind port 8080", "ERROR"); return False

        self.log("   ✅ All Services Ready")
        
        # Disable Interception
        self.graphql("mutation { toggleInterception(enabled: false) { enabled } }")
        return True

    # --- PHASE 1: TRAFFIC ---
    def generate_traffic(self):
        self.log(f"⚡ PHASE 1: Generating Traffic ({len(SCENARIOS)} requests)...")
        proxies = {'http': PROXY_URL, 'https': PROXY_URL}
        
        success = 0
        for method, endpoint, data in SCENARIOS:
            url = f"{TEST_SERVER_URL}{endpoint}"
            try:
                if method == "POST": requests.post(url, json=data, proxies=proxies, timeout=5)
                else: requests.get(url, proxies=proxies, timeout=5)
                success += 1
                time.sleep(0.05) # Throttle slightly
            except:
                self.log(f"   ⚠️ Failed to request {endpoint}", "WARN")
        
        self.log(f"   Sent {success} requests. Waiting 5s for DB Sync...")
        time.sleep(5)

    # --- PHASE 2: VERIFICATION ---
    def verify_capture(self):
        self.log("🔍 PHASE 2: Verifying Data Integrity...")
        
        # 1. Get List (Lightweight)
        res = self.graphql("query { requests(agentId: null) { requestId url } }")
        if not res or "data" not in res:
            self.log("❌ Failed to fetch list from DB", "ERROR"); return False

        requests_list = res["data"]["requests"]
        self.log(f"   📥 Fetched {len(requests_list)} records from DB")

        # Build Matrix
        self.id_matrix = {}
        for r in requests_list:
            for _, ep, _ in SCENARIOS:
                if ep in r["url"]: self.id_matrix[ep] = r["requestId"]

        # 2. Deep Check (Heavyweight)
        all_good = True
        criticals = ["/api/json", "/api/echo", "/api/large"]
        
        for ep in criticals:
            if ep not in self.id_matrix:
                self.log(f"   ❌ Missing capture for {ep}", "ERROR"); all_good = False; continue

            # Get Full Detail
            detail = self.graphql("""
                query GetDetail($id: String!) {
                    request(id: $id) { responseBody status }
                }
            """, {"id": self.id_matrix[ep]})
            
            data = detail.get("data", {}).get("request", {})
            body = data.get("responseBody", "")
            
            # Content Checks
            if ep == "/api/json":
                if "server_time" in body: self.log(f"   ✅ JSON Integrity OK")
                else: self.log("   ❌ JSON Corrupted", "ERROR"); all_good = False
            
            elif ep == "/api/echo":
                if str(ECHO_PAYLOAD["test_id"]) in body: self.log(f"   ✅ Echo Payload OK")
                else: self.log("   ❌ Echo Mismatch", "ERROR"); all_good = False

            elif ep == "/api/large":
                if len(body) > 10000: self.log(f"   ✅ Large Body OK ({len(body)} bytes)")
                else: self.log("   ❌ Large Body Truncated", "ERROR"); all_good = False

        return all_good

    def cleanup(self):
        self.log("🧹 Cleaning up...")
        for p in self.procs:
            p.terminate()
            try: p.wait(timeout=1)
            except: p.kill()

    def run(self):
        try:
            if self.start_services():
                self.generate_traffic()
                if self.verify_capture():
                    print("\n🏆 TEST RESULT: PASSED")
                else:
                    print("\n💥 TEST RESULT: FAILED (Integrity Check)")
            
            # --- YENİ EKLENEN KISIM: ID LISTESI ---
            print("\n📋 CAPTURED REQUEST IDs (Copy & Paste):")
            print("-" * 50)
            # URL uzunsa hizalamak için ljust kullanıyoruz
            for endpoint, req_id in self.id_matrix.items():
                # Sadece path kısmını göster (http://localhost... kısmını at)
                short_ep = endpoint.split("8000")[-1] if "8000" in endpoint else endpoint
                print(f"{short_ep.ljust(25)} : {req_id}")
            print("-" * 50)
            # -------------------------------------

            print("\n🛑 SERVERS RUNNING. Press ENTER to stop...")
            input()
        finally:
            self.cleanup()

if __name__ == "__main__":
    IntegrationTest().run()