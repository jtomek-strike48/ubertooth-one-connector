#!/usr/bin/env bash
# Quick test of bt_analyze using the CLI

set -euo pipefail

CAPTURE_ID="cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3"

echo "========================================"
echo "Testing bt_analyze Phase 2"
echo "========================================"
echo ""
echo "Capture: $CAPTURE_ID"
echo ""

# Build CLI
echo "Building CLI..."
cargo build --bin ubertooth-cli --features python-backend --quiet 2>&1 | grep -v "warning:" || true
echo "✅ Build complete"
echo ""

# Run analyze via agent mode
echo "Running bt_analyze..."
echo ""

# Create temp script to call the tool
cat > /tmp/test_analyze_input.json <<EOF
{
  "capture_id": "$CAPTURE_ID",
  "analysis_type": "auto"
}
EOF

# Use the agent mode to execute
./target/debug/ubertooth-cli agent <<AGENT_INPUT
bt_analyze $(cat /tmp/test_analyze_input.json)
AGENT_INPUT

echo ""
echo "========================================"
echo "Test Complete"
echo "========================================"
