//! Test bt_analyze Phase 2 PCAP analysis implementation

use serde_json::json;
use ubertooth_platform::backend::UbertoothBackendProvider;
use ubertooth_platform::sidecar::SidecarManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("bt_analyze Phase 2 Test");
    println!("========================================\n");

    let backend = SidecarManager::new();

    // Test capture ID (from actual capture file)
    let capture_id = "cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3";

    println!("Analyzing capture: {}", capture_id);
    println!("Expected: 258 packets, 30 second BLE scan\n");

    let params = json!({
        "capture_id": capture_id,
        "analysis_type": "auto"
    });

    println!("Executing bt_analyze...\n");
    let result = backend.execute_tool("bt_analyze", params).await?;

    // Display results
    if result["success"] == true {
        println!("✅ Analysis Complete!\n");

        let analysis = &result["analysis"];

        // Protocol Summary
        println!("=== Protocol Summary ===");
        println!("  Type: {}", analysis["protocol_summary"]["type"]);
        println!("  Packets: {}", analysis["protocol_summary"]["packet_count"]);
        println!("  Total bytes: {}", analysis["protocol_summary"]["total_bytes"]);
        println!("  Avg packet size: {:.1} bytes", analysis["protocol_summary"]["avg_packet_size"]);
        println!("  Unique devices: {}", analysis["protocol_summary"]["unique_devices"]);

        // Devices
        println!("\n=== Devices Found ===");
        if let Some(devices) = analysis["devices"].as_array() {
            println!("  Total: {} devices\n", devices.len());
            for (i, device) in devices.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, device["mac_address"]);
                if let Some(name) = device["name"].as_str() {
                    println!("     Name: {}", name);
                }
                println!("     PDU: {} | RSSI: {} dBm | Packets: {}",
                    device["pdu_type"], device["rssi"], device["packet_count"]);
            }
            if devices.len() > 10 {
                println!("  ... and {} more devices", devices.len() - 10);
            }
        }

        // Timing Analysis
        println!("\n=== Timing Analysis ===");
        let timing = &analysis["timing_analysis"];
        println!("  Duration: {:.2} sec", timing["duration_sec"]);
        println!("  Packet rate: {:.1} packets/sec", timing["packets_per_sec"]);
        println!("  Avg interval: {:.2} ms", timing["avg_interval_ms"]);
        println!("  Min interval: {:.2} ms", timing["min_interval_ms"]);
        println!("  Max interval: {:.2} ms", timing["max_interval_ms"]);
        println!("  Intervals calculated: {}", timing["intervals_calculated"]);

        // Security Summary
        println!("\n=== Security Summary ===");
        let security = &analysis["security_summary"];
        println!("  Privacy-enabled devices: {}", security["privacy_enabled_devices"]);
        println!("  Public address devices: {}", security["public_address_devices"]);
        println!("  Connection requests: {}", security["connection_requests"]);
        println!("  Scan requests: {}", security["scan_requests"]);
        println!("  Total observations: {}", security["total_observations"]);

        // Security Observations
        if let Some(observations) = analysis["security_observations"].as_array() {
            if !observations.is_empty() {
                println!("\n=== Security Observations ===");
                for obs in observations {
                    println!("\n  [{}] {}",
                        obs["severity"],
                        obs["type"]
                    );
                    println!("  {}", obs["description"]);
                }
            }
        }

        println!("\n========================================");
        println!("✅ Phase 2 PCAP Analysis Working!");
        println!("========================================");
    } else {
        println!("❌ Analysis failed: {:?}", result);
    }

    Ok(())
}
