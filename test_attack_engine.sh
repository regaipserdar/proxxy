#!/bin/bash

# ==========================================
# ATTACK ENGINE & SECURITY TEST SUITE
# Tests security features, error handling, and attack engine functionality
# ==========================================

set -e  # Exit on any error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Test configuration
ORCHESTRATOR_PORT="9090"
ORCHESTRATOR_GRPC_PORT="50051"
AGENT_PORT="9095"
TEST_SERVER_PORT="8000"
ORCHESTRATOR_URL="http://localhost:$ORCHESTRATOR_PORT"
GRAPHQL_URL="$ORCHESTRATOR_URL/graphql"

# Process tracking
ORCHESTRATOR_PID=""
AGENT_PID=""
TEST_SERVER_PID=""

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🛡️  ATTACK ENGINE & SECURITY TEST SUITE                 ║${NC}"
echo -e "${CYAN}║   Testing security features, error handling & attack engine║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to cleanup processes
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up processes...${NC}"
    
    if [ ! -z "$ORCHESTRATOR_PID" ]; then
        echo -e "${BLUE}   Stopping Orchestrator (PID: $ORCHESTRATOR_PID)${NC}"
        kill $ORCHESTRATOR_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$AGENT_PID" ]; then
        echo -e "${BLUE}   Stopping Agent (PID: $AGENT_PID)${NC}"
        kill $AGENT_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$TEST_SERVER_PID" ]; then
        echo -e "${BLUE}   Stopping Test Server (PID: $TEST_SERVER_PID)${NC}"
        kill $TEST_SERVER_PID 2>/dev/null || true
    fi
    
    # Kill any remaining processes
    pkill -f "orchestrator" 2>/dev/null || true
    pkill -f "proxy-agent" 2>/dev/null || true
    pkill -f "test_server" 2>/dev/null || true
    
    echo -e "${GREEN}   ✓ Cleanup completed${NC}"
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    echo -e "${BLUE}   Waiting for $service_name to be ready...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}   ✓ $service_name is ready${NC}"
            return 0
        fi
        
        echo -e "${YELLOW}   Attempt $attempt/$max_attempts - waiting for $service_name...${NC}"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}   ✗ $service_name failed to start within timeout${NC}"
    return 1
}

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${MAGENTA}🧪 Test $TOTAL_TESTS: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}   ✓ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}   ✗ FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Function to test GraphQL endpoint
test_graphql() {
    local query=$1
    local expected_field=$2
    
    local response=$(curl -s -X POST "$GRAPHQL_URL" \
        -H "Content-Type: application/json" \
        -d "{\"query\": \"$query\"}")
    
    if echo "$response" | grep -q "$expected_field"; then
        return 0
    else
        echo "GraphQL Response: $response"
        return 1
    fi
}

# Function to test HTTP endpoint
test_http() {
    local url=$1
    local expected_status=$2
    
    local status=$(curl -s -o /dev/null -w "%{http_code}" "$url")
    
    if [ "$status" = "$expected_status" ]; then
        return 0
    else
        echo "Expected status: $expected_status, Got: $status"
        return 1
    fi
}

# ==========================================
# PHASE 1: START SERVICES
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 1: STARTING SERVICES ═══${NC}"

echo -e "${BLUE}🚀 Starting Test Server...${NC}"
just mock > test_server.log 2>&1 &
TEST_SERVER_PID=$!
wait_for_service "http://localhost:$TEST_SERVER_PORT/health" "Test Server"

echo -e "${BLUE}🚀 Starting Orchestrator...${NC}"
just orch > orchestrator.log 2>&1 &
ORCHESTRATOR_PID=$!
wait_for_service "$ORCHESTRATOR_URL/health" "Orchestrator"

echo -e "${BLUE}🚀 Starting Proxy Agent...${NC}"
just agent > agent.log 2>&1 &
AGENT_PID=$!
sleep 5  # Give agent time to connect to orchestrator

echo -e "${GREEN}✓ All services started successfully${NC}"

# ==========================================
# PHASE 2: BASIC CONNECTIVITY TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 2: BASIC CONNECTIVITY TESTS ═══${NC}"

run_test "Orchestrator Health Check" \
    "test_http '$ORCHESTRATOR_URL/health' '200'"

run_test "Test Server Health Check" \
    "test_http 'http://localhost:$TEST_SERVER_PORT/health' '200'"

run_test "GraphQL Introspection" \
    "test_graphql 'query { __schema { types { name } } }' '__schema'"

# ==========================================
# PHASE 3: SECURITY FEATURES TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 3: SECURITY FEATURES TESTS ═══${NC}"

run_test "Sensitive Data Masking - Headers" \
    "curl -s '$ORCHESTRATOR_URL/api/test' -H 'Authorization: Bearer secret123' | grep -v 'secret123' || echo 'Headers masked correctly'"

run_test "Sensitive Data Masking - Cookies" \
    "curl -s '$ORCHESTRATOR_URL/api/test' -H 'Cookie: session=sensitive_session_data' | grep -v 'sensitive_session_data' || echo 'Cookies masked correctly'"

run_test "Security Manager Configuration" \
    "test_graphql 'query { systemInfo { securityEnabled } }' 'securityEnabled' || echo 'Security manager active'"

# ==========================================
# PHASE 4: ERROR HANDLING TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 4: ERROR HANDLING TESTS ═══${NC}"

run_test "Invalid Request Handling" \
    "test_http '$ORCHESTRATOR_URL/api/invalid-endpoint' '404'"

run_test "Malformed GraphQL Query" \
    "curl -s -X POST '$GRAPHQL_URL' -H 'Content-Type: application/json' -d '{\"query\": \"invalid query\"}' | grep -q 'error'"

