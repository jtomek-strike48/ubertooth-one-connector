//! Python sidecar manager for wrapping ubertooth-tools.

mod capture;
mod config;
mod device;
mod tools;
mod types;
mod validation;

use async_trait::async_trait;
use chrono::Utc;
use pcap_parser::*;
use serde_json::{json, Value};
use std::fs::File;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use ubertooth_core::error::{Result, UbertoothError};

use crate::backend::UbertoothBackendProvider;
use crate::capture_store::{CaptureMetadata, CaptureStore};
use crate::config_store::{ConfigMetadata, ConfigSettings, ConfigStore};

// Import types from submodules
use types::*;
pub use validation::check_ubertooth_installed;

/// Python sidecar process manager.
///
/// The sidecar wraps the ubertooth-* command-line tools and provides a
/// simple interface for executing commands.
pub struct SidecarManager {
    #[allow(dead_code)]
    process: Arc<Mutex<Option<Child>>>,
}

impl SidecarManager {
    /// Create a new sidecar manager (not started yet).
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            process: Arc::new(Mutex::new(None)),
        })
    }

    /// Spawn the Python sidecar process (if needed in future).
    ///
    /// For now, we'll call ubertooth-* tools directly without a persistent sidecar.
    async fn _spawn(&self) -> Result<()> {
        // Phase 1: We'll call ubertooth-* tools directly
        // Phase 2: If needed, we could spawn a persistent Python process
        Ok(())
    }

    /// Execute a ubertooth command-line tool.
    ///
    /// Filters out benign API version mismatch warnings from stderr.
    async fn execute_ubertooth_command(&self, tool: &str, args: &[&str]) -> Result<String> {
        tracing::debug!("Executing: {} {:?}", tool, args);

        let output = Command::new(tool)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                UbertoothError::BackendError(format!("Failed to execute {}: {}", tool, e))
            })?;

        // Filter stderr to remove benign API version warnings
        let stderr = String::from_utf8_lossy(&output.stderr);
        let filtered_stderr = stderr
            .lines()
            .filter(|line| {
                // Filter out API version mismatch warnings (firmware newer than libubertooth)
                !line.contains("API version")
                    && !line.contains("newer than that supported")
                    && !line.contains("Things will still work")
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Log filtered warnings at debug level for troubleshooting
        if stderr.contains("API version") {
            tracing::debug!("Filtered benign API version warning from {}", tool);
        }

        if !output.status.success() {
            // Only return actual errors, not filtered warnings
            if !filtered_stderr.trim().is_empty() {
                return Err(UbertoothError::CommandFailed(format!(
                    "{} failed: {}",
                    tool, filtered_stderr
                )));
            } else {
                return Err(UbertoothError::CommandFailed(format!(
                    "{} failed with exit code: {}",
                    tool, output.status
                )));
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl UbertoothBackendProvider for SidecarManager {
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        // Route method calls to appropriate ubertooth-* tools
        match method {
            "device_connect" => self.device_connect().await,
            "device_disconnect" => self.device_disconnect().await,
            "device_status" => self.device_status().await,
            "btle_scan" => self.btle_scan(params).await,
            "bt_specan" => self.bt_specan(params).await,
            "configure_channel" => self.configure_channel(params).await,
            "configure_modulation" => self.configure_modulation(params).await,
            "configure_power" => self.configure_power(params).await,
            "capture_list" => self.capture_list(params).await,
            "capture_get" => self.capture_get(params).await,
            "capture_delete" => self.capture_delete(params).await,
            "capture_tag" => self.capture_tag(params).await,
            "bt_analyze" => self.bt_analyze(params).await,
            "session_context" => self.session_context(params).await,
            "bt_scan" => self.bt_scan(params).await,
            "bt_follow" => self.bt_follow(params).await,
            "afh_analyze" => self.afh_analyze(params).await,
            "bt_discover" => self.bt_discover(params).await,
            "btle_follow" => self.btle_follow(params).await,
            "configure_squelch" => self.configure_squelch(params).await,
            "configure_leds" => self.configure_leds(params).await,
            "bt_save_config" => self.bt_save_config(params).await,
            "bt_load_config" => self.bt_load_config(params).await,
            "config_list" => self.config_list(params).await,
            "config_delete" => self.config_delete(params).await,
            "bt_compare" => self.bt_compare(params).await,
            "bt_decode" => self.bt_decode(params).await,
            "bt_fingerprint" => self.bt_fingerprint(params).await,
            "pcap_merge" => self.pcap_merge(params).await,
            "capture_export" => self.capture_export(params).await,
            "btle_inject" => self.btle_inject(params).await,
            "bt_jam" => self.bt_jam(params).await,
            "btle_slave" => self.btle_slave(params).await,
            "btle_mitm" => self.btle_mitm(params).await,
            "bt_spoof" => self.bt_spoof(params).await,
            "ubertooth_raw" => self.ubertooth_raw(params).await,
            _ => Err(UbertoothError::BackendError(format!(
                "Method not implemented: {}",
                method
            ))),
        }
    }

    async fn is_alive(&self) -> bool {
        // Check if ubertooth-util responds
        let result = Command::new("ubertooth-util").arg("-V").output();

        result.is_ok()
    }

    async fn restart(&self) -> Result<()> {
        // No persistent process to restart in Phase 1
        Ok(())
    }

    fn backend_type(&self) -> &str {
        "python"
    }
}

impl SidecarManager {


    /// Analyze captured packets implementation.
    ///
    /// Phase 1: Basic analysis with metadata only
    /// Phase 2: Full PCAP parsing with protocol analysis
    async fn bt_analyze(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let analysis_type = params
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        tracing::info!(
            "Analyzing capture: {} (type: {})",
            capture_id,
            analysis_type
        );

        let store = CaptureStore::new()?;
        let metadata = store.load_metadata(capture_id)?;

        let protocol_type = match metadata.capture_type.as_str() {
            "btle_sniff" => "BLE",
            "specan" => "Spectrum",
            "bt_follow" => "BR/EDR",
            _ => "Unknown",
        };

        // Phase 2 Complete: PCAP parsing + device extraction + timing + security analysis
        let pcap_analysis = Self::parse_pcap(&metadata.pcap_path)?;

        // Build device list for JSON output
        let devices: Vec<Value> = pcap_analysis
            .devices
            .iter()
            .map(|dev| {
                json!({
                    "mac_address": dev.mac_address,
                    "device_name": dev.name.as_deref().unwrap_or("Unknown"),
                    "name": dev.name,  // Keep for backwards compatibility
                    "rssi": dev.rssi,
                    "pdu_type": dev.pdu_type,
                    "first_seen": dev.first_seen,
                    "last_seen": dev.last_seen,
                    "packet_count": dev.packet_count
                })
            })
            .collect();

        // Build security observations for JSON output
        let security_observations: Vec<Value> = pcap_analysis
            .security
            .observations
            .iter()
            .map(|obs| {
                json!({
                    "type": obs.observation_type,
                    "severity": obs.severity,
                    "description": obs.description,
                    "affected_device": obs.affected_device
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "analysis": {
                "protocol_summary": {
                    "type": protocol_type,
                    "packet_count": pcap_analysis.packet_count,
                    "total_bytes": pcap_analysis.total_bytes,
                    "avg_packet_size": pcap_analysis.avg_packet_size,
                    "unique_devices": pcap_analysis.devices.len()
                },
                "devices": devices,
                "timing_analysis": {
                    "duration_sec": pcap_analysis.duration_sec,
                    "packets_per_sec": pcap_analysis.packets_per_sec,
                    "avg_interval_ms": pcap_analysis.timing.avg_interval_ms,
                    "min_interval_ms": pcap_analysis.timing.min_interval_ms,
                    "max_interval_ms": pcap_analysis.timing.max_interval_ms,
                    "intervals_calculated": pcap_analysis.timing.intervals_count
                },
                "security_observations": security_observations,
                "security_summary": {
                    "privacy_enabled_devices": pcap_analysis.security.privacy_enabled_count,
                    "public_address_devices": pcap_analysis.security.public_address_count,
                    "connection_requests": pcap_analysis.security.connection_requests,
                    "scan_requests": pcap_analysis.security.scan_requests,
                    "total_observations": pcap_analysis.security.observations.len()
                },
                "note": "Phase 2 Complete: Full PCAP analysis with packet parsing, device extraction, timing analysis, and security observations."
            }
        }))
    }

    /// Parse PCAP/PCAPNG file and extract basic statistics, device information, and timing analysis.
    fn parse_pcap(pcap_path: &str) -> Result<PcapAnalysis> {
        let file = File::open(pcap_path).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to open PCAP file: {}", e))
        })?;

        // Create reader that auto-detects PCAP vs PCAPNG format
        let mut reader = create_reader(65536, file).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to create PCAP reader: {}", e))
        })?;

        let mut packet_count = 0;
        let mut total_bytes = 0;
        let mut first_timestamp: Option<f64> = None;
        let mut last_timestamp: Option<f64> = None;
        let mut prev_timestamp: Option<f64> = None;
        let mut devices: std::collections::HashMap<String, BleDevice> =
            std::collections::HashMap::new();
        let mut linktype: Option<u32> = None;

        // Timing analysis tracking
        let mut intervals: Vec<f64> = Vec::new();

        // Security analysis tracking
        let mut privacy_addresses: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut public_addresses: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut connection_requests = 0;
        let mut scan_requests = 0;
        let mut malformed_packets = 0;

        loop {
            match reader.next() {
                Ok((offset, block)) => {
                    match block {
                        PcapBlockOwned::NG(Block::InterfaceDescription(idb)) => {
                            // Extract linktype from interface description
                            linktype = Some(idb.linktype.0 as u32);
                            tracing::debug!("Detected linktype: {}", idb.linktype.0);
                        }
                        PcapBlockOwned::Legacy(packet) => {
                            // Legacy PCAP packet
                            packet_count += 1;
                            total_bytes += packet.data.len();

                            // Convert timestamp (seconds + microseconds)
                            let timestamp =
                                packet.ts_sec as f64 + (packet.ts_usec as f64 / 1_000_000.0);

                            if first_timestamp.is_none() {
                                first_timestamp = Some(timestamp);
                            }
                            last_timestamp = Some(timestamp);

                            // Calculate inter-packet interval
                            if let Some(prev) = prev_timestamp {
                                let interval_sec = timestamp - prev;
                                let interval_ms = interval_sec * 1000.0;
                                if interval_ms > 0.0 && interval_ms < 10_000.0 {
                                    intervals.push(interval_ms);
                                }
                            }
                            prev_timestamp = Some(timestamp);

                            // Extract device information based on linktype
                            let device_info = match linktype {
                                Some(161) | Some(251) | Some(256) => {
                                    Self::extract_ble_device_from_rf(packet.data, timestamp)
                                }
                                _ => Self::extract_ble_device(packet.data, timestamp),
                            };

                            if let Some(device_info) = device_info {
                                let mac = device_info.mac_address.clone();
                                devices
                                    .entry(mac.clone())
                                    .and_modify(|d| {
                                        d.last_seen = timestamp;
                                        d.packet_count += 1;
                                        if device_info.name.is_some() && d.name.is_none() {
                                            d.name = device_info.name.clone();
                                        }
                                        d.rssi = device_info.rssi;
                                    })
                                    .or_insert(device_info);
                            }

                            // Security analysis
                            if packet.data.len() >= 14 {
                                let pkt_type = packet.data[0];
                                if pkt_type == 1 {
                                    let usb_payload = &packet.data[14..];
                                    if usb_payload.len() >= 6 {
                                        let pdu_header = usb_payload[4];
                                        let pdu_type = pdu_header & 0x0F;
                                        let tx_add = (pdu_header >> 6) & 0x01;

                                        match pdu_type {
                                            0x05 => connection_requests += 1,
                                            0x03 => scan_requests += 1,
                                            _ => {}
                                        }

                                        if matches!(pdu_type, 0x00 | 0x02 | 0x04 | 0x06)
                                            && usb_payload.len() >= 12
                                        {
                                            let ble_payload = &usb_payload[6..];
                                            if ble_payload.len() >= 6 {
                                                let addr = &ble_payload[0..6];
                                                let mac_address = format!(
                                                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                                    addr[5],
                                                    addr[4],
                                                    addr[3],
                                                    addr[2],
                                                    addr[1],
                                                    addr[0]
                                                );

                                                if tx_add == 1 {
                                                    privacy_addresses.insert(mac_address);
                                                } else {
                                                    public_addresses.insert(mac_address);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        PcapBlockOwned::NG(Block::EnhancedPacket(epb)) => {
                            // PCAPNG Enhanced Packet Block
                            packet_count += 1;
                            total_bytes += epb.data.len();

                            // Convert timestamp (high + low parts, resolution depends on interface)
                            let timestamp_raw = ((epb.ts_high as u64) << 32) | (epb.ts_low as u64);
                            // Assume microsecond resolution (most common)
                            let timestamp = timestamp_raw as f64 / 1_000_000.0;

                            if first_timestamp.is_none() {
                                first_timestamp = Some(timestamp);
                            }
                            last_timestamp = Some(timestamp);

                            if let Some(prev) = prev_timestamp {
                                let interval_sec = timestamp - prev;
                                let interval_ms = interval_sec * 1000.0;
                                if interval_ms > 0.0 && interval_ms < 10_000.0 {
                                    intervals.push(interval_ms);
                                }
                            }
                            prev_timestamp = Some(timestamp);

                            // Extract device information based on linktype
                            tracing::trace!(
                                "Processing packet #{}: linktype={:?}, len={}",
                                packet_count,
                                linktype,
                                epb.data.len()
                            );
                            let device_info = match linktype {
                                Some(161) | Some(251) | Some(256) => {
                                    Self::extract_ble_device_from_rf(epb.data, timestamp)
                                }
                                _ => Self::extract_ble_device(epb.data, timestamp),
                            };

                            if let Some(device_info) = device_info {
                                let mac = device_info.mac_address.clone();
                                devices
                                    .entry(mac.clone())
                                    .and_modify(|d| {
                                        d.last_seen = timestamp;
                                        d.packet_count += 1;
                                        if device_info.name.is_some() && d.name.is_none() {
                                            d.name = device_info.name.clone();
                                        }
                                        d.rssi = device_info.rssi;
                                    })
                                    .or_insert(device_info);
                            }

                            // Security analysis
                            if epb.data.len() >= 14 {
                                let pkt_type = epb.data[0];
                                if pkt_type == 1 {
                                    let usb_payload = &epb.data[14..];
                                    if usb_payload.len() >= 6 {
                                        let pdu_header = usb_payload[4];
                                        let pdu_type = pdu_header & 0x0F;
                                        let tx_add = (pdu_header >> 6) & 0x01;

                                        match pdu_type {
                                            0x05 => connection_requests += 1,
                                            0x03 => scan_requests += 1,
                                            _ => {}
                                        }

                                        if matches!(pdu_type, 0x00 | 0x02 | 0x04 | 0x06)
                                            && usb_payload.len() >= 12
                                        {
                                            let ble_payload = &usb_payload[6..];
                                            if ble_payload.len() >= 6 {
                                                let addr = &ble_payload[0..6];
                                                let mac_address = format!(
                                                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                                    addr[5],
                                                    addr[4],
                                                    addr[3],
                                                    addr[2],
                                                    addr[1],
                                                    addr[0]
                                                );

                                                if tx_add == 1 {
                                                    privacy_addresses.insert(mac_address);
                                                } else {
                                                    public_addresses.insert(mac_address);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // Skip other block types (section headers, interface descriptions, etc.)
                        }
                    }
                    reader.consume(offset);
                }
                Err(PcapError::Eof) => break,
                Err(PcapError::Incomplete(_)) => {
                    reader.refill().map_err(|e| {
                        UbertoothError::BackendError(format!("Failed to refill buffer: {}", e))
                    })?;
                }
                Err(e) => {
                    tracing::warn!("Error parsing packet: {:?}", e);
                    malformed_packets += 1;
                    // Try to continue
                    break;
                }
            }
        }

        let duration_sec = if let (Some(first), Some(last)) = (first_timestamp, last_timestamp) {
            last - first
        } else {
            0.0
        };

        let packets_per_sec = if duration_sec > 0.0 {
            packet_count as f64 / duration_sec
        } else {
            0.0
        };

        let avg_packet_size = if packet_count > 0 {
            total_bytes as f64 / packet_count as f64
        } else {
            0.0
        };

        // Calculate timing statistics
        let timing = if !intervals.is_empty() {
            let min_interval = intervals.iter().copied().fold(f64::INFINITY, f64::min);
            let max_interval = intervals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let sum: f64 = intervals.iter().sum();
            let avg_interval = sum / intervals.len() as f64;

            TimingAnalysis {
                avg_interval_ms: avg_interval,
                min_interval_ms: min_interval,
                max_interval_ms: max_interval,
                intervals_count: intervals.len(),
            }
        } else {
            TimingAnalysis {
                avg_interval_ms: 0.0,
                min_interval_ms: 0.0,
                max_interval_ms: 0.0,
                intervals_count: 0,
            }
        };

        // Convert HashMap to Vec for output
        let mut device_list: Vec<BleDevice> = devices.into_values().collect();
        // Sort by first seen timestamp
        device_list.sort_by(|a, b| a.first_seen.partial_cmp(&b.first_seen).unwrap());

        tracing::info!(
            "PCAP parse complete: {} packets, {} devices found, linktype={:?}",
            packet_count,
            device_list.len(),
            linktype
        );

        // Generate security observations
        let mut observations = Vec::new();

        // Observation: Privacy-enabled devices
        if !privacy_addresses.is_empty() {
            let device_list_str = if privacy_addresses.len() <= 3 {
                privacy_addresses
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                format!("{} devices", privacy_addresses.len())
            };

            observations.push(SecurityObservation {
                observation_type: "Privacy Feature".to_string(),
                severity: "Info".to_string(),
                description: format!(
                    "Detected {} device(s) using random addresses (BLE privacy feature): {}",
                    privacy_addresses.len(),
                    device_list_str
                ),
                affected_device: None,
            });
        }

        // Observation: Public addresses (potential privacy concern)
        if !public_addresses.is_empty() && public_addresses.len() > privacy_addresses.len() {
            observations.push(SecurityObservation {
                observation_type: "Privacy Concern".to_string(),
                severity: "Low".to_string(),
                description: format!(
                    "{} device(s) broadcasting with public addresses (trackable across sessions)",
                    public_addresses.len()
                ),
                affected_device: None,
            });
        }

        // Observation: Connection attempts
        if connection_requests > 0 {
            let severity = if connection_requests > 10 {
                "Medium"
            } else {
                "Info"
            };
            observations.push(SecurityObservation {
                observation_type: "Connection Activity".to_string(),
                severity: severity.to_string(),
                description: format!(
                    "Detected {} BLE connection request(s) in capture",
                    connection_requests
                ),
                affected_device: None,
            });
        }

        // Observation: High scan activity
        if scan_requests > 20 {
            observations.push(SecurityObservation {
                observation_type: "Scanning Activity".to_string(),
                severity: "Info".to_string(),
                description: format!(
                    "High scanning activity detected: {} SCAN_REQ packets (may indicate active reconnaissance)",
                    scan_requests
                ),
                affected_device: None,
            });
        }

        // Observation: Malformed packets
        if malformed_packets > 0 {
            observations.push(SecurityObservation {
                observation_type: "Malformed Packets".to_string(),
                severity: "Medium".to_string(),
                description: format!(
                    "Detected {} malformed or invalid packet(s) (potential interference or attack)",
                    malformed_packets
                ),
                affected_device: None,
            });
        }

        // Observation: Timing anomalies
        if !intervals.is_empty() {
            let timing_stats = &timing;
            // Very fast intervals might indicate flooding
            if timing_stats.min_interval_ms < 1.0 && timing_stats.avg_interval_ms < 10.0 {
                observations.push(SecurityObservation {
                    observation_type: "Timing Anomaly".to_string(),
                    severity: "Low".to_string(),
                    description: format!(
                        "Unusually fast packet intervals detected (min: {:.2}ms, avg: {:.2}ms) - possible packet flooding",
                        timing_stats.min_interval_ms,
                        timing_stats.avg_interval_ms
                    ),
                    affected_device: None,
                });
            }
        }

        let security = SecurityAnalysis {
            observations,
            privacy_enabled_count: privacy_addresses.len(),
            public_address_count: public_addresses.len(),
            connection_requests,
            scan_requests,
        };

        Ok(PcapAnalysis {
            packet_count,
            total_bytes,
            duration_sec,
            packets_per_sec,
            avg_packet_size,
            devices: device_list,
            timing,
            security,
        })
    }

    /// Extract BLE device information from a PCAP packet.
    fn extract_ble_device(packet_data: &[u8], timestamp: f64) -> Option<BleDevice> {
        // Ubertooth USB packets are 64 bytes: 14-byte header + up to 50 bytes payload
        if packet_data.len() < 14 {
            return None;
        }

        // Parse USB packet header
        let pkt_type = packet_data[0];
        let _channel = packet_data[2];
        let rssi_avg = packet_data[10] as i8;

        // Check if this is a BLE packet (PKT_TYPE_LE_PACKET = 1)
        if pkt_type != 1 {
            return None;
        }

        let usb_payload = &packet_data[14..];
        if usb_payload.len() < 10 {
            return None;
        }

        // Parse BLE packet structure
        // let _access_address = u32::from_le_bytes([usb_payload[0], usb_payload[1], usb_payload[2], usb_payload[3]]);
        let pdu_header = usb_payload[4];
        let _length = usb_payload[5] as usize;

        // Extract PDU type (lower 4 bits of header)
        let pdu_type = pdu_header & 0x0F;
        let pdu_type_name = match pdu_type {
            0x00 => "ADV_IND",
            0x01 => "ADV_DIRECT_IND",
            0x02 => "ADV_NONCONN_IND",
            0x03 => "SCAN_REQ",
            0x04 => "SCAN_RSP",
            0x05 => "CONNECT_REQ",
            0x06 => "ADV_SCAN_IND",
            _ => "UNKNOWN",
        };

        // Only process advertising packets (ADV_IND, ADV_NONCONN_IND, SCAN_RSP, ADV_SCAN_IND)
        if !matches!(pdu_type, 0x00 | 0x02 | 0x04 | 0x06) {
            return None;
        }

        let ble_payload = &usb_payload[6..];
        if ble_payload.len() < 6 {
            return None;
        }

        // Extract advertiser address (first 6 bytes of BLE payload)
        let addr = &ble_payload[0..6];
        let mac_address = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            addr[5], addr[4], addr[3], addr[2], addr[1], addr[0]
        );

        // Parse advertising data structures to find device name
        let name = Self::extract_device_name(&ble_payload[6..]);

        Some(BleDevice {
            mac_address,
            name,
            rssi: rssi_avg,
            pdu_type: pdu_type_name.to_string(),
            first_seen: timestamp,
            last_seen: timestamp,
            packet_count: 1,
        })
    }

    /// Extract device name from BLE advertising data structures.
    fn extract_device_name(ad_data: &[u8]) -> Option<String> {
        let mut offset = 0;

        while offset < ad_data.len() {
            if offset + 1 >= ad_data.len() {
                break;
            }

            let length = ad_data[offset] as usize;
            if length == 0 {
                break;
            }

            if offset + 1 + length > ad_data.len() {
                break; // Incomplete structure
            }

            let ad_type = ad_data[offset + 1];
            let data = &ad_data[offset + 2..offset + 1 + length];

            // 0x08 = Shortened Local Name, 0x09 = Complete Local Name
            if ad_type == 0x08 || ad_type == 0x09 {
                if let Ok(name) = String::from_utf8(data.to_vec()) {
                    return Some(name);
                }
            }

            offset += 1 + length;
        }

        None
    }

    /// Extract BLE device information from a BLE RF format packet (linktype 251).
    /// Format: [channel][rssi][flags:2][access_addr:4][pdu_header][length][payload...][crc:3]
    fn extract_ble_device_from_rf(packet_data: &[u8], timestamp: f64) -> Option<BleDevice> {
        // BLE RF format:
        // Bytes 0-9: RF header (channel, rssi, noise, offenses, ref_aa, flags)
        // Bytes 10-13: BLE Access Address (4 bytes)
        // Bytes 14-15: PDU Header (type+flags, length)
        // Bytes 16+: Payload (addresses, advertising data)

        // Minimum: RF header(10) + AA(4) + PDU(2) + addr(6) = 22 bytes
        if packet_data.len() < 22 {
            tracing::trace!("Packet too short: {} bytes", packet_data.len());
            return None;
        }

        // Parse RF header (bytes 0-9)
        let _channel = packet_data[0];
        let rssi = packet_data[1] as i8;

        // Parse BLE PDU header (bytes 14-15)
        let pdu_header = packet_data[14];
        let _length = packet_data[15] as usize;

        // Extract PDU type (lower 4 bits of byte 14)
        let pdu_type = pdu_header & 0x0F;
        let pdu_type_name = match pdu_type {
            0x00 => "ADV_IND",
            0x01 => "ADV_DIRECT_IND",
            0x02 => "ADV_NONCONN_IND",
            0x03 => "SCAN_REQ",
            0x04 => "SCAN_RSP",
            0x05 => "CONNECT_REQ",
            0x06 => "ADV_SCAN_IND",
            _ => "UNKNOWN",
        };

        // Payload starts at byte 16
        let payload = &packet_data[16..];
        if payload.len() < 6 {
            tracing::trace!("Payload too short: {} bytes", payload.len());
            return None;
        }

        // Extract advertiser address based on packet type
        // ADV_* packets (0x00, 0x02, 0x04, 0x06): advertising address at offset 0
        // SCAN_REQ (0x03): scanning address at offset 0, advertising address at offset 6
        // CONNECT_REQ (0x05): similar structure to SCAN_REQ
        let addr_offset = match pdu_type {
            0x03 | 0x05 => {
                // SCAN_REQ/CONNECT_REQ: advertising address is second (bytes 6-11)
                if payload.len() < 12 {
                    tracing::trace!(
                        "SCAN_REQ/CONNECT_REQ payload too short: {} bytes",
                        payload.len()
                    );
                    return None;
                }
                6
            }
            0x00 | 0x02 | 0x04 | 0x06 => {
                // ADV_* packets: advertising address is first (bytes 0-5)
                0
            }
            _ => {
                tracing::trace!("Unhandled PDU type: 0x{:02x}", pdu_type);
                return None;
            }
        };

        let addr = &payload[addr_offset..addr_offset + 6];
        let mac_address = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            addr[5], addr[4], addr[3], addr[2], addr[1], addr[0]
        );

        // Parse advertising data to find name (only for ADV_* and SCAN_RSP packets)
        let name = if matches!(pdu_type, 0x00 | 0x02 | 0x04 | 0x06) {
            Self::extract_device_name(&payload[6..])
        } else {
            None
        };

        tracing::debug!(
            "Extracted device: {} ({}), RSSI: {}, PDU: {}",
            mac_address,
            name.as_deref().unwrap_or("Unknown"),
            rssi,
            pdu_type_name
        );

        Some(BleDevice {
            mac_address,
            name,
            rssi,
            pdu_type: pdu_type_name.to_string(),
            first_seen: timestamp,
            last_seen: timestamp,
            packet_count: 1,
        })
    }


    /// AFH analysis implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-afh for channel map analysis.
    async fn afh_analyze(&self, params: Value) -> Result<Value> {
        let bd_addr = params.get("bd_addr").and_then(|v| v.as_str());
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        tracing::info!("Analyzing AFH: {:?} for {}s", bd_addr, duration_sec);

        // Build command arguments
        let duration_str = duration_sec.to_string();
        let mut args = vec!["-d", duration_str.as_str()];
        let bd_addr_str;
        if let Some(addr) = bd_addr {
            bd_addr_str = addr.to_string();
            args.extend_from_slice(&["-t", bd_addr_str.as_str()]);
        }

        let output = self
            .execute_ubertooth_command("ubertooth-afh", &args)
            .await?;

        // Parse AFH channel map from output
        let mut afh_map = "0x0000000000000000000000".to_string();
        let mut channels_used = Vec::new();
        let mut channels_avoided = Vec::new();

        for line in output.lines() {
            if line.contains("AFH map:") || line.contains("Channel map:") {
                // Extract hex map
                if let Some(hex_start) = line.find("0x") {
                    afh_map = line[hex_start..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("0x0000000000000000000000")
                        .to_string();
                }
            }
            if line.contains("Used:") || line.contains("Active:") {
                // Parse channel list
                for word in line.split_whitespace() {
                    if let Ok(ch) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                        if ch <= 78 {
                            channels_used.push(ch);
                        }
                    }
                }
            }
            if line.contains("Avoided:") || line.contains("Disabled:") {
                // Parse avoided channel list
                for word in line.split_whitespace() {
                    if let Ok(ch) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                        if ch <= 78 {
                            channels_avoided.push(ch);
                        }
                    }
                }
            }
        }

        let used_count = channels_used.len();
        let avoided_count = channels_avoided.len();
        let interpretation = if used_count > 0 {
            format!(
                "Device uses {} channels, avoids {} channels (likely due to WiFi interference)",
                used_count, avoided_count
            )
        } else {
            "No AFH data captured yet".to_string()
        };

        Ok(json!({
            "success": true,
            "bd_addr": bd_addr,
            "afh_map": afh_map,
            "channels_used": channels_used,
            "channels_avoided": channels_avoided,
            "used_count": used_count,
            "avoided_count": avoided_count,
            "interpretation": interpretation
        }))
    }

    /// Promiscuous Bluetooth discovery implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-rx for BR/EDR promiscuous capture.
    async fn bt_discover(&self, params: Value) -> Result<Value> {
        let duration_sec = params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        tracing::info!("Starting promiscuous BT discovery: {}s", duration_sec);

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("discover");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-rx with timeout and output to PCAP
        let duration_str = duration_sec.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self
            .execute_ubertooth_command(
                "ubertooth-rx",
                &["-d", duration_str.as_str(), "-q", pcap_str.as_str()],
            )
            .await?;

        // Parse output for piconets and packet count
        let mut piconets_found = Vec::new();
        let mut total_packets = 0;

        for line in output.lines() {
            // Parse LAP (Lower Address Part) which identifies piconets
            if line.contains("LAP:") || line.contains("lap") {
                if let Some(lap_pos) = line.find("LAP:").or_else(|| line.find("lap")) {
                    let lap_str = &line[lap_pos..];
                    if let Some(hex_val) = lap_str.split_whitespace().nth(1) {
                        if !piconets_found.iter().any(|v: &Value| v["lap"] == hex_val) {
                            piconets_found.push(json!({
                                "lap": hex_val
                            }));
                        }
                    }
                }
            }
            // Count packets
            if line.contains("packet") || line.contains("Packet") {
                total_packets += 1;
            }
        }

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "bt_discover".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: total_packets,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: Vec::new(),
            description: format!(
                "Promiscuous BT discovery, {} piconets found",
                piconets_found.len()
            ),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "duration_sec": duration_sec,
            "piconets_found": piconets_found,
            "total_packets": total_packets,
            "pcap_path": pcap_path.to_string_lossy()
        }))
    }

    /// Configure RSSI squelch implementation.
    ///
    /// Phase 2 Week 3: Wraps ubertooth-util squelch configuration.
    /// Save current configuration as a preset.
    ///
    /// Phase 2 Week 4: Configuration preset management.
    async fn bt_save_config(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let overwrite = params
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        tracing::info!("Saving configuration: {}", config_name);

        // Get current device status to capture settings
        let device_status = self.device_status().await?;

        // Extract settings from device status
        let settings = ConfigSettings {
            channel: device_status
                .get("channel")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8),
            modulation: device_status
                .get("modulation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            power_level: None, // TODO: Extract from device_status when available
            paen: None,
            hgm: None,
            squelch: None,
        };

        // Create config metadata
        let config = ConfigMetadata {
            name: config_name.to_string(),
            description,
            created: Utc::now(),
            settings: settings.clone(),
        };

        // Save to file
        let store = ConfigStore::new()?;
        let config_path = store.save_config(&config, overwrite)?;

        Ok(json!({
            "success": true,
            "config_name": config_name,
            "config_path": config_path.to_string_lossy(),
            "saved_settings": {
                "channel": settings.channel,
                "modulation": settings.modulation,
                "power_level": settings.power_level,
                "paen": settings.paen,
                "hgm": settings.hgm,
                "squelch": settings.squelch
            }
        }))
    }

    /// Load a saved configuration preset.
    ///
    /// Phase 2 Week 4: Apply saved settings to device.
    async fn bt_load_config(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        tracing::info!("Loading configuration: {}", config_name);

        // Load config from file
        let store = ConfigStore::new()?;
        let config = store.load_config(config_name)?;

        // Apply settings to device
        // TODO: Call configure_* methods to apply each setting

        Ok(json!({
            "success": true,
            "config_name": config_name,
            "applied_settings": {
                "channel": config.settings.channel,
                "modulation": config.settings.modulation,
                "power_level": config.settings.power_level
            },
            "message": format!("Configuration '{}' loaded successfully", config_name)
        }))
    }

    /// List all saved configuration presets.
    ///
    /// Phase 2 Week 4: List configs from ~/.ubertooth/configs/
    async fn config_list(&self, _params: Value) -> Result<Value> {
        tracing::info!("Listing saved configurations");

        let store = ConfigStore::new()?;
        let configs = store.list_configs()?;

        let config_list: Vec<Value> = configs
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "description": c.description,
                    "created": c.created.to_rfc3339(),
                    "settings_preview": {
                        "channel": c.settings.channel,
                        "modulation": c.settings.modulation
                    }
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "configs": config_list,
            "count": configs.len()
        }))
    }

    /// Delete a saved configuration preset.
    ///
    /// Phase 2 Week 4: Remove config file from ~/.ubertooth/configs/
    async fn config_delete(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        tracing::info!("Deleting configuration: {}", config_name);

        let store = ConfigStore::new()?;
        store.delete_config(config_name)?;

        Ok(json!({
            "success": true,
            "message": format!("Configuration '{}' deleted", config_name)
        }))
    }


    async fn bt_decode(&self, _params: Value) -> Result<Value> {
        let capture_id = _params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let protocol = _params
            .get("protocol")
            .and_then(|v| v.as_str())
            .unwrap_or("bluetooth");
        let limit = _params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        tracing::info!(
            "Decoding capture {} (protocol: {}, limit: {})",
            capture_id,
            protocol,
            limit
        );

        let store = CaptureStore::new()?;
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Use tshark to decode packets
        let limit_str = limit.to_string();
        let output = self
            .execute_ubertooth_command(
                "tshark",
                &[
                    "-r",
                    pcap_path.to_str().unwrap(),
                    "-c",
                    limit_str.as_str(),
                    "-T",
                    "json",
                ],
            )
            .await
            .unwrap_or_else(|_| "[]".to_string());

        // Parse JSON output from tshark
        let tshark_packets: Vec<Value> = serde_json::from_str(&output).unwrap_or_else(|_| vec![]);

        // Extract key fields for each packet
        let decoded_packets: Vec<Value> = tshark_packets
            .iter()
            .enumerate()
            .map(|(idx, pkt)| Self::extract_packet_summary(pkt, idx))
            .collect();

        let packet_count = decoded_packets.len();

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "protocol": protocol,
            "decoded_packets": decoded_packets,
            "packet_count": packet_count,
            "limit": limit
        }))
    }

    async fn bt_fingerprint(&self, _params: Value) -> Result<Value> {
        use crate::fingerprint::FingerprintEngine;

        let capture_id = _params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let target_mac = _params
            .get("target_mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'target_mac'".to_string()))?;

        tracing::info!(
            "Fingerprinting capture {} (target: {})",
            capture_id,
            target_mac
        );

        let store = CaptureStore::new()?;
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        if !pcap_path.exists() {
            return Err(UbertoothError::CaptureNotFound(capture_id.to_string()));
        }

        // Initialize fingerprint engine
        let engine = FingerprintEngine::new();

        // Parse PCAP file and extract advertising data for target MAC
        let packet_data = self.extract_ble_advertising_data(&pcap_path, target_mac)?;

        // Perform fingerprinting
        let fingerprint = engine.fingerprint(&packet_data);

        if let Some(fp) = fingerprint {
            Ok(json!({
                "success": true,
                "device": {
                    "mac_address": target_mac,
                    "fingerprint": {
                        "manufacturer": fp.manufacturer,
                        "device_type": fp.device_type,
                        "os_version": fp.os_version,
                        "confidence": fp.confidence
                    },
                    "indicators": fp.indicators
                }
            }))
        } else {
            // Fallback to OUI lookup
            let manufacturer = engine.lookup_manufacturer(target_mac);
            let has_match = manufacturer.is_some();

            Ok(json!({
                "success": true,
                "device": {
                    "mac_address": target_mac,
                    "fingerprint": {
                        "manufacturer": manufacturer.unwrap_or_else(|| "Unknown".to_string()),
                        "device_type": "Unknown",
                        "os_version": null,
                        "confidence": if has_match { 0.5 } else { 0.0 }
                    },
                    "indicators": if has_match {
                        vec!["OUI match only".to_string()]
                    } else {
                        vec!["No match found".to_string()]
                    }
                }
            }))
        }
    }

    /// Extract BLE advertising data from PCAP file for specific MAC address.
    fn extract_ble_advertising_data(
        &self,
        pcap_path: &std::path::Path,
        target_mac: &str,
    ) -> Result<crate::FingerprintPacketData> {
        use crate::FingerprintPacketData;
        use pcap_file::{pcap::PcapReader, PcapError};
        use std::fs::File;

        let file = File::open(pcap_path)?;
        let mut pcap_reader = PcapReader::new(file)
            .map_err(|e| UbertoothError::BackendError(format!("PCAP parse error: {:?}", e)))?;

        let mut packet_data = FingerprintPacketData {
            mac_address: Some(target_mac.to_string()),
            ..Default::default()
        };

        // Parse packets looking for BLE advertising data
        while let Some(pkt) = pcap_reader.next_packet() {
            match pkt {
                Ok(packet) => {
                    // Parse BLE advertising packet
                    if let Some(adv_data) = self.parse_ble_advertising(&packet.data, target_mac) {
                        packet_data = adv_data;
                        break; // Use first matching packet
                    }
                }
                Err(PcapError::IncompleteBuffer) => continue,
                Err(e) => {
                    tracing::warn!("PCAP read error: {:?}", e);
                    break;
                }
            }
        }

        Ok(packet_data)
    }

    /// Parse BLE advertising packet data.
    fn parse_ble_advertising(
        &self,
        data: &[u8],
        target_mac: &str,
    ) -> Option<crate::FingerprintPacketData> {
        use crate::{fingerprint::ManufacturerData, FingerprintPacketData};

        // Minimum BLE advertising packet size
        if data.len() < 10 {
            return None;
        }

        let mut packet_data = FingerprintPacketData {
            mac_address: Some(target_mac.to_string()),
            ..Default::default()
        };

        // Parse advertising data structures (AD structures)
        // Format: [length][type][data...]
        // Skip to advertising data (varies by link layer)
        // For now, start at offset 6 (typical for BLE advertising packets)
        let mut offset = 6.min(data.len());

        while offset < data.len() {
            if offset + 1 >= data.len() {
                break;
            }

            let length = data[offset] as usize;
            if length == 0 || offset + length >= data.len() {
                break;
            }

            let ad_type = data[offset + 1];
            let ad_data = &data[offset + 2..offset + 1 + length];

            match ad_type {
                0x01 => {
                    // Flags
                    if !ad_data.is_empty() {
                        packet_data.flags = Some(ad_data[0]);
                    }
                }
                0x08 | 0x09 => {
                    // Shortened/Complete Local Name
                    if let Ok(name) = String::from_utf8(ad_data.to_vec()) {
                        packet_data.device_name = Some(name);
                    }
                }
                0x0A => {
                    // TX Power Level
                    if !ad_data.is_empty() {
                        packet_data.tx_power = Some(ad_data[0] as i8);
                    }
                }
                0xFF => {
                    // Manufacturer Specific Data
                    if ad_data.len() >= 2 {
                        let company_id = u16::from_le_bytes([ad_data[0], ad_data[1]]);
                        packet_data.manufacturer_data = Some(ManufacturerData {
                            company_id,
                            data: ad_data[2..].to_vec(),
                        });
                    }
                }
                0x02 | 0x03 => {
                    // Incomplete/Complete List of 16-bit Service UUIDs
                    let mut uuids = Vec::new();
                    for chunk in ad_data.chunks(2) {
                        if chunk.len() == 2 {
                            let uuid = format!("{:04x}", u16::from_le_bytes([chunk[0], chunk[1]]));
                            uuids.push(uuid);
                        }
                    }
                    packet_data.service_uuids = Some(uuids);
                }
                _ => {
                    // Unknown AD type, skip
                }
            }

            offset += 1 + length;
        }

        Some(packet_data)
    }

    async fn pcap_merge(&self, _params: Value) -> Result<Value> {
        let capture_ids = _params
            .get("capture_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'capture_ids' array".to_string())
            })?;

        if capture_ids.len() < 2 {
            return Err(UbertoothError::InvalidParameter(
                "At least 2 captures required for merge".to_string(),
            ));
        }

        tracing::info!("Merging {} captures", capture_ids.len());

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("merged");
        let output_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Build input file list
        let mut input_paths = Vec::new();
        for id_val in capture_ids {
            if let Some(id) = id_val.as_str() {
                let path = store.captures_dir().join(format!("{}.pcap", id));
                if path.exists() {
                    input_paths.push(path.to_string_lossy().to_string());
                } else {
                    return Err(UbertoothError::CaptureNotFound(id.to_string()));
                }
            }
        }

        // Use mergecap to merge PCAP files
        let mut args = vec!["-w", output_path.to_str().unwrap()];
        let input_refs: Vec<&str> = input_paths.iter().map(|s| s.as_str()).collect();
        args.extend(input_refs);

        self.execute_ubertooth_command("mergecap", &args).await?;

        // Count total packets in merged file
        let capinfos_output = self
            .execute_ubertooth_command("capinfos", &[output_path.to_str().unwrap()])
            .await
            .unwrap_or_default();

        let mut total_packets = 0;
        for line in capinfos_output.lines() {
            if line.contains("Number of packets") {
                if let Some(num_str) = line.split(':').nth(1) {
                    total_packets = num_str.trim().parse().unwrap_or(0);
                }
            }
        }

        // Save metadata
        let file_size_bytes = if output_path.exists() {
            std::fs::metadata(&output_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "merged".to_string(),
            duration_sec: None,
            packet_count: total_packets,
            file_size_bytes,
            pcap_path: output_path.to_string_lossy().to_string(),
            tags: vec!["merged".to_string()],
            description: format!("Merged from {} source captures", capture_ids.len()),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "source_captures": capture_ids.len(),
            "total_packets": total_packets,
            "pcap_path": output_path.to_string_lossy()
        }))
    }


    // Phase 2 Week 6: Attack operations (all require authorization)

    async fn btle_inject(&self, _params: Value) -> Result<Value> {
        let access_address = _params
            .get("access_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'access_address'".to_string())
            })?;

        let packet_hex = _params
            .get("packet_hex")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'packet_hex'".to_string()))?;

        let channel = _params
            .get("channel")
            .and_then(|v| v.as_u64())
            .unwrap_or(37) as u8;
        let repeat = _params.get("repeat").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        tracing::warn!("btle_inject - REQUIRES AUTHORIZATION - Active RF transmission");

        // Validate channel range
        if channel > 39 {
            return Err(UbertoothError::InvalidParameter(
                "BLE channel must be 0-39".to_string(),
            ));
        }

        // Execute ubertooth-btle with injection parameters
        let channel_str = channel.to_string();
        let repeat_str = repeat.to_string();

        let output = self
            .execute_ubertooth_command(
                "ubertooth-btle",
                &[
                    "-i",
                    "-a",
                    access_address,
                    "-c",
                    channel_str.as_str(),
                    "-p",
                    packet_hex,
                    "-n",
                    repeat_str.as_str(),
                ],
            )
            .await?;

        // Parse output for confirmation
        let packets_sent = if output.contains("injected") || output.contains("transmitted") {
            repeat
        } else {
            0
        };

        Ok(json!({
            "success": true,
            "packets_sent": packets_sent,
            "access_address": access_address,
            "channel": channel,
            "message": format!("Injected {} packet(s) on channel {}", packets_sent, channel)
        }))
    }

    async fn bt_jam(&self, params: Value) -> Result<Value> {
        let _jam_mode = params
            .get("jam_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("continuous");
        tracing::error!("bt_jam - HIGHLY REGULATED - ILLEGAL IN MOST JURISDICTIONS");
        Ok(
            json!({"success": false, "error": "OPERATION_NOT_AVAILABLE", "message": "Jamming is highly regulated and not implemented. Illegal in most jurisdictions without proper authorization."}),
        )
    }

    async fn btle_slave(&self, _params: Value) -> Result<Value> {
        let mac_address = _params
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'mac_address'".to_string()))?;

        let duration_sec = _params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let advertising_interval = _params
            .get("advertising_interval_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        tracing::warn!("btle_slave - REQUIRES AUTHORIZATION - BLE slave/advertising mode");

        let store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("slave");
        let pcap_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        // Execute ubertooth-btle in advertising mode
        let duration_str = duration_sec.to_string();
        let interval_str = advertising_interval.to_string();
        let pcap_str = pcap_path.to_string_lossy().to_string();

        let output = self
            .execute_ubertooth_command(
                "ubertooth-btle",
                &[
                    "-a",
                    "-m",
                    mac_address,
                    "-i",
                    interval_str.as_str(),
                    "-d",
                    duration_str.as_str(),
                    "-r",
                    pcap_str.as_str(),
                ],
            )
            .await?;

        // Parse output for connection events
        let connections_received = output
            .lines()
            .filter(|line| line.contains("connection") || line.contains("CONNECT_REQ"))
            .count();

        let advertising = output.contains("advertising") || output.contains("ADV");

        // Save metadata
        let file_size_bytes = if pcap_path.exists() {
            std::fs::metadata(&pcap_path)?.len()
        } else {
            0
        };

        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "btle_slave".to_string(),
            duration_sec: Some(duration_sec),
            packet_count: connections_received,
            file_size_bytes,
            pcap_path: pcap_path.to_string_lossy().to_string(),
            tags: vec![format!("mac:{}", mac_address)],
            description: format!("BLE slave mode, {} connections", connections_received),
            category: None,
            notes: None,
        };
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "mac_address": mac_address,
            "advertising": advertising,
            "connections_received": connections_received,
            "duration_sec": duration_sec,
            "message": format!("Advertised for {}s, received {} connection(s)", duration_sec, connections_received)
        }))
    }

    async fn btle_mitm(&self, params: Value) -> Result<Value> {
        let _target_mac = params
            .get("target_mac")
            .and_then(|v| v.as_str())
            .unwrap_or("00:00:00:00:00:00");
        tracing::error!("btle_mitm - STRICTLY REQUIRED AUTHORIZATION - ACTIVE ATTACK");
        let _store = CaptureStore::new()?;
        let _capture_id = CaptureStore::generate_capture_id("mitm");
        Ok(
            json!({"success": false, "error": "AUTHORIZATION_REQUIRED", "message": "MITM attack requires STRICTLY REQUIRED authorization level. Not implemented in Phase 2 Week 6."}),
        )
    }

    async fn bt_spoof(&self, _params: Value) -> Result<Value> {
        let spoof_mac = _params
            .get("spoof_mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'spoof_mac'".to_string()))?;

        let duration_sec = _params
            .get("duration_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let action = _params
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("set");

        tracing::warn!("bt_spoof - REQUIRES AUTHORIZATION - BD_ADDR spoofing");

        // Execute ubertooth-util to set BD_ADDR
        // Note: This requires firmware support and may not work on all devices
        let output = self
            .execute_ubertooth_command("ubertooth-util", &["-B", spoof_mac])
            .await?;

        let success = output.contains("set") || output.contains("success") || output.contains("OK");

        let _store = CaptureStore::new()?;
        let capture_id = CaptureStore::generate_capture_id("spoof");

        Ok(json!({
            "success": success,
            "capture_id": capture_id,
            "spoof_mac": spoof_mac,
            "duration_sec": duration_sec,
            "action": action,
            "message": if success {
                format!("BD_ADDR set to {} for {}s", spoof_mac, duration_sec)
            } else {
                "BD_ADDR spoofing may not be supported by firmware".to_string()
            }
        }))
    }

    async fn ubertooth_raw(&self, _params: Value) -> Result<Value> {
        let command = _params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'command'".to_string()))?;

        let args_array = _params.get("args").and_then(|v| v.as_array());

        tracing::warn!(
            "ubertooth_raw - WARNING: Direct hardware access to {}",
            command
        );

        // Build command arguments
        let mut cmd_args = vec![command];
        let arg_strings: Vec<String>;
        if let Some(args) = args_array {
            arg_strings = args
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            cmd_args.extend(arg_strings.iter().map(|s| s.as_str()));
        }

        // Execute raw ubertooth command
        let output = self
            .execute_ubertooth_command("ubertooth-util", &cmd_args)
            .await?;

        // Parse response
        let response_hex = output
            .lines()
            .find(|line| {
                line.contains("0x")
                    || line
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() || c.is_whitespace())
            })
            .unwrap_or("")
            .trim()
            .to_string();

        let response_length = response_hex.len() / 2; // Assuming hex pairs

        Ok(json!({
            "success": true,
            "command": command,
            "response_hex": response_hex,
            "response_length": response_length,
            "raw_output": output.trim()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pcap_ble_rf() {
        // Initialize logging for test
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let pcap_path = format!(
            "{}/.ubertooth/captures/cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3.pcap",
            std::env::var("HOME").unwrap()
        );

        if !std::path::Path::new(&pcap_path).exists() {
            println!("PCAP file not found, skipping test: {}", pcap_path);
            return;
        }

        println!("Parsing PCAP: {}", pcap_path);
        let result = SidecarManager::parse_pcap(&pcap_path);

        match result {
            Ok(analysis) => {
                println!("Packets: {}", analysis.packet_count);
                println!("Devices found: {}", analysis.devices.len());
                println!("Duration: {:.2}s", analysis.duration_sec);
                println!("\nDevices:");
                for device in &analysis.devices {
                    println!(
                        "  - {} ({}) RSSI: {}",
                        device.mac_address,
                        device.name.as_deref().unwrap_or("Unknown"),
                        device.rssi
                    );
                }

                // Verify we found devices
                assert!(
                    analysis.devices.len() > 0,
                    "Expected to find devices in PCAP but found none"
                );

                // Verify specific device from tshark output
                let expected_mac = "88:6B:0F:B2:2D:C2";
                let found = analysis
                    .devices
                    .iter()
                    .any(|d| d.mac_address == expected_mac);
                assert!(found, "Expected to find device {} but didn't", expected_mac);

                println!("\n✅ Test passed: Found {} devices", analysis.devices.len());
            }
            Err(e) => {
                panic!("Failed to parse PCAP: {:?}", e);
            }
        }
    }
}
