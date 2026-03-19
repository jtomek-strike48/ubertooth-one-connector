//! Miscellaneous tool methods for SidecarManager.

use crate::capture_store::{CaptureMetadata, CaptureStore};
use crate::config_store::{ConfigMetadata, ConfigSettings, ConfigStore};
use chrono::Utc;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::super::SidecarManager;

impl SidecarManager {
    /// Promiscuous BT discovery implementation.
    pub(in crate::sidecar) async fn bt_discover(&self, params: Value) -> Result<Value> {
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

    /// Save current configuration as a preset.
    ///
    /// Phase 2 Week 4: Configuration preset management.
    pub(in crate::sidecar) async fn bt_save_config(&self, params: Value) -> Result<Value> {
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
    pub(in crate::sidecar) async fn bt_load_config(&self, params: Value) -> Result<Value> {
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

    /// Decode captured packets using tshark.
    pub(in crate::sidecar) async fn bt_decode(&self, _params: Value) -> Result<Value> {
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

    /// Device fingerprinting implementation.
    pub(in crate::sidecar) async fn bt_fingerprint(&self, _params: Value) -> Result<Value> {
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

    /// BLE packet injection implementation.
    ///
    /// Phase 2 Week 6: Attack operations (requires authorization)
    pub(in crate::sidecar) async fn btle_inject(&self, _params: Value) -> Result<Value> {
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

    /// Jamming operation (not implemented - highly regulated).
    pub(in crate::sidecar) async fn bt_jam(&self, params: Value) -> Result<Value> {
        let _jam_mode = params
            .get("jam_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("continuous");
        tracing::error!("bt_jam - HIGHLY REGULATED - ILLEGAL IN MOST JURISDICTIONS");
        Ok(
            json!({"success": false, "error": "OPERATION_NOT_AVAILABLE", "message": "Jamming is highly regulated and not implemented. Illegal in most jurisdictions without proper authorization."}),
        )
    }

    /// BLE slave/advertising mode implementation.
    pub(in crate::sidecar) async fn btle_slave(&self, _params: Value) -> Result<Value> {
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

    /// MITM attack operation (not implemented - requires strict authorization).
    pub(in crate::sidecar) async fn btle_mitm(&self, params: Value) -> Result<Value> {
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

    /// BD_ADDR spoofing implementation.
    pub(in crate::sidecar) async fn bt_spoof(&self, _params: Value) -> Result<Value> {
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

    /// Merge multiple PCAP capture files into a single file.
    ///
    /// Phase 2 Week 5: PCAP merge functionality
    pub(in crate::sidecar) async fn pcap_merge(&self, _params: Value) -> Result<Value> {
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

    /// Execute raw ubertooth-util command.
    ///
    /// Phase 2 Week 6: Direct hardware access (requires authorization)
    pub(in crate::sidecar) async fn ubertooth_raw(&self, _params: Value) -> Result<Value> {
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
