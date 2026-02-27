#!/bin/bash

echo "=========================================="
echo "USB Async Transfer Debug Script"
echo "Testing with sudo to rule out permissions"
echo "=========================================="
echo ""

# 1. Python baseline
echo "[1/5] Testing Python baseline (3 seconds)..."
timeout 3 ubertooth-btle -f -c 37 2>&1 | head -10
echo "✅ Python baseline complete"
echo ""

# 2. Simple nusb test with sudo
echo "[2/5] Testing simple nusb streaming WITH SUDO (15 seconds)..."
sudo timeout 15 ./target/release/examples/test_nusb_simple
NUSB_SIMPLE_RESULT=$?
echo "Exit code: $NUSB_SIMPLE_RESULT"
echo ""

# 3. Wrapper nusb test with sudo
echo "[3/5] Testing nusb streaming wrapper WITH SUDO (15 seconds)..."
sudo timeout 15 ./target/release/examples/test_nusb_stream
NUSB_WRAPPER_RESULT=$?
echo "Exit code: $NUSB_WRAPPER_RESULT"
echo ""

# 4. Original rusb test with sudo
echo "[4/5] Testing original rusb WITH SUDO (10 seconds)..."
sudo timeout 10 ./target/release/examples/test_bulk_read
RUSB_RESULT=$?
echo "Exit code: $RUSB_RESULT"
echo ""

# 5. Check dmesg for errors
echo "[5/5] Checking kernel logs for USB errors..."
sudo dmesg | tail -100 | grep -i "usb\|ubertooth\|1d50:6002" | tail -20 || echo "No USB messages found"
echo ""

echo "=========================================="
echo "RESULTS SUMMARY"
echo "=========================================="
echo "nusb simple:  Exit $NUSB_SIMPLE_RESULT (0=success, 124=timeout)"
echo "nusb wrapper: Exit $NUSB_WRAPPER_RESULT (0=success, 124=timeout)"
echo "rusb:         Exit $RUSB_RESULT (0=success, 124=timeout)"
echo ""

# Additional diagnostics
echo "=========================================="
echo "USB DIAGNOSTICS"
echo "=========================================="
echo ""

echo "USB Device Info:"
lsusb -d 1d50:6002 | head -5
echo ""

echo "USB Interface Details:"
lsusb -d 1d50:6002 -v 2>&1 | grep -A 3 "bInterfaceClass" | head -10
echo ""

echo "Udev Rules:"
ls -la /etc/udev/rules.d/ | grep -i ubertooth || echo "No ubertooth udev rules found"
echo ""

echo "Processes using USB:"
sudo lsof 2>/dev/null | grep -i "usb.*1d50" | head -10 || echo "No processes found"
echo ""

echo "=========================================="
echo "TEST COMPLETE"
echo "=========================================="
echo ""
echo "If sudo helped (packets > 0), it's a permissions issue."
echo "If sudo didn't help, it's a deeper nusb/firmware issue."
echo ""
