//! Ubertooth device implementation using nusb (async-first USB library).

use crate::constants::*;
use crate::error::{Result, UsbError};
use crate::protocol::DeviceInfo;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Ubertooth USB device handle with async support.
pub struct UbertoothDevice {
    /// nusb device interface
    interface: Arc<Mutex<Option<nusb::Interface>>>,

    /// Device information
    device_info: Arc<Mutex<Option<DeviceInfo>>>,

    /// Device index (for multi-device setups)
    device_index: usize,
}

impl UbertoothDevice {
    /// Create a new device instance (not yet connected).
    pub fn new() -> Result<Self> {
        Ok(Self {
            interface: Arc::new(Mutex::new(None)),
            device_info: Arc::new(Mutex::new(None)),
            device_index: 0,
        })
    }

    /// List all connected Ubertooth devices.
    pub async fn list_devices() -> Result<Vec<nusb::DeviceInfo>> {
        let all_devices = nusb::list_devices()
            .await
            .map_err(|e| UsbError::Other(format!("Failed to list devices: {}", e)))?;

        let devices = all_devices
            .filter(|d| d.vendor_id() == USB_VENDOR_ID && d.product_id() == USB_PRODUCT_ID)
            .collect();

        Ok(devices)
    }

    /// Connect to an Ubertooth device.
    ///
    /// # Arguments
    ///
    /// * `device_index` - Device index if multiple devices are present (default: 0)
    pub async fn connect(&mut self, device_index: usize) -> Result<()> {
        // Check if already connected
        {
            let interface_guard = self.interface.lock().await;
            if interface_guard.is_some() {
                return Err(UsbError::AlreadyOpen);
            }
        }

        info!("Searching for Ubertooth device (index: {})", device_index);

        // Find matching devices
        let devices = Self::list_devices().await?;

        if devices.is_empty() {
            return Err(UsbError::DeviceNotFound {
                vid: USB_VENDOR_ID,
                pid: USB_PRODUCT_ID,
            });
        }

        if device_index >= devices.len() {
            return Err(UsbError::DeviceNotFound {
                vid: USB_VENDOR_ID,
                pid: USB_PRODUCT_ID,
            });
        }

        let device_info = &devices[device_index];

        info!(
            "Found {} Ubertooth device(s), connecting to index {}",
            devices.len(),
            device_index
        );

        // Open device
        let device = device_info
            .open()
            .await
            .map_err(UsbError::from_nusb)?;

        // Claim interface 0
        debug!("Claiming interface 0");
        let interface = device
            .claim_interface(0)
            .await
            .map_err(UsbError::from_nusb)?;

        info!("Successfully connected to Ubertooth device");

        // Store interface
        {
            let mut interface_guard = self.interface.lock().await;
            *interface_guard = Some(interface);
        }

        self.device_index = device_index;

        // Ping device to ensure it's responsive
        debug!("Pinging device to verify connection...");
        match self.ping().await {
            Ok(_) => debug!("Device ping successful"),
            Err(e) => warn!("Device ping failed: {}, continuing anyway", e),
        }

        // Small delay to let device settle
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Retrieve device info
        self.refresh_device_info().await?;

        Ok(())
    }

    /// Disconnect from the device.
    pub async fn disconnect(&mut self) -> Result<()> {
        let mut interface_guard = self.interface.lock().await;

        if interface_guard.take().is_some() {
            info!("Disconnecting from Ubertooth device");
            // Interface will be automatically released when dropped
        }

        // Clear device info
        {
            let mut info_guard = self.device_info.lock().await;
            *info_guard = None;
        }

        Ok(())
    }

    /// Check if device is connected.
    pub async fn is_connected(&self) -> bool {
        let interface_guard = self.interface.lock().await;
        interface_guard.is_some()
    }

    /// Get device information (cached).
    pub async fn device_info(&self) -> Option<DeviceInfo> {
        let info_guard = self.device_info.lock().await;
        info_guard.clone()
    }

