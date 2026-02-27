//! Test nusb streaming reader with continuous packet capture

use ubertooth_usb::constants::*;
use ubertooth_usb::device_nusb::UbertoothDevice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("nusb Streaming Reader Test");
    println!("========================================\n");

    // Create and connect
    println!("[1/4] Connecting to Ubertooth...");
    let mut device = UbertoothDevice::new()?;
    device.connect(0).await?;
    println!("✅ Connected\n");

    // Show device info
    if let Some(info) = device.device_info().await {
        println!("Device: {} ({})", info.board_name(), info.firmware_version);
        println!();
    }

    // Configure for BLE
    println!("[2/4] Configuring for BLE channel 37...");
    device.set_modulation(MOD_BT_LOW_ENERGY).await?;
    device.set_channel(37).await?;
    println!("✅ Configured\n");

    // Start promiscuous mode
    println!("[3/4] Starting BLE promiscuous mode...");
    device.control_transfer(CMD_BTLE_PROMISC, 0, 0, &[], 1000).await?;
    println!("✅ Promiscuous mode started\n");

    // Wait for firmware to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Create streaming reader
    println!("[4/4] Starting packet stream (10 seconds)...\n");
    let mut reader = device.create_stream_reader().await?;

    let start = tokio::time::Instant::now();
    let duration = tokio::time::Duration::from_secs(10);
    let mut packet_count = 0;

    while start.elapsed() < duration {
        // Try to read with timeout
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            reader.read_packet()
        ).await {
            Ok(Some(packet)) => {
                packet_count += 1;
                println!("✅ Packet #{}: {} bytes - {:02X?}",
                    packet_count,
                    packet.len(),
                    &packet[..packet.len().min(16)]
                );

                // Parse if valid
                if packet.len() >= 14 {
                    let pkt_type = packet[0];
                    let channel = packet[2];
                    println!("   Type: {}, Channel: {}", pkt_type, channel);
                }
            }
            Ok(None) => {
                println!("Stream ended");
                break;
            }
            Err(_) => {
                // Timeout - no data in 100ms, continue
            }
        }
    }

    println!("\n========================================");
    println!("Results:");
    println!("  Duration: {:.1}s", start.elapsed().as_secs_f32());
    println!("  Packets: {}", packet_count);

    if packet_count > 0 {
        println!("\n🎉 SUCCESS! nusb async streaming works!");
    } else {
        println!("\n⚠️  No packets received");
    }

    println!("========================================\n");

    // Stop
    println!("Stopping...");
    device.stop().await?;

    // Disconnect
    device.disconnect().await?;
    println!("✅ Done\n");

    Ok(())
}
