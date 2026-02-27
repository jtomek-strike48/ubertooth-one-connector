//! USB diagnostics test - trying different read approaches

use ubertooth_usb::{UbertoothDevice, constants::*};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("trace")
        .init();

    println!("========================================");
    println!("USB Diagnostics Test");
    println!("========================================\n");

    // Connect
    println!("[1] Connecting...");
    let mut device = UbertoothDevice::new()?;
    device.connect(0)?;
    println!("✅ Connected\n");

    // Configure for BLE
    println!("[2] Configuring for BLE...");
    device.set_modulation(MOD_BT_LOW_ENERGY)?;
    device.set_channel(37)?;
    println!("✅ Configured\n");

    // Start promiscuous mode
    println!("[3] Starting BLE promiscuous mode...");
    device.control_transfer(CMD_BTLE_PROMISC, 0, 0, &[], 1000)?;
    println!("✅ Started\n");

    // Wait for data to accumulate
    println!("[4] Waiting 2 seconds for data...");
    std::thread::sleep(Duration::from_secs(2));
    println!("✅ Ready\n");

    // Test 1: Try reading with various timeouts
    println!("[5] Testing different timeout values:\n");

    let timeouts = vec![1, 5, 10, 50, 100, 500, 1000, 5000];

    for timeout_ms in timeouts {
        let mut buffer = vec![0u8; 64];
        print!("   Timeout {}ms: ", timeout_ms);

        match device.bulk_read(&mut buffer, timeout_ms) {
            Ok(len) => {
                println!("✅ SUCCESS! Read {} bytes", len);
                println!("      Data: {:02X?}\n", &buffer[..len.min(16)]);

                // Keep reading if we got data
                if len > 0 {
                    println!("   Continuing to read with {}ms timeout:", timeout_ms);
                    for i in 1..=5 {
                        match device.bulk_read(&mut buffer, timeout_ms) {
                            Ok(len2) => {
                                println!("      Read #{}: {} bytes: {:02X?}",
                                    i, len2, &buffer[..len2.min(16)]);
                            }
                            Err(e) => {
                                println!("      Read #{}: Error: {}", i, e);
                            }
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                println!("❌ {}", e);
            }
        }
    }

    // Stop
    println!("\n[6] Stopping...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;

    // Disconnect
    device.disconnect()?;
    println!("✅ Done\n");

    Ok(())
}
