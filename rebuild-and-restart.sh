#!/bin/bash
set -e

echo "=== Rebuilding Ubertooth Agent ==="

# Stop any running agents
echo "1. Stopping existing agents..."
pkill -9 -f ubertooth-agent || true
sleep 1

# Clean old build
echo "2. Cleaning old build artifacts..."
cargo clean -p ubertooth-agent
cargo clean -p ubertooth-core
rm -f ubertooth-agent.log

# Rebuild in release mode
echo "3. Building with rust-backend feature..."
cargo build --release --bin ubertooth-agent --features rust-backend

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

# Restart agent
echo "4. Starting agent..."
export STRIKE48_URL="wss://jt-demo-01.strike48.engineering"
export TENANT_ID="non-prod"
export MATRIX_API_URL="https://jt-demo-01.strike48.engineering"
export MATRIX_TENANT_ID="non-prod"
export UBERTOOTH_BACKEND="rust"

echo ""
echo "=== Agent Configuration ==="
echo "Server: $STRIKE48_URL"
echo "Tenant: $TENANT_ID"
echo "Backend: Rust USB"
echo ""

nohup ./target/release/ubertooth-agent --backend rust > ubertooth-agent.log 2>&1 &
AGENT_PID=$!

echo "Agent started with PID: $AGENT_PID"
sleep 3

# Check status
if ps -p $AGENT_PID > /dev/null; then
    echo "✅ Agent is running!"
    echo ""
    echo "=== Recent Log Output ==="
    tail -20 ubertooth-agent.log | grep -E "(INFO|Registered|Tool schemas)"
    echo ""
    echo "To follow logs: tail -f ubertooth-agent.log"
else
    echo "❌ Agent failed to start!"
    echo "Check logs: cat ubertooth-agent.log"
    exit 1
fi
