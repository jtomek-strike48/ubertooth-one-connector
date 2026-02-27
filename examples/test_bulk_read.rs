//! Simple synchronous test of bulk reads

use ubertooth_usb::{UbertoothDevice, constants::*};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    println!("========================================");
    println!("Simple Bulk Read Test");
    println!("========================================");
    println!();

    // Connect
    println!("[1/5] Connecting to Ubertooth...");
    let mut device = UbertoothDevice::new()?;
    device.connect(0)?;
    println!("✅ Connected");
    println!();

    // Set modulation to BLE
    println!("[2/5] Setting BLE modulation...");
    device.set_modulation(MOD_BT_LOW_ENERGY)?;
    println!("✅ Modulation set");
    println!();

    // Set channel
    println!("[3/5] Setting channel to 37...");
    device.set_channel(37)?;
    println!("✅ Channel set");
    println!();

    // Start promiscuous mode
    println!("[4/5] Starting BLE promiscuous mode...");
    device.control_transfer(CMD_BTLE_PROMISC, 0, 0, &[], 1000)?;
    println!("✅ Promiscuous mode started");
    println!();

    // Wait a moment
    thread::sleep(Duration::from_millis(500));

    // Try to read bulk data
    println!("[5/5] Attempting bulk reads (10 attempts, 2 sec timeout each)...");
    println!();

    for attempt in 1..=10 {
        let mut buffer = vec![0u8; 64];
        print!("Attempt {}/10: ", attempt);

        match device.bulk_read(&mut buffer, 2000) {
            Ok(len) => {
                println!("✅ Received {} bytes!", len);
                println!("    Data: {:02X?}", &buffer[..len.min(32)]);

                if len >= 14 {
                    println!("    Packet type: {}", buffer[0]);
                    println!("    Channel: {}", buffer[2]);
                }
                println!();
            }
            Err(e) => {
                println!("❌ {}", e);
            }
        }
    }

    // Stop
    println!();
    println!("Sending STOP command...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;

    // Disconnect
    println!("Disconnecting...");
    device.disconnect()?;
    println!("✅ Done");

    Ok(())
}
