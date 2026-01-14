# Justfile for Proxxy project

# Run the GUI (Tauri)
gui:
    cd proxxy-gui && npm run tauri dev

# Run the Orchestrator
orch:
    cargo run -p orchestrator -- --grpc-port 50051 --http-port 9090

# Run the Proxy Agent
agent:
    cargo run -p proxy-agent -- --name "Agent-Beta"

# Test proxy traffic
test:
    ./test_proxy_traffic.sh

# Run multiple (Helper - note: these usually need separate terminals)
# Usage: just orch & just agent & just gui
dev: gui orch agent

# Run the Mock Vulnerable Server
mock:
	cd test-server && cargo run --bin test_server

# Run comprehensive vulnerability scan through proxy
scan:
	./test_all_vulns.sh

# Run Attack Engine & Security Test Suite
test-security:
	./test_attack_engine.sh

# Run Unit Tests & Property-Based Tests
test-units:
	./test_units.sh

# Run all tests
test-all: test-units test-security

# Run the Mock Vulnerable Server (alias for test-server)
test-server:
	cd test-server && cargo run --bin test_server