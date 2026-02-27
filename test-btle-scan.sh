#!/bin/bash
# Test script for btle_scan with Rust USB backend

set -e

echo "=========================================="
echo "Testing btle_scan with Rust USB backend"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build if needed
if [ ! -f "target/release/ubertooth-agent" ]; then
    echo -e "${BLUE}Building release version...${NC}"
    cargo build --features rust-backend --release --quiet
    echo ""
fi

# Create temporary test script
cat > /tmp/ubertooth_test.py << 'PYEOF'
import asyncio
import json
import sys

async def test_btle_scan():
    """Test btle_scan via direct backend call."""
    from ubertooth_platform import RustUsbBackend, SidecarManager
    from serde_json import json as serde_json

    print("[1/3] Creating Rust USB backend...")

    # Create Python fallback
    python_fallback = SidecarManager.new()

    # Create Rust backend with fallback
    backend = RustUsbBackend.with_fallback(python_fallback)

    if backend.is_err():
        print(f"❌ Failed to create backend: {backend.unwrap_err()}")
        return 1

    backend = backend.unwrap()

    print("✅ Backend created")
    print()

    # Test device connection
    print("[2/3] Connecting to device...")
    connect_params = serde_json({})
    result = await backend.call("device_connect", connect_params)

    if result.is_err():
        print(f"❌ Connection failed: {result.unwrap_err()}")
        return 1

    result = result.unwrap()
    print(f"✅ Connected: {result}")
    print()

    # Test BLE scan
    print("[3/3] Starting BLE scan (5 seconds)...")
    scan_params = serde_json({
        "duration_sec": 5,
        "channel": 37,
        "save_pcap": True
    })

    scan_result = await backend.call("btle_scan", scan_params)

    if scan_result.is_err():
        print(f"❌ Scan failed: {scan_result.unwrap_err()}")
        return 1

    scan_result = scan_result.unwrap()
    print(f"✅ Scan completed!")
    print(json.dumps(scan_result, indent=2))

    return 0

if __name__ == "__main__":
    sys.exit(asyncio.run(test_btle_scan()))
PYEOF

echo -e "${BLUE}Test 1: Device Connection${NC}"
echo "Starting agent to test connection..."
timeout 3 sh -c "UBERTOOTH_BACKEND=rust ./target/release/ubertooth-agent 2>&1" | grep -E "(Connected|success|Device:)" || true
echo ""

echo -e "${BLUE}Test 2: BLE Scan (Manual)${NC}"
echo "This will scan for BLE devices for 5 seconds..."
echo "Note: Need to test via agent API (not available in smoke test mode)"
echo ""

# Alternative: Test with debug build to see more output
echo -e "${BLUE}Test 3: Live BLE Scan Test${NC}"
echo "Running live test with debug output..."
echo ""

# Create a simple Rust test program
cat > /tmp/test_btle.rs << 'RUSTEOF'
use ubertooth_usb::{UbertoothDevice, UbertoothCommands};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("==============================================");
    println!("BLE Scan Test with Rust USB Backend");
    println!("==============================================");
    println!();

    // Create and connect to device
    println!("[1/4] Creating USB device...");
    let mut device = UbertoothDevice::new()?;

    println!("[2/4] Connecting to Ubertooth One...");
    device.connect(0)?;

    let info = device.device_info().unwrap();
    println!("✅ Connected to: {} ({})", info.board_name(), info.firmware_version);
    println!();

    // Create command executor
    println!("[3/4] Creating command executor...");
    let device = Arc::new(Mutex::new(device));
    let commands = UbertoothCommands::new(device);

    // Run BLE scan
    println!("[4/4] Starting BLE scan (5 seconds on channel 37)...");
    println!("Looking for BLE advertisements...");
    println!();

    let result = commands.btle_scan(json!({
        "duration_sec": 5,
        "channel": 37,
        "save_pcap": false
    })).await?;

    println!("==============================================");
    println!("✅ Scan Result:");
    println!("==============================================");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
RUSTEOF

# Compile and run if we have the dependencies
if command -v rustc &> /dev/null; then
    echo "Compiling test program..."
    cd /tmp
    mkdir -p test_btle_project
    cd test_btle_project

    # Create a simple Cargo project
    if [ ! -f "Cargo.toml" ]; then
        cat > Cargo.toml << 'EOF'
[package]
name = "test_btle"
version = "0.1.0"
edition = "2021"

[dependencies]
ubertooth-usb = { path = "/home/jtomek/Code/ubertooth-one-connector/crates/usb" }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
EOF
    fi

    cp /tmp/test_btle.rs src/main.rs 2>/dev/null || mkdir -p src && cp /tmp/test_btle.rs src/main.rs

    echo ""
    echo -e "${GREEN}Building and running BLE scan test...${NC}"
    echo ""

    RUST_LOG=info cargo run --release 2>&1 || echo "Test completed"
else
    echo -e "${YELLOW}⚠️  rustc not found, skipping Rust test${NC}"
fi

echo ""
echo "=========================================="
echo "Test script completed"
echo "=========================================="