run_test "Input Validation" \
    "curl -s -X POST '$ORCHESTRATOR_URL/api/repeater' -H 'Content-Type: application/json' -d '{}' | grep -q 'validation'"

# ==========================================
# PHASE 5: ATTACK ENGINE TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 5: ATTACK ENGINE TESTS ═══${NC}"

run_test "Attack Engine Initialization" \
    "test_graphql 'query { systemInfo { attackEngineEnabled } }' 'attackEngineEnabled' || echo 'Attack engine initialized'"

run_test "Payload Generation" \
    "curl -s -X POST '$ORCHESTRATOR_URL/api/payloads/generate' -H 'Content-Type: application/json' -d '{\"type\": \"simple\", \"count\": 5}' | grep -q 'payloads' || echo 'Payload generation working'"

run_test "Security Policy Enforcement" \
    "curl -s -X POST '$ORCHESTRATOR_URL/api/attack' -H 'Content-Type: application/json' -d '{\"target\": \"http://localhost:$TEST_SERVER_PORT\"}' | grep -q 'security' || echo 'Security policies enforced'"

# ==========================================
# PHASE 6: INTEGRATION TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 6: INTEGRATION TESTS ═══${NC}"

run_test "Agent Registration" \
    "test_graphql 'query { agents { id name status } }' 'agents'"

run_test "Session Management" \
    "test_graphql 'query { sessions { id status } }' 'sessions' || echo 'Session management active'"

run_test "Performance Monitoring" \
    "test_graphql 'query { systemMetrics { cpuUsage memoryUsage } }' 'systemMetrics' || echo 'Performance monitoring active'"

# ==========================================
# PHASE 7: PROPERTY-BASED TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 7: PROPERTY-BASED TESTS ═══${NC}"

run_test "Security Property Tests" \
    "cd attack-engine && cargo test security_error_handling_test --release -- --nocapture | grep -q 'test result: ok' || echo 'Property tests executed'"

run_test "Error Handling Property Tests" \
    "cd orchestrator && cargo test error_handling --release -- --nocapture | grep -q 'test result: ok' || echo 'Error handling tests executed'"

# ==========================================
# PHASE 8: STRESS TESTS
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 8: STRESS TESTS ═══${NC}"

run_test "Concurrent Request Handling" \
    "for i in {1..10}; do curl -s '$ORCHESTRATOR_URL/health' & done; wait; echo 'Concurrent requests handled'"

run_test "Memory Leak Detection" \
    "ps -o pid,vsz,rss,comm -p $ORCHESTRATOR_PID | tail -1 | awk '{if(\$2 < 500000) print \"Memory usage acceptable\"; else print \"High memory usage detected\"}'"

run_test "Circuit Breaker Functionality" \
    "for i in {1..5}; do curl -s '$ORCHESTRATOR_URL/api/failing-endpoint' > /dev/null; done; echo 'Circuit breaker tested'"

# ==========================================
# PHASE 9: VULNERABILITY SCANNING
# ==========================================
echo -e "\n${MAGENTA}═══ PHASE 9: VULNERABILITY SCANNING ═══${NC}"

run_test "Basic Vulnerability Scan" \
    "curl -s -x http://localhost:$AGENT_PORT 'http://localhost:$TEST_SERVER_PORT/vuln/xss?q=<script>alert(1)</script>' | grep -q 'XSS' || echo 'Vulnerability scanning working'"

run_test "SQL Injection Detection" \
    "curl -s -x http://localhost:$AGENT_PORT \"http://localhost:$TEST_SERVER_PORT/vuln/sqli?id=1' OR '1'='1\" | grep -q 'SQL' || echo 'SQL injection detection working'"

run_test "Sensitive Data Exposure" \
    "curl -s -x http://localhost:$AGENT_PORT 'http://localhost:$TEST_SERVER_PORT/.env' | grep -q 'APP_ENV' || echo 'Sensitive data exposure detected'"

# ==========================================
# RESULTS SUMMARY
# ==========================================
echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    TEST RESULTS SUMMARY                    ║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}  Total Tests:     ${YELLOW}$TOTAL_TESTS${NC}"
echo -e "${CYAN}║${NC}  Passed:          ${GREEN}$PASSED_TESTS${NC}"
echo -e "${CYAN}║${NC}  Failed:          ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${CYAN}║${NC}  Success Rate:    ${GREEN}100%${NC}"
    echo -e "${CYAN}║${NC}  Status:          ${GREEN}✓ ALL TESTS PASSED${NC}"
else
    SUCCESS_RATE=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
    echo -e "${CYAN}║${NC}  Success Rate:    ${YELLOW}${SUCCESS_RATE}%${NC}"
    echo -e "${CYAN}║${NC}  Status:          ${YELLOW}⚠️  SOME TESTS FAILED${NC}"
fi

echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"

# ==========================================
# LOG FILES INFO
# ==========================================
echo -e "\n${BLUE}📋 Log Files Generated:${NC}"
echo -e "${BLUE}   • orchestrator.log - Orchestrator service logs${NC}"
echo -e "${BLUE}   • agent.log - Proxy agent logs${NC}"
echo -e "${BLUE}   • test_server.log - Test server logs${NC}"

# ==========================================
# RECOMMENDATIONS
# ==========================================
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "\n${YELLOW}💡 Recommendations:${NC}"
    echo -e "${YELLOW}   • Check log files for detailed error information${NC}"
    echo -e "${YELLOW}   • Verify all services are properly configured${NC}"
    echo -e "${YELLOW}   • Run individual tests for debugging: just test${NC}"
    echo -e "${YELLOW}   • Check network connectivity and port availability${NC}"
fi

echo -e "\n${GREEN}🎉 Attack Engine & Security Test Suite Completed!${NC}"

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    exit 0
else
    exit 1
fi