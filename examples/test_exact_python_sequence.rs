//! Test: EXACT command sequence from Python ubertooth-btle tool
//! Based on USB capture analysis

use ubertooth_usb::{UbertoothDevice, constants::*};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("EXACT Python Tool Command Sequence");
    println!("========================================\n");

    let mut device = UbertoothDevice::new()?;
    device.connect(0)?;
    println!("✅ Connected\n");

    // EXACT sequence from USB capture:

    // 1. CMD_JAM_MODE (59) with value 0
    println!("[1/4] CMD_JAM_MODE (59), value=0...");
    device.control_transfer(59, 0, 0, &[], 1000)?;

    // 2. CMD_SET_MODULATION (23) with value 1 (BLE)
    println!("[2/4] CMD_SET_MODULATION (23), value=1 (BLE)...");
    device.control_transfer(CMD_SET_MODULATION, 1, 0, &[], 1000)?;

    // 3. CMD_SET_CHANNEL (12) with value 2402 (FREQUENCY in MHz, not channel!)
    println!("[3/4] CMD_SET_CHANNEL (12), value=2402 MHz (channel 37)...");
    device.control_transfer(CMD_SET_CHANNEL, 2402, 0, &[], 1000)?;

    // 4. CMD_BTLE_SNIFFING (42) - NOT CMD_BTLE_PROMISC!
    println!("[4/4] CMD_BTLE_SNIFFING (42), value=0...");
    device.control_transfer(CMD_BTLE_SNIFFING, 0, 0, &[], 1000)?;

    println!("✅ Commands sent\n");

    // Wait a bit
    std::thread::sleep(Duration::from_millis(500));

    println!("Reading packets with CMD_POLL...\n");
    let start = std::time::Instant::now();
    let mut packet_count = 0;
    let mut ble_count = 0;

    while start.elapsed() < Duration::from_secs(5) && ble_count < 5 {
        let mut buffer = [0u8; 64];
        match device.control_transfer_read(CMD_POLL, 0, 0, &mut buffer, 100) {
            Ok(64) => {
                packet_count += 1;
                let pkt_type = buffer[0];

                if pkt_type == 1 {
                    // Type 1 = BLE packets (constants were wrong!)
                    ble_count += 1;
                    if ble_count <= 5 {
                        println!("🎉 BLE Packet #{}: {:02X?}...", ble_count, &buffer[..24]);
                    }
                } else if packet_count <= 3 {
                    println!("  Other packet type={}", pkt_type);
                }
            }
            Ok(_) => {}
            Err(e) if !e.to_string().contains("timeout") => {
                println!("  Error: {}", e);
            }
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_micros(500));
    }

    println!("\n========================================");
    println!("Results:");
    println!("  Total packets: {}", packet_count);
    println!("  BLE packets: {}", ble_count);

    if ble_count > 0 {
        println!("\n🎉🎉🎉 SUCCESS! BLE packets received!");
    } else {
        println!("\n❌ Still no BLE packets");
    }
    println!("========================================\n");

    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    device.disconnect()?;

    Ok(())
}
