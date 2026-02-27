//! Test BLE scanning with native Rust USB backend

use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_usb::{UbertoothCommands, UbertoothDevice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,ubertooth_usb=debug")
        .init();

    println!("==============================================");
    println!("BLE Scan Test - Rust USB Backend");
    println!("==============================================");
    println!();

    // Create and connect to device
    println!("[1/4] Creating USB device connection...");
    let mut device = UbertoothDevice::new()?;

    println!("[2/4] Connecting to Ubertooth One (index 0)...");
    device.connect(0)?;

    let info = device.device_info().expect("Device info should be available");
    println!("✅ Connected to: {} (firmware: {})",
             info.board_name(),
             info.firmware_version);
    println!("   Board ID: {}", info.board_id);
    println!("   Serial: {}", info.serial_number);
    println!();

    // Create command executor
    println!("[3/4] Creating command executor...");
    let device = Arc::new(Mutex::new(device));
    let commands = UbertoothCommands::new(device.clone());
    println!("✅ Command executor ready");
    println!();

    // Run BLE scan
    println!("[4/4] Starting BLE scan...");
    println!("   Duration: 5 seconds");
    println!("   Channel: 37 (BLE advertising)");
    println!("   Mode: Promiscuous");
    println!();
    println!("Scanning for BLE advertisements...");
    println!();

    let start = std::time::Instant::now();

    let result = commands.btle_scan(json!({
        "duration_sec": 5,
        "channel": 37,
        "promiscuous": true,
        "save_pcap": false
    })).await;

    let elapsed = start.elapsed();

    match result {
        Ok(scan_result) => {
            println!();
            println!("==============================================");
            println!("✅ Scan Completed in {:.2}s", elapsed.as_secs_f64());
            println!("==============================================");
            println!();

            // Extract key metrics
            let total_packets = scan_result["total_packets"].as_u64().unwrap_or(0);
            let devices = scan_result["devices_found"].as_array();
            let device_count = devices.map(|d| d.len()).unwrap_or(0);
            let channel = scan_result["channel"].as_u64().unwrap_or(0);

            println!("Summary:");
            println!("  • Channel: {}", channel);
            println!("  • Total packets: {}", total_packets);
            println!("  • Unique devices: {}", device_count);
            println!("  • Scan time: {:.2}s", elapsed.as_secs_f64());

            if total_packets > 0 {
                println!("  • Throughput: {:.0} packets/sec",
                         total_packets as f64 / elapsed.as_secs_f64());
            }

            println!();

            // Show discovered devices
            if let Some(devices) = devices {
                if !devices.is_empty() {
                    println!("Discovered BLE Devices:");
                    println!();
                    for (i, device) in devices.iter().enumerate() {
                        let mac = device["mac_address"].as_str().unwrap_or("Unknown");
                        let name = device["device_name"].as_str().unwrap_or("Unknown");
                        let rssi = device["rssi_avg"].as_i64().unwrap_or(0);
                        let packets = device["packet_count"].as_u64().unwrap_or(0);

                        println!("  {}. {} ({})", i + 1, mac, name);
                        println!("     RSSI: {} dBm | Packets: {}", rssi, packets);
                    }
                    println!();
                } else {
                    println!("ℹ️  No BLE devices detected");
                    println!("   (This is normal if no BLE devices are nearby)");
                    println!();
                }
            }

            // Show raw result for debugging
            if std::env::var("SHOW_RAW").is_ok() {
                println!("Raw scan result:");
                println!("{}", serde_json::to_string_pretty(&scan_result)?);
                println!();
            }

            println!("==============================================");
            println!("✅ Test Successful!");
            println!("==============================================");
        }
        Err(e) => {
            println!();
            println!("==============================================");
            println!("❌ Scan Failed");
            println!("==============================================");
            println!();
            println!("Error: {}", e);
            println!();
            return Err(e.into());
        }
    }

    // Disconnect
    println!();
    println!("Disconnecting...");
    let mut device = device.lock().await;
    device.disconnect()?;
    println!("✅ Disconnected");

    Ok(())
}
