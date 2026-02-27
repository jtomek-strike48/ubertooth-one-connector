//! Test nusb async bulk transfers

use std::time::Duration;

const USB_VENDOR_ID: u16 = 0x1d50;
const USB_PRODUCT_ID: u16 = 0x6002;
const ENDPOINT_DATA_IN: u8 = 0x82;

// USB commands
const CMD_SET_MODULATION: u8 = 22;
const CMD_SET_CHANNEL: u8 = 12;
const CMD_BTLE_PROMISC: u8 = 37;
const CMD_STOP: u8 = 21;

const MOD_BT_LOW_ENERGY: u8 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("nusb Async Transfer Test");
    println!("========================================\n");

    // Find device
    println!("[1] Finding Ubertooth device...");
    let device_info = nusb::list_devices().await?
        .find(|d| d.vendor_id() == USB_VENDOR_ID && d.product_id() == USB_PRODUCT_ID)
        .ok_or("Ubertooth not found")?;

    println!("✅ Found device\n");

    // Open device
    println!("[2] Opening device...");
    let device = device_info.open().await?;

    // Claim interface
    let interface = device.claim_interface(0).await?;
    println!("✅ Device opened and interface claimed\n");

    // Configure for BLE (control transfers)
    println!("[3] Configuring for BLE...");

    // Set modulation
    interface.control_out_blocking(
        nusb::transfer::ControlType::Vendor,
        CMD_SET_MODULATION,
        MOD_BT_LOW_ENERGY as u16,
        0,
        &[],
        Duration::from_secs(1),
    )?;

    // Set channel
    interface.control_out_blocking(
        nusb::transfer::ControlType::Vendor,
        CMD_SET_CHANNEL,
        37,
        0,
        &[],
        Duration::from_secs(1),
    )?;

    // Start BLE promiscuous mode
    interface.control_out_blocking(
        nusb::transfer::ControlType::Vendor,
        CMD_BTLE_PROMISC,
        0,
        0,
        &[],
        Duration::from_secs(1),
    )?;

    println!("✅ BLE promiscuous mode started\n");

    // Wait for data to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Read packets asynchronously
    println!("[4] Reading packets (10 seconds)...\n");

    let mut packet_count = 0;
    let start = tokio::time::Instant::now();
    let duration = Duration::from_secs(10);

    while start.elapsed() < duration {
        // Submit async bulk read
        let queue = interface.bulk_in_queue(ENDPOINT_DATA_IN);

        // Allocate buffer
        let mut buffer = vec![0u8; 64];

        // Submit read with 100ms timeout
        match tokio::time::timeout(
            Duration::from_millis(100),
            queue.submit(buffer.clone())
        ).await {
            Ok(Ok(completion)) => {
                let data = completion.data;
                if data.len() > 0 {
                    packet_count += 1;
                    println!("✅ Packet #{}: {} bytes - {:02X?}",
                        packet_count,
                        data.len(),
                        &data[..data.len().min(16)]
                    );
                }
            }
            Ok(Err(e)) => {
                eprintln!("❌ USB error: {}", e);
            }
            Err(_) => {
                // Timeout - no data available, continue
            }
        }

        // Small delay
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    println!("\n[5] Stopping...");

    // Stop
    interface.control_out_blocking(
        nusb::transfer::ControlType::Vendor,
        CMD_STOP,
        0,
        0,
        &[],
        Duration::from_secs(1),
    )?;

    println!("✅ Done\n");
    println!("========================================");
    println!("Total packets received: {}", packet_count);
    println!("========================================\n");

    Ok(())
}
