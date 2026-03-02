//! Test spectrum analysis mode

use ubertooth_usb::device_libusb::UbertoothDeviceLibusb;
use ubertooth_usb::protocol::{UsbPacket, SpectrumPoint};
use ubertooth_usb::constants::*;
use std::time::Duration;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("Spectrum Analysis Test");
    println!("========================================\n");

    let mut device = UbertoothDeviceLibusb::new()?;
    device.connect(0)?;
    println!("✅ Connected\n");

    if let Some(info) = device.device_info() {
        println!("Device: {} ({})", info.board_name(), info.firmware_version);
        println!("Serial: {}\n", info.serial_number);
    }

    // Setup spectrum analysis
    let low_freq = 2402u16;
    let high_freq = 2480u16;

    println!("Setting up spectrum analysis...");
    println!("  Frequency range: {}-{} MHz", low_freq, high_freq);
    println!("  Channels: 0-78 (Bluetooth)\n");

    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Start spectrum analysis mode
    // wValue = low_freq, wIndex = high_freq
    device.control_transfer(CMD_SPECAN, low_freq, high_freq, &[], 1000)?;
    println!("✅ Spectrum analysis started\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("Collecting spectrum data (10 seconds)...\n");
    println!("Using bulk reads from endpoint 0x82...\n");

    let start = std::time::Instant::now();
    let mut sweep_count = 0;
    let mut total_samples = 0;
    let mut channel_stats: HashMap<u8, (i8, i8, i32, usize)> = HashMap::new(); // (min, max, sum, count)

    let mut read_count = 0;
    let mut timeout_count = 0;
    let mut error_count = 0;

    while start.elapsed() < Duration::from_secs(10) {
        let mut buffer = [0u8; 64];

        match device.bulk_read(&mut buffer, 50) {
            Ok(size) if size == 64 => {
                read_count += 1;
                if read_count <= 3 {
                    println!("\n  📦 Received {} bytes (read #{})", size, read_count);
                    println!("     Header (0-13):  {:02x?}", &buffer[..14]);
                    println!("     Payload(14-63): {:02x?}", &buffer[14..]);
                }

                // Parse USB packet
                if let Ok(usb_pkt) = UsbPacket::from_bytes(&buffer.to_vec()) {
                    if read_count <= 3 {
                        println!("     Parsed: pkt_type={}, channel={}, payload_len={}",
                            usb_pkt.header.pkt_type,
                            usb_pkt.header.channel,
                            usb_pkt.payload.len());
                    }
                    if usb_pkt.is_specan() {
                        // Parse spectrum points
                        match SpectrumPoint::from_usb_packet(&usb_pkt) {
                            Ok(points) => {
                                sweep_count += 1;
                                let point_count = points.len();

                                for point in points {
                                    total_samples += 1;

                                    let stats = channel_stats
                                        .entry(point.channel)
                                        .or_insert((point.rssi, point.rssi, 0, 0));

                                    stats.0 = stats.0.min(point.rssi); // min
                                    stats.1 = stats.1.max(point.rssi); // max
                                    stats.2 += point.rssi as i32; // sum
                                    stats.3 += 1; // count
                                }

                                // Show progress
                                if sweep_count % 100 == 0 {
                                    println!("  Sweep #{}: {} channels, {} total samples",
                                        sweep_count, point_count, total_samples);
                                }
                            }
                            Err(e) => {
                                if sweep_count < 5 {
                                    println!("⚠️  Parse error: {}", e);
                                }
                            }
                        }
                    } else {
                        if read_count <= 5 {
                            println!("  ⚠️  Packet type: {} (expected SPECAN=1)", usb_pkt.header.pkt_type);
                        }
                    }
                } else {
                    if read_count <= 5 {
                        println!("  ⚠️  Failed to parse USB packet");
                    }
                }
            }
            Ok(size) if size == 0 => {
                timeout_count += 1;
                if timeout_count <= 5 {
                    println!("  ⏱️  Timeout (no data) - count: {}", timeout_count);
                }
            }
            Ok(size) => {
                if read_count <= 5 {
                    println!("  ⚠️  Unexpected size: {} bytes", size);
                }
            }
            Err(e) => {
                error_count += 1;
                if error_count <= 5 {
                    println!("  ❌ Error: {}", e);
                }
            }
        }

        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    println!("\n📊 Statistics:");
    println!("  Successful reads: {}", read_count);
    println!("  Timeouts: {}", timeout_count);
    println!("  Errors: {}", error_count);

    println!("\n========================================");
    println!("Summary");
    println!("========================================");
    println!("  Duration: {:.1}s", start.elapsed().as_secs_f64());
    println!("  Sweeps: {}", sweep_count);
    println!("  Total samples: {}", total_samples);
    println!("  Unique channels: {}", channel_stats.len());
    println!();

    if !channel_stats.is_empty() {
        // Calculate and display channel statistics
        let mut channels: Vec<_> = channel_stats.iter().collect();
        channels.sort_by_key(|(channel, _)| **channel);

        println!("Top 20 Channels by Average RSSI:");
        println!("  Ch  Freq(MHz)  Min    Max    Avg    Samples");
        println!("  ─────────────────────────────────────────────");

        let mut sorted_by_avg: Vec<_> = channels.iter()
            .map(|(ch, (min, max, sum, count))| {
                let avg = *sum / *count as i32;
                (*ch, 2402 + **ch as u16, *min, *max, avg, *count)
            })
            .collect();
        sorted_by_avg.sort_by_key(|(_, _, _, _, avg, _)| -avg);

        for (ch, freq, min, max, avg, count) in sorted_by_avg.iter().take(20) {
            println!("  {:3}  {:4}      {:4}   {:4}   {:4}   {}",
                ch, freq, min, max, avg, count);
        }

        println!();

        // Show channels with strongest signals (likely active)
        println!("Active Channels (avg RSSI > -80 dBm):");
        let active: Vec<_> = sorted_by_avg.iter()
            .filter(|(_, _, _, _, avg, _)| *avg > -80)
            .collect();

        if active.is_empty() {
            println!("  None detected");
        } else {
            for (ch, freq, _, _, avg, _) in active {
                println!("  Channel {} ({} MHz): {} dBm", ch, freq, avg);
            }
        }
        println!();

        println!("✅ Spectrum analysis completed successfully!");
        println!("   Analyzed {} channels with {} sweeps", channel_stats.len(), sweep_count);
    } else {
        println!("⚠️  No spectrum data received");
        println!("   Device may not be in spectrum analysis mode");
    }

    println!("========================================\n");

    // Cleanup
    device.control_transfer(CMD_STOP, 0, 0, &[], 1000)?;
    device.disconnect()?;

    Ok(())
}
