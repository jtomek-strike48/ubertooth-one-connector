//! High-level USB command implementations.

use crate::constants::*;
use crate::device::UbertoothDevice;
use crate::error::UsbError;
use crate::protocol::{BlePacket, UsbPacket};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};
use ubertooth_core::error::Result;

/// High-level command executor for Ubertooth operations.
pub struct UbertoothCommands {
    /// USB device
    device: Arc<Mutex<UbertoothDevice>>,
}

/// Helper macro to convert UsbError to UbertoothError
macro_rules! usb_result {
    ($expr:expr) => {
        $expr.map_err(|e: UsbError| -> ubertooth_core::error::UbertoothError { e.into() })
    };
}

impl UbertoothCommands {
    /// Create a new command executor.
    pub fn new(device: Arc<Mutex<UbertoothDevice>>) -> Self {
        Self { device }
    }

    /// Execute device_connect command.
    pub async fn device_connect(&self, params: Value) -> Result<Value> {
        let device_index = params["device_index"].as_u64().unwrap_or(0) as usize;

        info!("Connecting to Ubertooth device (index: {})", device_index);

        let mut device = self.device.lock().await;

        // Connect to device
        usb_result!(device.connect(device_index))?;

        // Get device info
        let info = usb_result!(device
            .device_info()
            .ok_or_else(|| UsbError::InvalidPacket("Failed to get device info".to_string())))?;

        // Build capabilities list
        let capabilities = vec![
            "bt-recon",
            "bt-attack",
            "btle-recon",
            "btle-attack",
            "spectrum",
        ];

        Ok(json!({
            "success": true,
            "device_id": format!("ubertooth-{}", info.serial_number),
            "serial": info.serial_number,
            "firmware_version": info.firmware_version,
            "api_version": info.api_version,
            "board_id": info.board_id,
            "board_name": info.board_name(),
            "capabilities": capabilities,
            "message": format!("Connected to {} ({})", info.board_name(), info.firmware_version)
        }))
    }

    /// Execute device_status command.
    pub async fn device_status(&self, _params: Value) -> Result<Value> {
        let device = self.device.lock().await;

        if !device.is_connected() {
            return Ok(json!({
                "connected": false,
                "message": "No device connected"
            }));
        }

        // Ping device to check if still alive
        if usb_result!(device.ping()).is_err() {
            warn!("Device ping failed");
            return Ok(json!({
                "connected": false,
                "message": "Device not responding"
            }));
        }

        let info = device.device_info().unwrap();

        Ok(json!({
            "connected": true,
            "device_id": format!("ubertooth-{}", info.serial_number),
            "serial": info.serial_number,
            "firmware_version": info.firmware_version,
            "board_name": info.board_name(),
            "message": "Device connected and responsive"
        }))
    }

    /// Execute device_disconnect command.
    pub async fn device_disconnect(&self, _params: Value) -> Result<Value> {
        let mut device = self.device.lock().await;

        if !device.is_connected() {
            return Ok(json!({
                "success": true,
                "message": "No device was connected"
            }));
        }

        usb_result!(device.disconnect())?;

        Ok(json!({
            "success": true,
            "message": "Device disconnected successfully"
        }))
    }

    /// Execute configure_channel command.
    pub async fn configure_channel(&self, params: Value) -> Result<Value> {
        let channel = params["channel"]
            .as_u64()
            .ok_or_else(|| ubertooth_core::error::UbertoothError::InvalidParameter("channel required".to_string()))? as u8;

        // Validate channel
        if channel > BLE_CHANNEL_MAX {
            return usb_result!(Err(UsbError::InvalidParameter(format!(
                "Invalid channel: {} (max: {})",
                channel, BLE_CHANNEL_MAX
            ))));
        }

        let device = self.device.lock().await;
        usb_result!(device.set_channel(channel))?;

        Ok(json!({
            "success": true,
            "channel": channel,
            "frequency_mhz": 2402 + channel as u16,
            "message": format!("Channel set to {} ({}MHz)", channel, 2402 + channel as u16)
        }))
    }

    /// Execute configure_modulation command.
    pub async fn configure_modulation(&self, params: Value) -> Result<Value> {
        let modulation_str = params["modulation"]
            .as_str()
            .ok_or_else(|| ubertooth_core::error::UbertoothError::InvalidParameter("modulation required".to_string()))?;

        let modulation = match modulation_str {
            "bt_basic_rate" | "BR" => MOD_BT_BASIC_RATE,
            "bt_low_energy" | "BLE" => MOD_BT_LOW_ENERGY,
            "802.15.4" | "FHSS" => MOD_80211_FHSS,
            _ => {
                return usb_result!(Err(UsbError::InvalidParameter(format!(
                    "Invalid modulation: {}",
                    modulation_str
                ))))
            }
        };

        let device = self.device.lock().await;
        usb_result!(device.set_modulation(modulation))?;

        Ok(json!({
            "success": true,
            "modulation": modulation_str,
            "message": format!("Modulation set to {}", modulation_str)
        }))
    }

