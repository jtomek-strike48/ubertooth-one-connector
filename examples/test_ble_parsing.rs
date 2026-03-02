//! Test BLE packet parsing and advertising data extraction

use ubertooth_usb::device_libusb::UbertoothDeviceLibusb;
use ubertooth_usb::protocol::{UsbPacket, BlePacket};
use ubertooth_usb::constants::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("BLE Packet Parsing Test");
    println!("========================================\n");

    let mut device = UbertoothDeviceLibusb::new()?;
    device.connect(0)?;
    println!("✅ Connected\n");

    // Setup BLE scanning
    println!("Setting up BLE scanning on channel 37...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    device.control_transfer(59, 0, 0, &[], 1000)?;
    device.control_transfer(CMD_SET_MODULATION, MOD_BT_LOW_ENERGY as u16, 0, &[], 1000)?;
    device.control_transfer(CMD_SET_CHANNEL, 2402, 0, &[], 1000)?;
    device.control_transfer(CMD_BTLE_SNIFFING, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("✅ Scanning started\n");

    println!("Capturing and parsing BLE advertisements (10 seconds)...\n");

    let start = std::time::Instant::now();
    let mut total_packets = 0;
    let mut parsed_packets = 0;
    let mut devices_seen = std::collections::HashSet::new();

    while start.elapsed() < Duration::from_secs(10) {
        let mut buffer = [0u8; 64];

        match device.control_transfer_in(CMD_POLL, 0, 0, &mut buffer, 100) {
            Ok(64) => {
                total_packets += 1;

                // Parse USB packet
                if let Ok(usb_pkt) = UsbPacket::from_bytes(&buffer.to_vec()) {
                    if !usb_pkt.is_ble() {
                        continue;
                    }

                    // Parse BLE packet
                    match BlePacket::from_usb_packet(&usb_pkt) {
                        Ok(ble_pkt) => {
                            parsed_packets += 1;

                            // Check if it's an advertising packet
                            if ble_pkt.is_advertising() {
                                // Parse advertising data
                                match ble_pkt.parse_advertising_data() {
                                    Ok(ad_data) => {
                                        let addr_str = ad_data.address_string();

                                        // Track unique devices
                                        if devices_seen.insert(addr_str.clone()) {
                                            println!("📱 New Device: {}", addr_str);
                                            println!("   Type: {} ({})",
                                                ble_pkt.pdu_type_name(),
                                                match ad_data.address_type {
                                                    ubertooth_usb::protocol::AddressType::Public => "Public",
                                                    ubertooth_usb::protocol::AddressType::Random => "Random",
                                                }
                                            );

                                            if let Some(name) = &ad_data.name {
                                                println!("   Name: {}", name);
                                            }

                                            if let Some(flags) = ad_data.flags {
                                                println!("   Flags: 0x{:02X}", flags);
                                                if flags & 0x01 != 0 {
                                                    println!("     - LE Limited Discoverable");
                                                }
                                                if flags & 0x02 != 0 {
                                                    println!("     - LE General Discoverable");
                                                }
                                                if flags & 0x04 != 0 {
                                                    println!("     - BR/EDR Not Supported");
                                                }
                                            }

                                            if let Some(tx_power) = ad_data.tx_power {
                                                println!("   TX Power: {} dBm", tx_power);
                                            }

                                            if !ad_data.service_uuids_16.is_empty() {
                                                print!("   Services (16-bit): ");
                                                for uuid in &ad_data.service_uuids_16 {
                                                    print!("0x{:04X} ", uuid);
                                                }
                                                println!();
                                            }

                                            if !ad_data.service_uuids_128.is_empty() {
                                                println!("   Services (128-bit): {} UUIDs", ad_data.service_uuids_128.len());
                                            }

                                            if let Some((company_id, data)) = &ad_data.manufacturer_data {
                                                println!("   Manufacturer: 0x{:04X} ({} bytes)",
                                                    company_id, data.len());
                                            }

                                            println!("   RSSI: {} dBm", ble_pkt.rssi);
                                            println!("   Access Address: 0x{:08X}", ble_pkt.access_address);
                                            println!();
                                        }
                                    }
                                    Err(e) => {
                                        if parsed_packets <= 5 {
                                            println!("⚠️  Failed to parse advertising data: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if parsed_packets <= 5 {
                                println!("⚠️  Failed to parse BLE packet: {}", e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    println!("========================================");
    println!("Summary");
    println!("========================================");
    println!("  Total USB packets: {}", total_packets);
    println!("  Parsed BLE packets: {}", parsed_packets);
    println!("  Unique devices: {}", devices_seen.len());
    println!();

    if devices_seen.is_empty() {
        println!("⚠️  No BLE devices detected");
        println!("   Make sure BLE devices are broadcasting nearby");
    } else {
        println!("✅ Successfully parsed {} unique BLE devices", devices_seen.len());
    }

    println!("========================================\n");

    // Cleanup
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    device.disconnect()?;

    Ok(())
}
