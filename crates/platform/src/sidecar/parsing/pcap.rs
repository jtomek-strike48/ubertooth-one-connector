//! PCAP parsing methods for SidecarManager.

use crate::sidecar::types::*;
use pcap_parser::*;
use std::fs::File;
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Parse PCAP/PCAPNG file and extract basic statistics, device information, and timing analysis.
    pub(in crate::sidecar) fn parse_pcap(pcap_path: &str) -> Result<PcapAnalysis> {
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

    /// Extract BLE device information from Ubertooth USB packet format.
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
}
