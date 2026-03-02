#!/bin/bash
# Run Ubertooth Agent with Strike48

export STRIKE48_URL="wss://jt-demo-01.strike48.engineering"
export TENANT_ID="non-prod"
export AUTH_TOKEN="ott_QoU4XIHRjkuDRxMpSBLHCyaRmaBfWG1_30WV0vV3MOs"
export MATRIX_API_URL="https://jt-demo-01.strike48.engineering"
export MATRIX_TENANT_ID="non-prod"
export UBERTOOTH_BACKEND="rust"

echo "=== Starting Ubertooth Agent ==="
echo "Server: $STRIKE48_URL"
echo "Tenant: $TENANT_ID"
echo "Backend: $UBERTOOTH_BACKEND"
echo ""

./target/release/ubertooth-agent --backend rust
