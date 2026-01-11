# Justfile for Proxxy project

# Run the GUI (Tauri)
gui:
    cd proxxy-gui && npm run tauri dev

# Run the Orchestrator
orch:
    cargo run -p orchestrator -- --grpc-port 50051 --http-port 9090

# Run the Proxy Agent
agent:
    cargo run -p proxy-agent -- --name "Agent-Alpha"

# Test proxy traffic
test:
    ./test_proxy_traffic.sh

# Run multiple (Helper - note: these usually need separate terminals)
# Usage: just orch & just agent & just gui
dev: gui orch agent
