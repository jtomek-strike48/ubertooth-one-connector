#!/bin/bash
# Run Ubertooth One connector with Prospector Studio
#
# Usage: ./run-connector.sh

set -e

echo "=== Ubertooth One Connector for Prospector Studio ==="
echo

# Check if Ubertooth is connected
if ! lsusb | grep -q "1d50:6002"; then
    echo "❌ Ubertooth One not detected!"
    echo "   Please plug in your Ubertooth One device"
    exit 1
fi

echo "✅ Ubertooth One detected"

# Check if ubertooth-tools are installed
if ! command -v ubertooth-util &> /dev/null; then
    echo "❌ ubertooth-tools not installed!"
    echo "   Run: sudo apt-get install ubertooth"
    exit 1
fi

echo "✅ ubertooth-tools installed"

# Get firmware version
FIRMWARE=$(ubertooth-util -v 2>&1 | grep "Firmware version" || echo "Unknown")
echo "✅ Firmware: $FIRMWARE"
echo

# Configuration
export STRIKE48_ACCEPT_INVALID_CERTS=true
export MATRIX_TLS_INSECURE=true  # SDK also checks this
export RUST_LOG=info
export INSTANCE_ID=ubertooth-connector-1
export MATRIX_HOST=connectors-jt-demo-01.strike48.test
export MATRIX_API_URL=https://jt-demo-01.strike48.test
export MATRIX_TENANT_ID=non-prod
export TENANT_ID=non-prod
export SK_PORT=3033

# Optional: Set display name for UI
export CONNECTOR_DISPLAY_NAME="Ubertooth One"

# Backend selection (python = CLI wrapper, rust = Phase 3 not implemented yet)
export UBERTOOTH_BACKEND=python

echo "Configuration:"
echo "  Host: $MATRIX_HOST"
echo "  Tenant: $TENANT_ID"
echo "  Instance: $INSTANCE_ID"
echo "  Backend: $UBERTOOTH_BACKEND"
echo "  TLS: Insecure (accepting invalid certs)"
echo

# Set STRIKE48_URL from MATRIX_HOST (SDK will parse this)
# SDK supports: wss://host, grpc://host, grpcs://host
export STRIKE48_URL="grpcs://${MATRIX_HOST}"

echo "Starting connector..."
echo "Press Ctrl+C to stop"
echo
echo "Logs will appear below:"
echo "---"

# Run the connector
cargo run --bin ubertooth-agent --release 2>&1 | tee ubertooth-connector.log
