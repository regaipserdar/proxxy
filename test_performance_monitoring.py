#!/usr/bin/env python3
"""
Test script to verify performance monitoring for response body capture.
This test exercises the body capture functionality and checks that metrics
are properly recorded and exposed via the admin API.
"""

import asyncio
import json
import time
import requests
import subprocess
import signal
import os
import sys
from typing import Optional, Dict, Any

class PerformanceMonitoringTest:
    def __init__(self):
        self.orchestrator_process: Optional[subprocess.Popen] = None
        self.proxy_agent_process: Optional[subprocess.Popen] = None
        self.test_server_process: Optional[subprocess.Popen] = None
        
        # Configuration
        self.orchestrator_port = 8080
        self.grpc_port = 50052  # Use different gRPC port
        self.proxy_port = 8081
        self.admin_port = 8082
        self.test_server_port = 3000
        self.project_name = "performance_test"
        
    def start_orchestrator(self) -> bool:
        """Start the orchestrator with project support"""
        print("🚀 Starting orchestrator...")
        try:
            self.orchestrator_process = subprocess.Popen([
                "./target/release/orchestrator",
                "--project", self.project_name,
                "--http-port", str(self.orchestrator_port),
                "--grpc-port", str(self.grpc_port)
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            
            # Wait for orchestrator to start
            time.sleep(3)
            
            # Check if process is still running
            if self.orchestrator_process.poll() is not None:
                stdout, stderr = self.orchestrator_process.communicate()
                print(f"❌ Orchestrator failed to start:")
                print(f"STDOUT: {stdout}")
                print(f"STDERR: {stderr}")
                return False
                
            print("✅ Orchestrator started successfully")
            return True
            
        except Exception as e:
            print(f"❌ Failed to start orchestrator: {e}")
            return False
    
    def start_proxy_agent(self) -> bool:
        """Start the proxy agent with body capture enabled"""
        print("🚀 Starting proxy agent with body capture...")
        try:
            self.proxy_agent_process = subprocess.Popen([
                "./target/release/proxy-agent",
                "--orchestrator-url", f"http://localhost:{self.grpc_port}",
                "--listen-port", str(self.proxy_port),
                "--admin-port", str(self.admin_port),
                "--enable-body-capture", "true",
                "--max-body-size", "1048576",  # 1MB
                "--response-timeout", "30",    # 30 seconds
                "--stream-timeout", "5"        # 5 seconds
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            
            # Wait for proxy agent to start
            time.sleep(3)
            
            # Check if process is still running
            if self.proxy_agent_process.poll() is not None:
                stdout, stderr = self.proxy_agent_process.communicate()
                print(f"❌ Proxy agent failed to start:")
                print(f"STDOUT: {stdout}")
                print(f"STDERR: {stderr}")
                return False
                
            print("✅ Proxy agent started successfully")
            return True
            
        except Exception as e:
            print(f"❌ Failed to start proxy agent: {e}")
            return False
    
    def start_test_server(self) -> bool:
        """Start the test server"""
        print("🚀 Starting test server...")
        try:
            self.test_server_process = subprocess.Popen([
                "./target/release/test_server",
                "--port", str(self.test_server_port)
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            
            # Wait for test server to start
            time.sleep(2)
            
            # Check if process is still running
            if self.test_server_process.poll() is not None:
                stdout, stderr = self.test_server_process.communicate()
                print(f"❌ Test server failed to start:")
                print(f"STDOUT: {stdout}")
                print(f"STDERR: {stderr}")
                return False
                
            print("✅ Test server started successfully")
            return True
            
        except Exception as e:
            print(f"❌ Failed to start test server: {e}")
            return False
    
    def get_admin_metrics(self) -> Optional[Dict[str, Any]]:
        """Get metrics from the admin API"""
        try:
            response = requests.get(f"http://localhost:{self.admin_port}/metrics", timeout=5)
            if response.status_code == 200:
                return response.json()
            else:
                print(f"❌ Failed to get metrics: HTTP {response.status_code}")
                return None
        except Exception as e:
            print(f"❌ Failed to get admin metrics: {e}")
            return None
    
    def generate_test_traffic(self) -> bool:
        """Generate test traffic through the proxy to exercise body capture"""
        print("📡 Generating test traffic...")
        
        proxy_url = f"http://localhost:{self.proxy_port}"
        test_endpoints = [
            f"http://localhost:{self.test_server_port}/api/json",      # JSON response
            f"http://localhost:{self.test_server_port}/api/xml",       # XML response  
            f"http://localhost:{self.test_server_port}/api/large",     # Large response
            f"http://localhost:{self.test_server_port}/",              # HTML response
        ]
        
        success_count = 0
        total_requests = len(test_endpoints) * 3  # 3 requests per endpoint
        
        for endpoint in test_endpoints:
            for i in range(3):  # Make 3 requests to each endpoint
                try:
                    print(f"  📤 Request {success_count + 1}/{total_requests}: {endpoint}")
                    
                    # Make request through proxy
                    response = requests.get(
                        endpoint,
                        proxies={"http": proxy_url, "https": proxy_url},
                        timeout=10
                    )
                    
                    if response.status_code == 200:
                        success_count += 1
                        print(f"    ✅ Success: {len(response.content)} bytes")
                    else:
                        print(f"    ❌ Failed: HTTP {response.status_code}")
                        
                    # Small delay between requests
                    time.sleep(0.5)
                    
                except Exception as e:
                    print(f"    ❌ Request failed: {e}")
        
        print(f"📊 Traffic generation complete: {success_count}/{total_requests} successful")
        return success_count > 0
    
    def verify_performance_metrics(self) -> bool:
        """Verify that performance metrics are properly recorded"""
        print("📊 Verifying performance metrics...")
        
        # Get initial metrics
        initial_metrics = self.get_admin_metrics()
        if not initial_metrics:
            print("❌ Failed to get initial metrics")
            return False
        
        print("📈 Initial metrics:")
        self.print_body_capture_metrics(initial_metrics)
        
        # Generate traffic
        if not self.generate_test_traffic():
            print("❌ Failed to generate test traffic")
            return False
        
        # Wait a moment for metrics to be updated
        time.sleep(2)
        
        # Get final metrics
        final_metrics = self.get_admin_metrics()
        if not final_metrics:
            print("❌ Failed to get final metrics")
            return False
        
        print("📈 Final metrics:")
        self.print_body_capture_metrics(final_metrics)
        
        # Verify metrics have changed
        return self.validate_metrics_changes(initial_metrics, final_metrics)
    
    def print_body_capture_metrics(self, metrics: Dict[str, Any]):
        """Print body capture metrics in a readable format"""
        if "body_capture" in metrics:
            bc = metrics["body_capture"]
            print(f"  📊 Body Capture Metrics:")
            print(f"    • Attempts: {bc.get('attempts', 0)}")
            print(f"    • Successes: {bc.get('successes', 0)}")
            print(f"    • Failures: {bc.get('failures', 0)}")
            print(f"    • Timeouts: {bc.get('timeouts', 0)}")
            print(f"    • Memory Errors: {bc.get('memory_errors', 0)}")
            print(f"    • Success Rate: {bc.get('success_rate', 0):.1f}%")
            print(f"    • Average Latency: {bc.get('average_latency_ms', 0):.2f}ms")
            print(f"    • Total Bytes Captured: {bc.get('total_bytes_captured', 0)}")
        else:
            print("  ❌ No body_capture metrics found")
    
    def validate_metrics_changes(self, initial: Dict[str, Any], final: Dict[str, Any]) -> bool:
        """Validate that metrics have changed as expected"""
        print("🔍 Validating metrics changes...")
        
        if "body_capture" not in initial or "body_capture" not in final:
            print("❌ Body capture metrics not found in initial or final metrics")
            return False
        
        initial_bc = initial["body_capture"]
        final_bc = final["body_capture"]
        
        # Check that attempts increased
        attempts_diff = final_bc.get("attempts", 0) - initial_bc.get("attempts", 0)
        if attempts_diff <= 0:
            print(f"❌ Body capture attempts did not increase: {attempts_diff}")
            return False
        
        # Check that successes increased
        successes_diff = final_bc.get("successes", 0) - initial_bc.get("successes", 0)
        if successes_diff <= 0:
            print(f"❌ Body capture successes did not increase: {successes_diff}")
            return False
        
        # Check that bytes were captured
        bytes_diff = final_bc.get("total_bytes_captured", 0) - initial_bc.get("total_bytes_captured", 0)
        if bytes_diff <= 0:
            print(f"❌ No bytes were captured: {bytes_diff}")
            return False
        
        # Check success rate is reasonable
        success_rate = final_bc.get("success_rate", 0)
        if success_rate < 50:  # At least 50% success rate
            print(f"❌ Success rate too low: {success_rate}%")
            return False
        
        # Check average latency is recorded
        avg_latency = final_bc.get("average_latency_ms", 0)
        if avg_latency <= 0:
            print(f"❌ Average latency not recorded: {avg_latency}ms")
            return False
        
        print("✅ All metrics validations passed:")
        print(f"  • Attempts increased by: {attempts_diff}")
        print(f"  • Successes increased by: {successes_diff}")
        print(f"  • Bytes captured: {bytes_diff}")
        print(f"  • Success rate: {success_rate:.1f}%")
        print(f"  • Average latency: {avg_latency:.2f}ms")
        
        return True
    
    def cleanup(self):
        """Clean up all processes"""
        print("🧹 Cleaning up processes...")
        
        processes = [
            ("Test Server", self.test_server_process),
            ("Proxy Agent", self.proxy_agent_process),
            ("Orchestrator", self.orchestrator_process),
        ]
        
        for name, process in processes:
            if process and process.poll() is None:
                print(f"  🛑 Stopping {name}...")
                try:
                    process.terminate()
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    print(f"    ⚠️  Force killing {name}...")
                    process.kill()
                    process.wait()
                except Exception as e:
                    print(f"    ❌ Error stopping {name}: {e}")
    
    def run_test(self) -> bool:
        """Run the complete performance monitoring test"""
        print("🧪 Starting Performance Monitoring Test")
        print("=" * 50)
        
        try:
            # Start all services
            if not self.start_orchestrator():
                return False
            
            if not self.start_proxy_agent():
                return False
            
            if not self.start_test_server():
                return False
            
            # Wait for all services to be ready
            print("⏳ Waiting for services to be ready...")
            time.sleep(5)
            
            # Run the performance monitoring test
            success = self.verify_performance_metrics()
            
            if success:
                print("\n🎉 Performance Monitoring Test PASSED!")
                print("✅ All body capture metrics are working correctly")
            else:
                print("\n❌ Performance Monitoring Test FAILED!")
                
            return success
            
        except KeyboardInterrupt:
            print("\n⚠️  Test interrupted by user")
            return False
        except Exception as e:
            print(f"\n❌ Test failed with exception: {e}")
            return False
        finally:
            self.cleanup()

def main():
    """Main function"""
    if not os.path.exists("./target/release/orchestrator"):
        print("❌ Orchestrator binary not found. Please run 'cargo build --release' first.")
        return 1
    
    if not os.path.exists("./target/release/proxy-agent"):
        print("❌ Proxy agent binary not found. Please run 'cargo build --release' first.")
        return 1
    
    if not os.path.exists("./target/release/test_server"):
        print("❌ Test server binary not found. Please run 'cargo build --release' first.")
        return 1
    
    test = PerformanceMonitoringTest()
    success = test.run_test()
    
    return 0 if success else 1

if __name__ == "__main__":
    sys.exit(main())