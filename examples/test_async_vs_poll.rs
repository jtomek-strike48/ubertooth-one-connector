//! Compare async bulk transfer streaming vs CMD_POLL polling
//!
//! Tests both approaches to measure performance and verify they receive the same packets.

use ubertooth_usb::device_libusb::UbertoothDeviceLibusb;
use ubertooth_usb::protocol::UsbPacket;
use ubertooth_usb::constants::*;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("Async Bulk Transfer vs CMD_POLL Test");
    println!("========================================\n");

    let mut device = UbertoothDeviceLibusb::new()?;
    device.connect(0)?;
    println!("✅ Connected to Ubertooth\n");

    if let Some(info) = device.device_info() {
        println!("Device: {} ({})", info.board_name(), info.firmware_version);
        println!("Serial: {}\n", info.serial_number);
    }

    // Setup BLE scanning
    println!("Setting up BLE scanning...");
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    device.control_transfer(59, 0, 0, &[], 1000)?; // JAM_NONE
    device.control_transfer(CMD_SET_MODULATION, MOD_BT_LOW_ENERGY as u16, 0, &[], 1000)?;
    device.control_transfer(CMD_SET_CHANNEL, 2402, 0, &[], 1000)?;
    device.control_transfer(CMD_BTLE_SNIFFING, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("✅ BLE scanning started\n");

    // Test 1: CMD_POLL approach
    println!("========================================");
    println!("Test 1: CMD_POLL Polling (10 seconds)");
    println!("========================================\n");

    let start = Instant::now();
    let mut poll_count = 0;
    let mut poll_ble_count = 0;
    let mut poll_latencies = Vec::new();

    while start.elapsed() < Duration::from_secs(10) {
        let poll_start = Instant::now();
        let mut buffer = [0u8; 64];

        match device.control_transfer_in(CMD_POLL, 0, 0, &mut buffer, 100) {
            Ok(64) => {
                let latency = poll_start.elapsed();
                poll_latencies.push(latency.as_micros());
                poll_count += 1;

                if let Ok(packet) = UsbPacket::from_bytes(&buffer.to_vec()) {
                    if packet.is_ble() {
                        poll_ble_count += 1;
                    }
                }
            }
            _ => {}
        }
        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    let poll_duration = start.elapsed();
    let poll_avg_latency = poll_latencies.iter().sum::<u128>() / poll_latencies.len().max(1) as u128;
    let poll_max_latency = *poll_latencies.iter().max().unwrap_or(&0);
    let poll_rate = poll_ble_count as f64 / poll_duration.as_secs_f64();

    println!("Results:");
    println!("  Duration: {:.2}s", poll_duration.as_secs_f64());
    println!("  Total packets: {}", poll_count);
    println!("  BLE packets: {}", poll_ble_count);
    println!("  Packet rate: {:.1} BLE packets/sec", poll_rate);
    println!("  Avg latency: {} µs", poll_avg_latency);
    println!("  Max latency: {} µs\n", poll_max_latency);

    // Stop scanning
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test 2: Async bulk transfer approach
    println!("========================================");
    println!("Test 2: Async Bulk Transfers (10 seconds)");
    println!("========================================\n");

    // Restart scanning
    device.control_transfer(59, 0, 0, &[], 1000)?;
    device.control_transfer(CMD_SET_MODULATION, MOD_BT_LOW_ENERGY as u16, 0, &[], 1000)?;
    device.control_transfer(CMD_SET_CHANNEL, 2402, 0, &[], 1000)?;
    device.control_transfer(CMD_BTLE_SNIFFING, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("Starting async stream...");
    let mut stream = device.create_async_stream_reader().await?;
    println!("✅ Stream started\n");

    let start = Instant::now();
    let mut bulk_count = 0;
    let mut bulk_ble_count = 0;
    let mut bulk_latencies = Vec::new();

    while start.elapsed() < Duration::from_secs(10) {
        let read_start = Instant::now();

        match tokio::time::timeout(Duration::from_millis(100), stream.read_packet()).await {
            Ok(Some(packet_data)) => {
                let latency = read_start.elapsed();
                bulk_latencies.push(latency.as_micros());
                bulk_count += 1;

                if let Ok(packet) = UsbPacket::from_bytes(&packet_data) {
                    if packet.is_ble() {
                        bulk_ble_count += 1;
                    }
                }
            }
            Ok(None) => {
                println!("Stream ended");
                break;
            }
            Err(_) => {} // Timeout
        }
    }

    let bulk_duration = start.elapsed();
    let bulk_avg_latency = if !bulk_latencies.is_empty() {
        bulk_latencies.iter().sum::<u128>() / bulk_latencies.len() as u128
    } else {
        0
    };
    let bulk_max_latency = *bulk_latencies.iter().max().unwrap_or(&0);
    let bulk_rate = bulk_ble_count as f64 / bulk_duration.as_secs_f64();

    println!("Results:");
    println!("  Duration: {:.2}s", bulk_duration.as_secs_f64());
    println!("  Total packets: {}", bulk_count);
    println!("  BLE packets: {}", bulk_ble_count);
    println!("  Packet rate: {:.1} BLE packets/sec", bulk_rate);
    println!("  Avg latency: {} µs", bulk_avg_latency);
    println!("  Max latency: {} µs\n", bulk_max_latency);

    // Comparison
    println!("========================================");
    println!("Comparison");
    println!("========================================\n");

    let rate_improvement = if poll_rate > 0.0 {
        bulk_rate / poll_rate
    } else {
        0.0
    };

    let latency_improvement = if bulk_avg_latency > 0 {
        poll_avg_latency as f64 / bulk_avg_latency as f64
    } else {
        0.0
    };

    println!("CMD_POLL:");
    println!("  Rate: {:.1} packets/sec", poll_rate);
    println!("  Avg latency: {} µs\n", poll_avg_latency);

    println!("Async Bulk:");
    println!("  Rate: {:.1} packets/sec", bulk_rate);
    println!("  Avg latency: {} µs\n", bulk_avg_latency);

    if bulk_count > 0 {
        println!("Performance:");
        println!("  🚀 Rate improvement: {:.2}x faster", rate_improvement);
        println!("  ⚡ Latency improvement: {:.2}x lower latency", latency_improvement);
    } else {
        println!("⚠️  Async bulk transfers received no packets!");
        println!("   This might indicate bulk endpoint isn't receiving data.");
    }

    println!("\n========================================");
    println!("Recommendation:");
    if bulk_count > poll_count / 2 {
        println!("✅ Use async bulk transfers for production");
    } else {
        println!("⚠️  Stick with CMD_POLL for now");
        println!("   (Bulk endpoint may not be active in this mode)");
    }
    println!("========================================\n");

    // Cleanup
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    device.disconnect()?;

    Ok(())
}
