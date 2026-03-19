//! Capture comparison methods for SidecarManager.

use crate::capture_store::CaptureStore;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Compare two captures implementation.
    ///
    /// Phase 2 Week 5: Analysis tools
    pub(in crate::sidecar) async fn bt_compare(&self, _params: Value) -> Result<Value> {
        let capture_id_a = _params
            .get("capture_id_a")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'capture_id_a'".to_string())
            })?;

        let capture_id_b = _params
            .get("capture_id_b")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'capture_id_b'".to_string())
            })?;

        let mode = _params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("packets");

        tracing::info!(
            "Comparing captures {} and {} (mode: {})",
            capture_id_a,
            capture_id_b,
            mode
        );

        let store = CaptureStore::new()?;

        // Load metadata for both captures
        let meta_a = store.load_metadata(capture_id_a)?;
        let meta_b = store.load_metadata(capture_id_b)?;

        // Use capinfos to get detailed stats
        let pcap_a = store.captures_dir().join(format!("{}.pcap", capture_id_a));
        let pcap_b = store.captures_dir().join(format!("{}.pcap", capture_id_b));

        let _stats_a = self
            .execute_ubertooth_command("capinfos", &[pcap_a.to_str().unwrap()])
            .await
            .unwrap_or_default();
        let _stats_b = self
            .execute_ubertooth_command("capinfos", &[pcap_b.to_str().unwrap()])
            .await
            .unwrap_or_default();

        // Basic comparison based on metadata
        let common_packets = std::cmp::min(meta_a.packet_count, meta_b.packet_count);
        let unique_to_a = meta_a.packet_count.saturating_sub(common_packets);
        let unique_to_b = meta_b.packet_count.saturating_sub(common_packets);
        let total = meta_a.packet_count + meta_b.packet_count;
        let similarity_percent = if total > 0 {
            (common_packets as f64 * 200.0) / total as f64
        } else {
            0.0
        };

        let mut differences = Vec::new();
        if meta_a.capture_type != meta_b.capture_type {
            differences.push(format!(
                "Capture types differ: {} vs {}",
                meta_a.capture_type, meta_b.capture_type
            ));
        }
        if meta_a.packet_count != meta_b.packet_count {
            differences.push(format!(
                "Packet counts differ: {} vs {}",
                meta_a.packet_count, meta_b.packet_count
            ));
        }

        Ok(json!({
            "success": true,
            "comparison": {
                "mode": mode,
                "similarity_percent": similarity_percent,
                "differences": differences,
                "unique_to_a": unique_to_a,
                "unique_to_b": unique_to_b,
                "common_packets": common_packets
            },
            "capture_a": {
                "id": capture_id_a,
                "type": meta_a.capture_type,
                "packets": meta_a.packet_count
            },
            "capture_b": {
                "id": capture_id_b,
                "type": meta_b.capture_type,
                "packets": meta_b.packet_count
            }
        }))
    }

    /// Extract key fields from tshark JSON packet for display
    pub(in crate::sidecar) fn extract_packet_summary(packet: &Value, index: usize) -> Value {
        let layers = packet.get("_source").and_then(|s| s.get("layers"));

        // Extract frame info
        let frame = layers.and_then(|l| l.get("frame"));
        let timestamp = frame
            .and_then(|f| f.get("frame.time"))
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");
        let frame_number = frame
            .and_then(|f| f.get("frame.number"))
            .and_then(|n| n.as_str())
            .unwrap_or("0");

        // Extract BLE RF info
        let btle_rf = layers.and_then(|l| l.get("btle_rf"));
        let channel = btle_rf
            .and_then(|rf| rf.get("btle_rf.channel"))
            .and_then(|c| c.as_str())
            .unwrap_or("?");
        let rssi = btle_rf
            .and_then(|rf| rf.get("btle_rf.signal_dbm"))
            .and_then(|r| r.as_str())
            .unwrap_or("?");

        // Extract BLE packet info
        let btle = layers.and_then(|l| l.get("btle"));
        let access_addr = btle
            .and_then(|b| b.get("btle.access_address"))
            .and_then(|a| a.as_str())
            .unwrap_or("Unknown");

        // Determine packet type
        let pdu_type = if let Some(adv_header) =
            btle.and_then(|b| b.get("btle.advertising_header.pdu_type"))
        {
            let pdu_val = adv_header.as_str().unwrap_or("?");
            match pdu_val {
                "0x00" | "0" => "ADV_IND",
                "0x01" | "1" => "ADV_DIRECT_IND",
                "0x02" | "2" => "ADV_NONCONN_IND",
                "0x03" | "3" => "SCAN_REQ",
                "0x04" | "4" => "SCAN_RSP",
                "0x05" | "5" => "CONNECT_REQ",
                "0x06" | "6" => "ADV_SCAN_IND",
                _ => "ADV",
            }
        } else if btle.and_then(|b| b.get("btle.data_header")).is_some() {
            "DATA"
        } else {
            "UNKNOWN"
        };

        // Extract MAC addresses
        let adv_addr = btle
            .and_then(|b| b.get("btle.advertising_address"))
            .and_then(|a| a.as_str())
            .or_else(|| {
                btle.and_then(|b| b.get("btle.initiator_address"))
                    .and_then(|a| a.as_str())
            })
            .unwrap_or("N/A");

        let scan_addr = btle
            .and_then(|b| b.get("btle.scanning_address"))
            .and_then(|a| a.as_str())
            .or_else(|| {
                btle.and_then(|b| b.get("btle.advertiser_address"))
                    .and_then(|a| a.as_str())
            })
            .unwrap_or("");

        // Create summary text
        let mac_display = if !scan_addr.is_empty() && scan_addr != adv_addr {
            format!("{} ← {}", adv_addr, scan_addr)
        } else {
            adv_addr.to_string()
        };

        // Determine protocol layer
        let protocol = if layers.and_then(|l| l.get("btl2cap")).is_some() {
            "L2CAP"
        } else if layers.and_then(|l| l.get("btatt")).is_some() {
            "ATT"
        } else if layers.and_then(|l| l.get("btsmp")).is_some() {
            "SMP"
        } else {
            "BLE"
        };

        // Create summary
        let summary = if pdu_type == "SCAN_REQ" {
            format!("Scan request to {}", adv_addr)
        } else if pdu_type == "SCAN_RSP" {
            "Scan response".to_string()
        } else if pdu_type.starts_with("ADV") {
            format!("Advertisement from {}", adv_addr)
        } else if pdu_type == "CONNECT_REQ" {
            format!("Connection request to {}", adv_addr)
        } else {
            "Data packet".to_string()
        };

        json!({
            "index": index,
            "frame_number": frame_number,
            "timestamp": timestamp,
            "channel": channel,
            "rssi": rssi,
            "packet_type": pdu_type,
            "mac_address": mac_display,
            "protocol": protocol,
            "summary": summary,
            "access_addr": access_addr,
            // Include full packet data for expanded view
            "full_packet": packet
        })
    }
}
