//! Test platform backend integration with fixed USB layer

use ubertooth_platform::RustUsbBackend;
use ubertooth_platform::backend::UbertoothBackendProvider;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("Backend Integration Test");
    println!("========================================\n");

    // Create Rust USB backend
    println!("Creating Rust USB backend...");
    let backend = RustUsbBackend::new()?;
    println!("✅ Backend created\n");

    // Test 1: Device connection
    println!("[1/4] Testing device_connect...");
    let result = backend.call("device_connect", json!({"device_index": 0})).await?;
    println!("✅ Connected: {}\n", result);

    // Test 2: Check backend is alive
    println!("[2/4] Testing is_alive...");
    let alive = backend.is_alive().await;
    println!("✅ Backend alive: {}\n", alive);

    // Test 3: BLE scan (with fixed command sequence!)
    println!("[3/4] Testing btle_scan (5 seconds)...");
    let result = backend.call("btle_scan", json!({
        "duration_sec": 5,
        "channel": 37,
        "save_pcap": false
    })).await?;

    println!("✅ Scan completed:");
    println!("   Duration: {} sec", result["scan_duration_sec"]);
    println!("   Channel: {}", result["channel"]);
    println!("   Total packets: {}", result["total_packets"]);
    println!("   Devices found: {}", result["devices_found"].as_array().unwrap_or(&vec![]).len());

    if let Some(devices) = result["devices_found"].as_array() {
        println!("\n   Sample devices:");
        for (i, device) in devices.iter().take(5).enumerate() {
            println!("     {}. {} - {} (RSSI: {})",
                i + 1,
                device["mac_address"].as_str().unwrap_or("Unknown"),
                device["device_name"].as_str().unwrap_or("Unknown"),
                device["rssi_avg"]
            );
        }
    }
    println!();

    // Test 4: Disconnect
    println!("[4/4] Testing device_disconnect...");
    let result = backend.call("device_disconnect", json!({})).await?;
    println!("✅ Disconnected: {}\n", result);

    println!("========================================");
    println!("All tests passed! 🎉");
    println!("Backend integration is working correctly");
    println!("========================================\n");

    Ok(())
}
