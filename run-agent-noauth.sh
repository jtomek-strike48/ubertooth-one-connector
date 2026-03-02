#!/bin/bash
# Run Ubertooth Agent WITHOUT authentication

export STRIKE48_URL="wss://jt-demo-01.strike48.engineering"
export TENANT_ID="non-prod"
export MATRIX_API_URL="https://jt-demo-01.strike48.engineering"
export MATRIX_TENANT_ID="non-prod"
export UBERTOOTH_BACKEND="rust"
# NO AUTH_TOKEN - running unauthenticated

echo "=== Ubertooth Agent (Unauthenticated Mode) ==="
echo "Server: $STRIKE48_URL"
echo "Tenant: $TENANT_ID"
echo "Instance: Ubertooth One"
echo ""
echo "Registered 36 tools:"
echo "  - Device Management (3)"
echo "  - Reconnaissance (7)"
echo "  - Analysis (5)"
echo "  - Capture Management (5)"
echo "  - Configuration (8)"
echo "  - Attack Operations (5)"
echo "  - Advanced (2)"
echo ""
echo "Starting agent..."
echo ""

exec ./target/release/ubertooth-agent --backend rust