    /// Execute configure_power command.
    pub async fn configure_power(&self, params: Value) -> Result<Value> {
        let power_dbm = params["power_dbm"]
            .as_i64()
            .ok_or_else(|| ubertooth_core::error::UbertoothError::InvalidParameter("power_dbm required".to_string()))?
            as i8;

        let device = self.device.lock().await;
        usb_result!(device.set_power(power_dbm))?;

        Ok(json!({
            "success": true,
            "power_dbm": power_dbm,
            "message": format!("Transmit power set to {} dBm", power_dbm)
        }))
    }

    /// Execute btle_scan command (BLE advertisement scanning).
    pub async fn btle_scan(&self, params: Value) -> Result<Value> {
        let duration_sec = params["duration_sec"].as_u64().unwrap_or(30);
        let channel = params["channel"].as_u64().unwrap_or(37) as u8;
        let save_pcap = params["save_pcap"].as_bool().unwrap_or(true);

        info!(
            "Starting BLE scan: duration={}s, channel={}",
            duration_sec, channel
        );

        // Validate channel (must be advertising channel)
        if channel != 37 && channel != 38 && channel != 39 {
            return usb_result!(Err(UsbError::InvalidParameter(format!(
                "Invalid BLE advertising channel: {} (must be 37, 38, or 39)",
                channel
            ))));
        }

        let device = self.device.lock().await;

        // Configure device for BLE scanning
        usb_result!(device.set_modulation(MOD_BT_LOW_ENERGY))?;
        usb_result!(device.set_channel(channel))?;

        // Set BLE advertising access address (required for advertisement sniffing)
        let aa_bytes = BLE_ADV_ACCESS_ADDRESS.to_le_bytes();
        usb_result!(device.control_transfer(
            CMD_BTLE_SET_ACCESS_ADDRESS,
            0,
            0,
            &aa_bytes,
            USB_TIMEOUT_SHORT_MS
        ))?;
        info!("Set BLE access address to 0x{:08X}", BLE_ADV_ACCESS_ADDRESS);

        // Start BLE advertisement sniffing
        usb_result!(device.control_transfer(CMD_BTLE_SNIFF_AA, 0, 0, &[], USB_TIMEOUT_SHORT_MS))?;

        info!("BLE scan started on channel {} (sniffing advertisements)", channel);

        // Try to flush any stale data in the USB buffer
        let mut flush_buffer = vec![0u8; USB_PKT_SIZE];
        for _ in 0..5 {
            let _ = device.bulk_read(&mut flush_buffer, 10);  // Quick 10ms reads to flush
        }

        // Drop the lock while scanning
        drop(device);

        // Small delay to let device start capturing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Scan for the specified duration
        let scan_result = self.scan_ble_packets(duration_sec, channel).await?;

        // Stop scanning
        let device = self.device.lock().await;
        usb_result!(device.stop())?;

        info!(
            "BLE scan completed: {} packets, {} devices",
            scan_result.total_packets,
            scan_result.devices.len()
        );

        // Generate capture ID
        let capture_id = format!(
            "cap-btle-{}-{}",
            channel,
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        );

        // Save PCAP if requested
        let pcap_path = if save_pcap {
            let path = format!(
                "/home/{}/.ubertooth/captures/{}.pcap",
                std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
                capture_id
            );

            // Create captures directory
            if let Some(parent) = std::path::Path::new(&path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            // TODO: Write PCAP file (will be implemented in pcap.rs)
            // For now, return the intended path
            Some(path)
        } else {
            None
        };

        // Build device list
        let devices_found: Vec<Value> = scan_result
            .devices
            .into_iter()
            .map(|(mac, dev)| {
                json!({
                    "mac_address": mac,
                    "address_type": dev.address_type,
                    "device_name": dev.name.unwrap_or_else(|| "Unknown".to_string()),
                    "rssi_avg": dev.rssi_avg,
                    "packet_count": dev.packet_count
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "scan_duration_sec": duration_sec,
            "channel": channel,
            "devices_found": devices_found,
            "total_packets": scan_result.total_packets,
            "pcap_path": pcap_path,
            "preview": scan_result.preview
        }))
    }

    /// Scan for BLE packets (helper function).
    async fn scan_ble_packets(&self, duration_sec: u64, _channel: u8) -> Result<ScanResult> {
        let start_time = Instant::now();
        let scan_duration = Duration::from_secs(duration_sec);

        let mut devices: HashMap<String, DeviceStats> = HashMap::new();
        let mut total_packets = 0;
        let mut preview = Vec::new();
        let mut read_attempts = 0;
        let mut timeout_count = 0;
        let mut received_bytes = 0;

        info!("Starting packet capture loop...");

        // Read packets in a loop
        while start_time.elapsed() < scan_duration {
            // Lock device for reading
            let device = self.device.lock().await;

            // Try to read a packet (with 1 second timeout)
            let mut buffer = vec![0u8; USB_PKT_SIZE];
            read_attempts += 1;

            match device.bulk_read(&mut buffer, 1000) {
                Ok(len) => {
                    drop(device); // Release lock immediately
                    received_bytes += len;

                    info!("Received {} bytes (attempt #{}): {:02X?}", len, read_attempts, &buffer[..len]);

                    if len >= 14 {
                        // Parse USB packet
                        if let Ok(usb_pkt) = UsbPacket::from_bytes(&buffer[..len]) {
                            info!("Parsed USB packet: type={}, channel={}, payload_len={}",
                                  usb_pkt.header.pkt_type,
                                  usb_pkt.header.channel,
                                  usb_pkt.payload.len());

                            if usb_pkt.is_ble() {
                                // Parse BLE packet
                                if let Ok(ble_pkt) = BlePacket::from_usb_packet(&usb_pkt) {
                                    total_packets += 1;

                                    // Extract device info
                                    if let Some(addr) = ble_pkt.advertiser_address() {
                                        let mac = format!(
                                            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                            addr[5], addr[4], addr[3], addr[2], addr[1], addr[0]
                                        );

                                        let stats = devices.entry(mac.clone()).or_insert(DeviceStats {
                                            address_type: "public".to_string(),
                                            name: None,
                                            rssi_sum: 0,
                                            packet_count: 0,
                                            rssi_avg: 0,
                                        });

                                        stats.packet_count += 1;
                                        stats.rssi_sum += ble_pkt.rssi as i32;
                                        stats.rssi_avg = stats.rssi_sum / stats.packet_count as i32;

                                        // Try to extract device name
                                        if stats.name.is_none() {
                                            if let Some(name) = ble_pkt.device_name() {
                                                stats.name = Some(name);
                                            }
                                        }

                                        // Add to preview (first 5 packets)
                                        if preview.len() < 5 {
                                            preview.push(format!(
                                                "{}: {} (RSSI: {})",
                                                mac,
                                                stats.name.as_deref().unwrap_or("Unknown"),
                                                ble_pkt.rssi
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(UsbError::Timeout { .. }) => {
                    drop(device);
                    timeout_count += 1;
                    // Timeout is expected, just continue
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    drop(device);
                    warn!("Error reading packet: {}", e);
                    break;
                }
            }
        }

        info!(
            "Packet capture complete: {} attempts, {} timeouts, {} bytes received, {} packets parsed",
            read_attempts, timeout_count, received_bytes, total_packets
        );

        Ok(ScanResult {
            devices,
            total_packets,
            preview,
        })
    }

    /// Execute bt_specan command (spectrum analysis).
    pub async fn bt_specan(&self, params: Value) -> Result<Value> {
        let duration_sec = params["duration_sec"].as_u64().unwrap_or(10);
        let low_freq = params["low_frequency_mhz"].as_u64().unwrap_or(2402) as u16;
        let high_freq = params["high_frequency_mhz"].as_u64().unwrap_or(2480) as u16;

        info!(
            "Starting spectrum analysis: duration={}s, range={}-{} MHz",
            duration_sec, low_freq, high_freq
        );

        let device = self.device.lock().await;

        // Start spectrum analysis mode
        usb_result!(device.control_transfer(CMD_SPECAN, low_freq, high_freq, &[], USB_TIMEOUT_SHORT_MS))?;

        info!("Spectrum analysis started");

        // TODO: Implement actual spectrum data collection
        // For now, return placeholder data

        usb_result!(device.stop())?;

        Ok(json!({
            "success": true,
            "duration_sec": duration_sec,
            "low_frequency_mhz": low_freq,
            "high_frequency_mhz": high_freq,
            "spectrum_data": [],
            "message": "Spectrum analysis completed (placeholder - full implementation pending)"
        }))
    }
}

/// Device statistics collected during scanning.
#[derive(Debug, Clone)]
struct DeviceStats {
    address_type: String,
    name: Option<String>,
    rssi_sum: i32,
    packet_count: usize,
    rssi_avg: i32,
}

/// Scan result structure.
#[derive(Debug)]
struct ScanResult {
    devices: HashMap<String, DeviceStats>,
    total_packets: usize,
    preview: Vec<String>,
}
