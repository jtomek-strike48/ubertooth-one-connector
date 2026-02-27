#!/usr/bin/env python3
"""Test script for btle_scan with Rust USB backend."""

import json
import sys
import os

# Add the project root to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

from ubertooth_connector import UbertoothConnector

def main():
    print("=" * 60)
    print("Testing btle_scan with Rust USB backend")
    print("=" * 60)
    print()

    # Create connector instance
    connector = UbertoothConnector()

    # Test 1: Device connection
    print("[1/3] Connecting to Ubertooth device...")
    result = connector.execute("device_connect", {})

    if not result.get("success"):
        print(f"❌ Connection failed: {result.get('error', 'Unknown error')}")
        return 1

    print(f"✅ Connected to {result.get('board_name', 'Unknown')}")
    print(f"   Firmware: {result.get('firmware_version', 'Unknown')}")
    print(f"   Serial: {result.get('serial', 'Unknown')}")
    print()

    # Test 2: BLE scan (5 seconds)
    print("[2/3] Starting BLE scan (5 seconds on channel 37)...")
    print("      Looking for BLE advertisements...")

    scan_result = connector.execute("btle_scan", {
        "duration_sec": 5,
        "channel": 37,
        "save_pcap": True
    })

    if not scan_result.get("success"):
        print(f"❌ Scan failed: {scan_result.get('error', 'Unknown error')}")
        return 1

    print(f"✅ Scan completed!")
    print(f"   Duration: {scan_result.get('scan_duration_sec', 0)} seconds")
    print(f"   Channel: {scan_result.get('channel', 0)}")
    print(f"   Total packets: {scan_result.get('total_packets', 0)}")
    print(f"   Devices found: {len(scan_result.get('devices_found', []))}")
    print(f"   Capture ID: {scan_result.get('capture_id', 'N/A')}")

    if scan_result.get("pcap_path"):
        print(f"   PCAP path: {scan_result['pcap_path']}")

    print()

    # Show discovered devices
    devices = scan_result.get("devices_found", [])
    if devices:
        print("   Discovered BLE devices:")
        for i, device in enumerate(devices, 1):
            mac = device.get("mac_address", "Unknown")
            name = device.get("device_name", "Unknown")
            rssi = device.get("rssi_avg", 0)
            packets = device.get("packet_count", 0)
            print(f"     {i}. {mac}")
            print(f"        Name: {name}")
            print(f"        RSSI: {rssi} dBm")
            print(f"        Packets: {packets}")
    else:
        print("   ⚠️  No BLE devices detected (normal if no devices nearby)")

    print()

    # Test 3: Device disconnect
    print("[3/3] Disconnecting...")
    disconnect_result = connector.execute("device_disconnect", {})

    if disconnect_result.get("success"):
        print("✅ Disconnected successfully")
    else:
        print("⚠️  Disconnect warning:", disconnect_result.get("message", ""))

    print()
    print("=" * 60)
    print("✅ All tests completed successfully!")
    print("=" * 60)

    return 0

if __name__ == "__main__":
    sys.exit(main())
