//! Test nusb device implementation with async bulk transfers

use ubertooth_usb::constants::*;
use ubertooth_usb::device_nusb::UbertoothDevice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("nusb Device Implementation Test");
    println!("========================================\n");

    // Create and connect
    println!("[1/5] Creating device...");
    let mut device = UbertoothDevice::new()?;
    println!("✅ Device created\n");

    println!("[2/5] Connecting to Ubertooth...");
    device.connect(0).await?;
    println!("✅ Connected\n");

    // Show device info
    if let Some(info) = device.device_info().await {
        println!("Device Information:");
        println!("  Board: {}", info.board_name());
        println!("  Firmware: {}", info.firmware_version);
        println!("  API: {}", info.api_version);
        println!("  Serial: {}", info.serial_number);
        println!();
    }

    // Configure for BLE
    println!("[3/5] Configuring for BLE...");
    device.set_modulation(MOD_BT_LOW_ENERGY).await?;
    device.set_channel(37).await?;
    println!("✅ Configured\n");

    // Start promiscuous mode
    println!("[4/5] Starting BLE promiscuous mode...");
    device.control_transfer(CMD_BTLE_PROMISC, 0, 0, &[], 1000).await?;
    println!("✅ Promiscuous mode started\n");

    // Wait for data
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Read packets
    println!("[5/5] Reading packets (10 seconds)...\n");

    let mut packet_count = 0;
    let start = tokio::time::Instant::now();
    let duration = tokio::time::Duration::from_secs(10);

    while start.elapsed() < duration {
        let mut buffer = vec![0u8; 64];

        match device.bulk_read(&mut buffer, 100).await {
            Ok(len) if len > 0 => {
                packet_count += 1;
                println!("✅ Packet #{}: {} bytes - {:02X?}",
                    packet_count,
                    len,
                    &buffer[..len.min(16)]
                );

                // Parse if it's a valid BLE packet
                if len >= 14 {
                    let pkt_type = buffer[0];
                    let channel = buffer[2];
                    println!("   Type: {}, Channel: {}", pkt_type, channel);
                }
            }
            Ok(_) => {
                // 0 bytes - no data
            }
            Err(e) => {
                if !e.to_string().contains("timeout") {
                    eprintln!("❌ Error: {}", e);
                }
            }
        }

        // Small delay between reads
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    println!("\n========================================");
    println!("Results:");
    println!("  Duration: 10 seconds");
    println!("  Packets: {}", packet_count);
    println!("========================================\n");

    // Stop
    println!("Stopping...");
    device.stop().await?;

    // Disconnect
    device.disconnect().await?;
    println!("✅ Test complete!\n");

    Ok(())
}
