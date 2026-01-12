#!/bin/bash

# ==========================================
# UNIT TESTS RUNNER
# Runs all unit tests including property-based tests
# ==========================================

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🧪 UNIT TESTS & PROPERTY-BASED TESTS                    ║${NC}"
echo -e "${CYAN}║   Running all unit tests for attack engine & orchestrator  ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"

# Test counters
TOTAL_SUITES=0
PASSED_SUITES=0
FAILED_SUITES=0

run_test_suite() {
    local suite_name=$1
    local test_command=$2
    
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    echo -e "\n${MAGENTA}🧪 Test Suite $TOTAL_SUITES: $suite_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}   ✓ PASSED${NC}"
        PASSED_SUITES=$((PASSED_SUITES + 1))
        return 0
    else
        echo -e "${RED}   ✗ FAILED${NC}"
        FAILED_SUITES=$((FAILED_SUITES + 1))
        return 1
    fi
}

# ==========================================
# ATTACK ENGINE TESTS
# ==========================================
echo -e "\n${BLUE}🛡️  Testing Attack Engine...${NC}"

run_test_suite "Attack Engine - Security Tests" \
    "cd attack-engine && cargo test security --release -- --nocapture"

run_test_suite "Attack Engine - Error Handling Tests" \
    "cd attack-engine && cargo test error --release -- --nocapture"

run_test_suite "Attack Engine - Property-Based Tests" \
    "cd attack-engine && cargo test security_error_handling_test --release -- --nocapture"

run_test_suite "Attack Engine - All Unit Tests" \
    "cd attack-engine && cargo test --release -- --nocapture"

# ==========================================
# ORCHESTRATOR TESTS
# ==========================================
echo -e "\n${BLUE}🎼 Testing Orchestrator...${NC}"

run_test_suite "Orchestrator - Error Handling Tests" \
    "cd orchestrator && cargo test error_handling --release -- --nocapture"

run_test_suite "Orchestrator - GraphQL Integration Tests" \
    "cd orchestrator && cargo test graphql_integration --release -- --nocapture"

run_test_suite "Orchestrator - Session Integration Tests" \
    "cd orchestrator && cargo test session_integration --release -- --nocapture"

run_test_suite "Orchestrator - Intruder Distribution Tests" \
    "cd orchestrator && cargo test intruder_distribution --release -- --nocapture"

run_test_suite "Orchestrator - Core Functionality Tests" \
    "cd orchestrator && cargo test core_functionality --release -- --nocapture"

run_test_suite "Orchestrator - All Unit Tests" \
    "cd orchestrator && cargo test --release -- --nocapture"

# ==========================================
# PROXY COMPONENTS TESTS
# ==========================================
echo -e "\n${BLUE}🔗 Testing Proxy Components...${NC}"

run_test_suite "Proxy Core Tests" \
    "cd proxy-core && cargo test --release -- --nocapture"

run_test_suite "Proxy Common Tests" \
    "cd proxy-common && cargo test --release -- --nocapture"

run_test_suite "Proxy Agent Tests" \
    "cd proxy-agent && cargo test --release -- --nocapture"

# ==========================================
# INTEGRATION TESTS
# ==========================================
echo -e "\n${BLUE}🔄 Running Integration Tests...${NC}"

run_test_suite "Workspace Integration Tests" \
    "cd workspace-tests && cargo test --release -- --nocapture"

# ==========================================
# RESULTS SUMMARY
# ==========================================
echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    UNIT TEST RESULTS                       ║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}  Total Test Suites: ${YELLOW}$TOTAL_SUITES${NC}"
echo -e "${CYAN}║${NC}  Passed:            ${GREEN}$PASSED_SUITES${NC}"
echo -e "${CYAN}║${NC}  Failed:            ${RED}$FAILED_SUITES${NC}"

if [ $FAILED_SUITES -eq 0 ]; then
    echo -e "${CYAN}║${NC}  Success Rate:      ${GREEN}100%${NC}"
    echo -e "${CYAN}║${NC}  Status:            ${GREEN}✓ ALL TESTS PASSED${NC}"
else
    SUCCESS_RATE=$(( (PASSED_SUITES * 100) / TOTAL_SUITES ))
    echo -e "${CYAN}║${NC}  Success Rate:      ${YELLOW}${SUCCESS_RATE}%${NC}"
    echo -e "${CYAN}║${NC}  Status:            ${YELLOW}⚠️  SOME TESTS FAILED${NC}"
fi

echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"

if [ $FAILED_SUITES -gt 0 ]; then
    echo -e "\n${YELLOW}💡 To debug failed tests:${NC}"
    echo -e "${YELLOW}   • Run specific test: cargo test <test_name> -- --nocapture${NC}"
    echo -e "${YELLOW}   • Check test output above for detailed error messages${NC}"
    echo -e "${YELLOW}   • Verify all dependencies are properly configured${NC}"
fi

echo -e "\n${GREEN}🎉 Unit Test Suite Completed!${NC}"

# Exit with appropriate code
if [ $FAILED_SUITES -eq 0 ]; then
    exit 0
else
    exit 1
fi