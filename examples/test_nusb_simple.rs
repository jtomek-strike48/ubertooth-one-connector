//! Simplest possible nusb streaming test

use std::time::Duration;

const USB_VENDOR_ID: u16 = 0x1d50;
const USB_PRODUCT_ID: u16 = 0x6002;
const ENDPOINT_DATA_IN: u8 = 0x82;
const CMD_SET_MODULATION: u8 = 22;
const CMD_SET_CHANNEL: u8 = 12;
const CMD_BTLE_PROMISC: u8 = 37;
const MOD_BT_LOW_ENERGY: u8 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Simple nusb Streaming Test");
    println!("========================================\n");

    // Find device
    println!("[1/4] Finding device...");
    let device_info = nusb::list_devices().await?
        .find(|d| d.vendor_id() == USB_VENDOR_ID && d.product_id() == USB_PRODUCT_ID)
        .ok_or("Ubertooth not found")?;
    println!("✅ Found\n");

    // Open device
    println!("[2/4] Opening device...");
    let device = device_info.open().await?;
    let interface = device.claim_interface(0).await?;
    println!("✅ Opened\n");

    // Configure
    println!("[3/4] Configuring...");
    interface.control_out(
        nusb::transfer::ControlOut {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Device,
            request: CMD_SET_MODULATION,
            value: MOD_BT_LOW_ENERGY as u16,
            index: 0,
            data: &[],
        },
        Duration::from_secs(1),
    ).await?;

    interface.control_out(
        nusb::transfer::ControlOut {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Device,
            request: CMD_SET_CHANNEL,
            value: 37,
            index: 0,
            data: &[],
        },
        Duration::from_secs(1),
    ).await?;

    interface.control_out(
        nusb::transfer::ControlOut {
            control_type: nusb::transfer::ControlType::Vendor,
            recipient: nusb::transfer::Recipient::Device,
            request: CMD_BTLE_PROMISC,
            value: 0,
            index: 0,
            data: &[],
        },
        Duration::from_secs(1),
    ).await?;

    println!("✅ Configured\n");

    // Stream
    println!("[4/4] Streaming (10 seconds)...\n");

    // Open endpoint
    let mut endpoint = interface.endpoint::<nusb::transfer::Bulk, nusb::transfer::In>(ENDPOINT_DATA_IN)?;

    // Submit initial transfers
    for _ in 0..8 {
        endpoint.submit(nusb::transfer::Buffer::new(64));
    }

    let start = tokio::time::Instant::now();
    let duration = Duration::from_secs(10);
    let mut packet_count = 0;

    while start.elapsed() < duration {
        // Wait for completion
        let completion = endpoint.next_complete().await;

        if let Err(e) = completion.status {
            println!("❌ Transfer error: {}", e);
        } else if completion.actual_len > 0 {
            packet_count += 1;
            println!("✅ Packet #{}: {} bytes - {:02X?}",
                packet_count,
                completion.actual_len,
                &completion.buffer[..completion.actual_len.min(16)]
            );
        }

        // Resubmit
        endpoint.submit(completion.buffer);
    }

    println!("\n========================================");
    println!("Packets: {}", packet_count);

    if packet_count > 0 {
        println!("🎉 SUCCESS!");
    }

    println!("========================================\n");

    Ok(())
}
