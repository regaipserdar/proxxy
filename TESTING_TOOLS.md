# Proxxy Testing Tools & Server Documentation

This document provides a guide to the testing tools available in the Proxxy project and explains the new stress testing endpoints added to the test server.

## 1. Test Server Endpoints

The `test_server` has been enhanced with dynamic endpoints specifically designed for stress testing and integrity checks.

### New Endpoints

| Endpoint | Method | Description | Content-Type |
|----------|--------|-------------|--------------|
| `/api/json` | GET | Returns dynamic JSON with timestamp, UUID, and random value. | `application/json` |
| `/api/xml` | GET | Returns dynamic XML with timestamp and UUID. | `application/xml` |
| `/api/large` | GET | Returns a large response body (>1MB). | `text/plain` |
| `/api/echo` | POST | Reflects the request body and adds `X-Server-Reflected-Random` header if `X-Custom-Random` is present. | `application/json` |

### Existing Vulnerability Endpoints

The server also includes numerous endpoints for testing vulnerability scanners (`/vuln/*`), including LFI, XSS, SQLi, and more.

## 2. Python Testing Tools

The project includes several Python scripts for testing different aspects of the Proxxy system.

### `send_test_requests.py`
**Purpose:** Sends specific HTTP requests (GET, POST, PUT, DELETE, etc.) through the proxy to `testphp.vulnweb.com`.
**Usage:**
```bash
python3 send_test_requests.py
```
**Key Features:**
- Tests various HTTP methods.
- Verifies proxy handling of different request types.

### `test_integration.py`
**Purpose:** Tests the complete workflow of the Proxxy Orchestrator, from startup to traffic capture.
**Usage:**
```bash
python3 test_integration.py
```
**Key Features:**
- Starts Orchestrator and Proxy Agent.
- Generates traffic.
- Verifies project loading and GraphQL API connectivity.
- Checks if traffic is correctly captured and stored.

### `test_performance_monitoring.py`
**Purpose:** Verifies that performance metrics (e.g., body capture success rate, latency) are properly recorded.
**Usage:**
```bash
python3 test_performance_monitoring.py
```
**Key Features:**
- Generates traffic to test body capture performance.
- Validates metrics exposed via the Admin API.

### `test_response_body_capture.py`
**Purpose:** Specifically tests the "Response Body Capture" feature.
**Usage:**
```bash
python3 test_response_body_capture.py
```
**Key Features:**
- Verifies that response bodies (JSON, HTML, etc.) are captured.
- Checks data integrity of captured bodies.
- Validates GraphQL queries for retrieving response bodies.

## 3. Documentation Resources

- **[RESPONSE_BODY_CAPTURE.md](RESPONSE_BODY_CAPTURE.md):** Detailed documentation on the Response Body Capture feature, including configuration and architecture.
- **[graphql_test_queries.md](graphql_test_queries.md):** A collection of GraphQL queries for testing the Orchestrator API manually.

## 4. How to Run Stress Tests

To use your custom stress test script with the new dynamic endpoints:

1. **Build and Start the Test Server:**
   ```bash
   cd test-server
   cargo run --release
   ```
   The server will listen on `http://127.0.0.1:8000` (or configured port).

2. **Run the Proxy Agent:**
   Ensure your proxy agent is running and configured to forward traffic to the test server.

3. **Run your Python Stress Script:**
   Execute your stress test script. It can now target:
   - `http://127.0.0.1:8000/api/json`
   - `http://127.0.0.1:8000/api/xml`
   - `http://127.0.0.1:8000/api/large`
   - `http://127.0.0.1:8000/api/echo`

These endpoints will provide the dynamic responses needed to verify the integrity and performance of the proxy.
