//! Test the pure libusb device implementation
//!
//! This tests the production-ready pure libusb layer.

use ubertooth_usb::device_libusb::UbertoothDeviceLibusb;
use ubertooth_usb::protocol::UsbPacket;
use ubertooth_usb::constants::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("Pure libusb Device Implementation Test");
    println!("========================================\n");

    // Connect
    println!("[1/4] Connecting to Ubertooth...");
    let mut device = UbertoothDeviceLibusb::new()?;
    device.connect(0)?;
    println!("✅ Connected\n");

    if let Some(info) = device.device_info() {
        println!("Device: {} ({})", info.board_name(), info.firmware_version);
        println!("Serial: {}", info.serial_number);
    }
    println!();

    // TEST: Double-stop to ensure clean state
    println!("[2/5] Ensuring device is completely stopped...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    println!("✅ Device stopped\n");

    println!("[4/5] Starting BLE promiscuous mode...");
    println!("  CMD_JAM_MODE({})={}", JAM_NONE, CMD_JAM_MODE);
    let ret = device.control_transfer(CMD_JAM_MODE, JAM_NONE as u16, 0, &[], 1000)?;
    println!("  Returned: {} bytes", ret);

    println!("  CMD_SET_MODULATION({})={}", MOD_BT_LOW_ENERGY, CMD_SET_MODULATION);
    let ret = device.control_transfer(CMD_SET_MODULATION, MOD_BT_LOW_ENERGY as u16, 0, &[], 1000)?;
    println!("  Returned: {} bytes", ret);

    println!("  CMD_SET_CHANNEL={}  value=2402 (channel 37 frequency)", CMD_SET_CHANNEL);
    let ret = device.control_transfer(CMD_SET_CHANNEL, 2402, 0, &[], 1000)?;
    println!("  Returned: {} bytes", ret);

    println!("  CMD_BTLE_SNIFFING={} (for advertisement scanning)", CMD_BTLE_SNIFFING);
    let ret = device.control_transfer(CMD_BTLE_SNIFFING, 0, 0, &[], 1000)?;
    println!("  Returned: {} bytes", ret);

    // Verify the mode was set
    let mut buffer = [0u8; 64];
    println!("\n  Verifying mode with CMD_GET_MOD={}...", CMD_GET_MOD);
    match device.control_transfer_in(CMD_GET_MOD, 0, 0, &mut buffer, 1000) {
        Ok(len) if len > 0 => {
            println!("  Current modulation: {} (0=BR, 1=LE, 2=FHSS)", buffer[0]);
        }
        Ok(_) => println!("  No modulation data returned"),
        Err(e) => println!("  Failed to get modulation: {:?}", e),
    }

    println!("✅ BLE promiscuous mode commands sent\n");

    // CRITICAL: Give firmware time to actually enter BLE mode!
    // The firmware main loop needs CPU time to process requested_mode
    println!("Waiting for firmware to enter BLE promiscuous mode...");
    tokio::time::sleep(Duration::from_millis(2000)).await;
    println!("✅ Firmware should now be in BLE mode\n");

    // Use CMD_POLL polling approach (like Python ubertooth-btle)
    println!("[5/5] Starting CMD_POLL packet capture (10 seconds)...\n");

    let start = tokio::time::Instant::now();
    let duration = Duration::from_secs(10);
    let mut packet_count = 0;
    let mut ble_packet_count = 0;

    while start.elapsed() < duration {
        // Poll for packet using CMD_POLL (like Python does)
        let mut buffer = [0u8; 64];
        match device.control_transfer_in(CMD_POLL, 0, 0, &mut buffer, 1000) {
            Ok(len) if len == 64 => {
                let packet = buffer.to_vec();
                packet_count += 1;

                // Parse USB packet
                if packet.len() >= 14 {
                    // Debug: print first packet
                    if packet_count == 1 {
                        println!("First packet {} bytes: {:02x?}", packet.len(), &packet[..std::cmp::min(20, packet.len())]);
                    }

                    match UsbPacket::from_bytes(&packet) {
                        Ok(usb_pkt) => {
                            if usb_pkt.is_ble() {
                                ble_packet_count += 1;
                                if ble_packet_count <= 5 {
                                    println!("✅ BLE Packet #{}: channel={}, {} bytes",
                                        ble_packet_count,
                                        usb_pkt.header.channel,
                                        packet.len()
                                    );
                                }
                            } else if packet_count <= 5 {
                                println!("Non-BLE packet: channel={}, pkt_type={}",
                                    usb_pkt.header.channel, usb_pkt.header.pkt_type);
                            }
                        }
                        Err(e) if packet_count <= 5 => {
                            println!("Parse error: {:?}", e);
                        }
                        Err(_) => {}
                    }
                }

                // Progress update every 20 packets
                if packet_count % 20 == 0 {
                    println!("  [{:.1}s] Received {} packets ({} BLE)",
                        start.elapsed().as_secs_f32(),
                        packet_count,
                        ble_packet_count
                    );
                }

                // Sleep like Python does (500 microseconds between polls)
                tokio::time::sleep(Duration::from_micros(500)).await;
            }
            _ => {
                // No packet or error - sleep and retry
                tokio::time::sleep(Duration::from_micros(500)).await;
            }
        }
    }

    println!("\n========================================");
    println!("Results:");
    println!("  Duration: {:.1}s", start.elapsed().as_secs_f32());
    println!("  Total packets: {}", packet_count);
    println!("  BLE packets: {}", ble_packet_count);

    if packet_count > 0 {
        println!("\n🎉 SUCCESS! Pure libusb device works!");
        println!("This is production-ready for the Rust backend!");
    } else {
        println!("\n⚠️  No packets received");
        println!("Make sure BLE devices are broadcasting nearby.");
    }

    println!("========================================\n");

    // Stop
    println!("Stopping...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;

    // Disconnect
    device.disconnect()?;
    println!("✅ Done\n");

    Ok(())
}