    /// Refresh device information from hardware.
    async fn refresh_device_info(&mut self) -> Result<()> {
        // Get board ID (optional - may not work on all firmware)
        let board_id = self
            .get_board_id()
            .await
            .unwrap_or_else(|e| {
                debug!("Failed to get board ID: {}, using default (1=Ubertooth One)", e);
                1 // Default to Ubertooth One
            });

        // Get firmware version (optional)
        let firmware_version = self
            .get_firmware_version()
            .await
            .unwrap_or_else(|e| {
                debug!("Failed to get firmware version: {}, using default", e);
                "unknown".to_string()
            });

        // Get API version (optional - may not be supported on all firmware)
        let api_version = self
            .get_api_version()
            .await
            .unwrap_or_else(|e| {
                debug!("Failed to get API version: {}, using default", e);
                "unknown".to_string()
            });

        // Get serial number (optional)
        let serial_number = self
            .get_serial_number()
            .await
            .unwrap_or_else(|e| {
                debug!("Failed to get serial number: {}, using default", e);
                "unknown".to_string()
            });

        // Get compile info (optional)
        let compile_info = self
            .get_compile_info()
            .await
            .unwrap_or_else(|e| {
                debug!("Failed to get compile info: {}, using default", e);
                "unknown".to_string()
            });

        let info = DeviceInfo {
            board_id,
            firmware_version: firmware_version.clone(),
            api_version,
            serial_number,
            compile_info,
        };

        // Check firmware compatibility
        if !info.is_firmware_compatible() {
            warn!(
                "Firmware version {} may be too old (minimum: {})",
                firmware_version, MIN_FIRMWARE_VERSION
            );
        }

        info!("Device: {} ({})", info.board_name(), firmware_version);

        // Store device info
        {
            let mut info_guard = self.device_info.lock().await;
            *info_guard = Some(info);
        }

        Ok(())
    }

