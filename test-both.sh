#!/bin/bash
# Quick comparison test - Python vs Rust

echo "=========================================="
echo "BLE Packet Capture Comparison Test"
echo "=========================================="
echo ""

echo "[1/2] Testing Python ubertooth-btle (5 seconds)..."
echo "----------------------------------------------"
timeout 5s sudo ubertooth-btle -f -c 37 2>&1 | grep -E "systime=|Advertising" | head -10
PYTHON_PACKETS=$(timeout 5s sudo ubertooth-btle -f -c 37 2>&1 | grep -c "systime=")
echo ""
echo "Python captured: ~$PYTHON_PACKETS packets"
echo ""

sleep 2

echo "[2/2] Testing Rust pure libusb (10 seconds)..."
echo "----------------------------------------------"
sudo ./target/release/examples/test_pure_libusb 2>&1 | grep -E "Packet #|Results:" | head -20
echo ""

echo "=========================================="
echo "If Python gets packets but Rust doesn't,"
echo "we have a code issue to fix."
echo ""
echo "If BOTH get 0 packets, no BLE devices are"
echo "broadcasting nearby."
echo "=========================================="