    /// Send a control transfer (vendor request).
    pub async fn control_transfer(
        &self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let interface_guard = self.interface.lock().await;
        let interface = interface_guard
            .as_ref()
            .ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        let result = if data.is_empty() {
            // Control OUT with no data
            interface
                .control_out(
                    nusb::transfer::ControlOut {
                        control_type: nusb::transfer::ControlType::Vendor,
                        recipient: nusb::transfer::Recipient::Device,
                        request,
                        value,
                        index,
                        data,
                    },
                    timeout,
                )
                .await
                .map(|_| 0) // Return 0 bytes transferred for OUT
        } else {
            // Control OUT with data
            interface
                .control_out(
                    nusb::transfer::ControlOut {
                        control_type: nusb::transfer::ControlType::Vendor,
                        recipient: nusb::transfer::Recipient::Device,
                        request,
                        value,
                        index,
                        data,
                    },
                    timeout,
                )
                .await
                .map(|_| data.len())
        };

        result.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("timeout") || msg.contains("timed out") {
                UsbError::Timeout { timeout_ms }
            } else if msg.contains("disconnected") || msg.contains("no device") {
                UsbError::Disconnected
            } else {
                UsbError::ControlTransferFailed {
                    cmd: request,
                    details: msg,
                }
            }
        })
    }

    /// Read a control transfer (vendor request with data IN).
    pub async fn control_transfer_read(
        &self,
        request: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let interface_guard = self.interface.lock().await;
        let interface = interface_guard
            .as_ref()
            .ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        let result = interface
            .control_in(
                nusb::transfer::ControlIn {
                    control_type: nusb::transfer::ControlType::Vendor,
                    recipient: nusb::transfer::Recipient::Device,
                    request,
                    value,
                    index,
                    length: buffer.len() as u16,
                },
                timeout,
            )
            .await;

        match result {
            Ok(data) => {
                let len = data.len().min(buffer.len());
                buffer[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("timeout") || msg.contains("timed out") {
                    Err(UsbError::Timeout { timeout_ms })
                } else if msg.contains("disconnected") || msg.contains("no device") {
                    Err(UsbError::Disconnected)
                } else {
                    Err(UsbError::ControlTransferFailed {
                        cmd: request,
                        details: msg,
                    })
                }
            }
        }
    }

    /// Read bulk data from device (async).
    pub async fn bulk_read(&self, buffer: &mut [u8], timeout_ms: u64) -> Result<usize> {
        let interface_guard = self.interface.lock().await;
        let interface = interface_guard
            .as_ref()
            .ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        // Open bulk IN endpoint
        let mut endpoint = interface
            .endpoint::<nusb::transfer::Bulk, nusb::transfer::In>(ENDPOINT_DATA_IN)
            .map_err(UsbError::from_nusb)?;

        // Submit transfer
        endpoint.submit(nusb::transfer::Buffer::new(buffer.len()));

        // Wait for completion with timeout
        let result = tokio::time::timeout(timeout, endpoint.next_complete()).await;

        match result {
            Ok(completion) => {
                // Check for errors
                completion.status.map_err(|e| match e {
                    nusb::transfer::TransferError::Cancelled => UsbError::Timeout { timeout_ms },
                    _ => UsbError::BulkTransferFailed {
                        endpoint: ENDPOINT_DATA_IN,
                        details: e.to_string(),
                    },
                })?;

                // Copy data
                let len = completion.actual_len.min(buffer.len());
                buffer[..len].copy_from_slice(&completion.buffer[..len]);
                Ok(len)
            }
            Err(_) => Err(UsbError::Timeout { timeout_ms }),
        }
    }

    /// Write bulk data to device (async).
    pub async fn bulk_write(&self, data: &[u8], timeout_ms: u64) -> Result<usize> {
        let interface_guard = self.interface.lock().await;
        let interface = interface_guard
            .as_ref()
            .ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        // Open bulk OUT endpoint
        let mut endpoint = interface
            .endpoint::<nusb::transfer::Bulk, nusb::transfer::Out>(ENDPOINT_DATA_OUT)
            .map_err(UsbError::from_nusb)?;

        // Submit transfer
        endpoint.submit(data.to_vec().into());

        // Wait for completion with timeout
        let result = tokio::time::timeout(timeout, endpoint.next_complete()).await;

        match result {
            Ok(completion) => {
                // Check for errors
                completion.status.map_err(|e| match e {
                    nusb::transfer::TransferError::Cancelled => UsbError::Timeout { timeout_ms },
                    _ => UsbError::BulkTransferFailed {
                        endpoint: ENDPOINT_DATA_OUT,
                        details: e.to_string(),
                    },
                })?;

                Ok(data.len())
            }
            Err(_) => Err(UsbError::Timeout { timeout_ms }),
        }
    }

    // === Low-level command implementations ===

    /// Ping the device (test responsiveness).
    pub async fn ping(&self) -> Result<()> {
        debug!("Sending ping command");
        self.control_transfer(CMD_PING, 0, 0, &[], USB_TIMEOUT_SHORT_MS)
            .await?;
        Ok(())
    }

    /// Reset the device.
    pub async fn reset(&self) -> Result<()> {
        debug!("Sending reset command");
        self.control_transfer(CMD_RESET, 0, 0, &[], USB_TIMEOUT_SHORT_MS)
            .await?;
        Ok(())
    }

    /// Get board ID.
    async fn get_board_id(&self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.control_transfer_read(
            CMD_GET_BOARD_ID,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )
        .await?;
        Ok(buffer[0])
    }

    /// Get firmware version string.
    async fn get_firmware_version(&self) -> Result<String> {
        let mut buffer = [0u8; 64];
        let len = self
            .control_transfer_read(CMD_GET_REV_NUM, 0, 0, &mut buffer, USB_TIMEOUT_SHORT_MS)
            .await?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Get API version.
    async fn get_api_version(&self) -> Result<String> {
        let mut buffer = [0u8; 4];
        let len = self
            .control_transfer_read(
                CMD_GET_API_VERSION,
                0,
                0,
                &mut buffer,
                USB_TIMEOUT_SHORT_MS,
            )
            .await?;

        if len >= 4 {
            Ok(format!("{}.{}.{}", buffer[0], buffer[1], buffer[2]))
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Get serial number.
    async fn get_serial_number(&self) -> Result<String> {
        let mut buffer = [0u8; 64];
        let len = self
            .control_transfer_read(CMD_GET_SERIAL, 0, 0, &mut buffer, USB_TIMEOUT_SHORT_MS)
            .await?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Get compile information.
    async fn get_compile_info(&self) -> Result<String> {
        let mut buffer = [0u8; 256];
        let len = self
            .control_transfer_read(
                CMD_GET_COMPILE_INFO,
                0,
                0,
                &mut buffer,
                USB_TIMEOUT_SHORT_MS,
            )
            .await?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Set channel.
    pub async fn set_channel(&self, channel: u8) -> Result<()> {
        debug!("Setting channel to {}", channel);
        self.control_transfer(
            CMD_SET_CHANNEL,
            channel as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )
        .await?;
        Ok(())
    }

    /// Get current channel.
    pub async fn get_channel(&self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.control_transfer_read(CMD_GET_CHANNEL, 0, 0, &mut buffer, USB_TIMEOUT_SHORT_MS)
            .await?;
        Ok(buffer[0])
    }

    /// Set modulation type.
    pub async fn set_modulation(&self, modulation: u8) -> Result<()> {
        debug!("Setting modulation to {}", modulation);
        self.control_transfer(
            CMD_SET_MODULATION,
            modulation as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )
        .await?;
        Ok(())
    }

    /// Set transmit power.
    pub async fn set_power(&self, power_dbm: i8) -> Result<()> {
        debug!("Setting power to {} dBm", power_dbm);

        // Validate power range
        if power_dbm < TX_POWER_MIN || power_dbm > TX_POWER_MAX {
            return Err(UsbError::InvalidParameter(format!(
                "Power {} dBm out of range ({} to {})",
                power_dbm, TX_POWER_MIN, TX_POWER_MAX
            )));
        }

        self.control_transfer(
            CMD_SET_POWER,
            power_dbm as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )
        .await?;
        Ok(())
    }

    /// Set squelch level.
    pub async fn set_squelch(&self, squelch: i8) -> Result<()> {
        debug!("Setting squelch to {}", squelch);
        self.control_transfer(
            CMD_SET_SQUELCH,
            squelch as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )
        .await?;
        Ok(())
    }

    /// Stop current operation.
    pub async fn stop(&self) -> Result<()> {
        debug!("Sending stop command");
        self.control_transfer(CMD_STOP, 0, 0, &[], USB_TIMEOUT_SHORT_MS)
            .await?;
        Ok(())
    }

    /// Get the underlying nusb interface for advanced operations.
    pub async fn interface(&self) -> Option<Arc<Mutex<Option<nusb::Interface>>>> {
        Some(self.interface.clone())
    }

    /// Create a streaming packet reader for continuous packet capture.
    ///
    /// This uses nusb's optimized multi-transfer streaming pattern.
    pub async fn create_stream_reader(&self) -> Result<crate::stream_reader::StreamingPacketReader> {
        let interface_guard = self.interface.lock().await;
        let interface = interface_guard
            .as_ref()
            .ok_or(UsbError::NotOpen)?
            .clone();

        // Release lock before starting stream
        drop(interface_guard);

        crate::stream_reader::StreamingPacketReader::start(interface)
    }
}

impl Default for UbertoothDevice {
    fn default() -> Self {
        Self::new().expect("Failed to create device instance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_creation() {
        let device = UbertoothDevice::new();
        assert!(device.is_ok());
    }

    #[tokio::test]
    async fn test_device_not_connected() {
        let device = UbertoothDevice::new().unwrap();
        assert!(!device.is_connected().await);
        assert!(device.device_info().await.is_none());
    }
}
