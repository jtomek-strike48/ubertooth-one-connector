//! Ubertooth device connection and management.

use crate::constants::*;
use crate::error::{Result, UsbError};
use crate::protocol::DeviceInfo;
use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Ubertooth USB device handle with connection management.
pub struct UbertoothDevice {
    /// USB device handle (wrapped in Arc<Mutex<>> for thread safety)
    handle: Arc<Mutex<Option<DeviceHandle<Context>>>>,

    /// USB context
    context: Context,

    /// Device information
    device_info: Arc<Mutex<Option<DeviceInfo>>>,

    /// Device index (for multi-device setups)
    device_index: usize,
}

impl UbertoothDevice {
    /// Create a new device instance (not yet connected).
    pub fn new() -> Result<Self> {
        let context = Context::new()?;

        Ok(Self {
            handle: Arc::new(Mutex::new(None)),
            context,
            device_info: Arc::new(Mutex::new(None)),
            device_index: 0,
        })
    }

    /// List all connected Ubertooth devices.
    pub fn list_devices(&self) -> Result<Vec<Device<Context>>> {
        let devices = self.context.devices()?;

        let ubertooth_devices: Vec<_> = devices
            .iter()
            .filter(|device| {
                if let Ok(desc) = device.device_descriptor() {
                    desc.vendor_id() == USB_VENDOR_ID && desc.product_id() == USB_PRODUCT_ID
                } else {
                    false
                }
            })
            .collect();

        Ok(ubertooth_devices)
    }

    /// Connect to an Ubertooth device.
    ///
    /// # Arguments
    ///
    /// * `device_index` - Device index if multiple devices are present (default: 0)
    pub fn connect(&mut self, device_index: usize) -> Result<()> {
        // Check if already connected
        {
            let handle_guard = self.handle.lock().unwrap();
            if handle_guard.is_some() {
                return Err(UsbError::AlreadyOpen);
            }
        }

        info!("Searching for Ubertooth device (index: {})", device_index);

        // Find matching devices
        let devices = self.list_devices()?;

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

        let device = &devices[device_index];

        info!(
            "Found {} Ubertooth device(s), connecting to index {}",
            devices.len(),
            device_index
        );

        // Open device
        let handle = match device.open() {
            Ok(h) => h,
            Err(rusb::Error::Access) => return Err(UsbError::PermissionDenied),
            Err(e) => return Err(UsbError::from(e)),
        };

        // Detach kernel driver if necessary (Linux)
        #[cfg(target_os = "linux")]
        {
            if let Ok(true) = handle.kernel_driver_active(0) {
                debug!("Detaching kernel driver");
                handle.detach_kernel_driver(0)?;
            }
        }

        // Claim interface
        handle.claim_interface(0)?;

        info!("Successfully connected to Ubertooth device");

        // Store handle
        {
            let mut handle_guard = self.handle.lock().unwrap();
            *handle_guard = Some(handle);
        }

        self.device_index = device_index;

        // Retrieve device info
        self.refresh_device_info()?;

        Ok(())
    }

    /// Disconnect from the device.
    pub fn disconnect(&mut self) -> Result<()> {
        let mut handle_guard = self.handle.lock().unwrap();

        if let Some(handle) = handle_guard.take() {
            info!("Disconnecting from Ubertooth device");

            // Release interface
            let _ = handle.release_interface(0);

            #[cfg(target_os = "linux")]
            {
                // Reattach kernel driver
                let _ = handle.attach_kernel_driver(0);
            }

            drop(handle);
        }

        // Clear device info
        {
            let mut info_guard = self.device_info.lock().unwrap();
            *info_guard = None;
        }

        Ok(())
    }

    /// Check if device is connected.
    pub fn is_connected(&self) -> bool {
        let handle_guard = self.handle.lock().unwrap();
        handle_guard.is_some()
    }

    /// Get device information (cached).
    pub fn device_info(&self) -> Option<DeviceInfo> {
        let info_guard = self.device_info.lock().unwrap();
        info_guard.clone()
    }

    /// Refresh device information from hardware.
    fn refresh_device_info(&mut self) -> Result<()> {
        // Get board ID (required)
        let board_id = self.get_board_id()?;

        // Get firmware version (required)
        let firmware_version = self.get_firmware_version()?;

        // Get API version (optional - may not be supported on all firmware)
        let api_version = self.get_api_version()
            .unwrap_or_else(|e| {
                debug!("Failed to get API version: {}, using default", e);
                "unknown".to_string()
            });

        // Get serial number (optional)
        let serial_number = self.get_serial_number()
            .unwrap_or_else(|e| {
                debug!("Failed to get serial number: {}, using default", e);
                "unknown".to_string()
            });

        // Get compile info (optional)
        let compile_info = self.get_compile_info()
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
            let mut info_guard = self.device_info.lock().unwrap();
            *info_guard = Some(info);
        }

        Ok(())
    }

    /// Send a control transfer (vendor request).
    pub fn control_transfer(
        &self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let handle_guard = self.handle.lock().unwrap();
        let handle = handle_guard.as_ref().ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        match handle.write_control(
            USB_REQ_TYPE_OUT,
            request,
            value,
            index,
            data,
            timeout,
        ) {
            Ok(len) => Ok(len),
            Err(rusb::Error::Timeout) => Err(UsbError::Timeout { timeout_ms }),
            Err(rusb::Error::NoDevice) | Err(rusb::Error::Io) => Err(UsbError::Disconnected),
            Err(e) => Err(UsbError::ControlTransferFailed {
                cmd: request,
                details: e.to_string(),
            }),
        }
    }

    /// Read a control transfer (vendor request with data IN).
    pub fn control_transfer_read(
        &self,
        request: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let handle_guard = self.handle.lock().unwrap();
        let handle = handle_guard.as_ref().ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        match handle.read_control(
            USB_REQ_TYPE_IN,
            request,
            value,
            index,
            buffer,
            timeout,
        ) {
            Ok(len) => Ok(len),
            Err(rusb::Error::Timeout) => Err(UsbError::Timeout { timeout_ms }),
            Err(rusb::Error::NoDevice) | Err(rusb::Error::Io) => Err(UsbError::Disconnected),
            Err(e) => Err(UsbError::ControlTransferFailed {
                cmd: request,
                details: e.to_string(),
            }),
        }
    }

    /// Read bulk data from device.
    pub fn bulk_read(&self, buffer: &mut [u8], timeout_ms: u64) -> Result<usize> {
        let handle_guard = self.handle.lock().unwrap();
        let handle = handle_guard.as_ref().ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        match handle.read_bulk(ENDPOINT_DATA_IN, buffer, timeout) {
            Ok(len) => Ok(len),
            Err(rusb::Error::Timeout) => Err(UsbError::Timeout { timeout_ms }),
            Err(rusb::Error::NoDevice) | Err(rusb::Error::Io) => Err(UsbError::Disconnected),
            Err(e) => Err(UsbError::BulkTransferFailed {
                endpoint: ENDPOINT_DATA_IN,
                details: e.to_string(),
            }),
        }
    }

    /// Write bulk data to device.
    pub fn bulk_write(&self, data: &[u8], timeout_ms: u64) -> Result<usize> {
        let handle_guard = self.handle.lock().unwrap();
        let handle = handle_guard.as_ref().ok_or(UsbError::NotOpen)?;

        let timeout = Duration::from_millis(timeout_ms);

        match handle.write_bulk(ENDPOINT_DATA_OUT, data, timeout) {
            Ok(len) => Ok(len),
            Err(rusb::Error::Timeout) => Err(UsbError::Timeout { timeout_ms }),
            Err(rusb::Error::NoDevice) | Err(rusb::Error::Io) => Err(UsbError::Disconnected),
            Err(e) => Err(UsbError::BulkTransferFailed {
                endpoint: ENDPOINT_DATA_OUT,
                details: e.to_string(),
            }),
        }
    }

    // === Low-level command implementations ===

    /// Ping the device (test responsiveness).
    pub fn ping(&self) -> Result<()> {
        debug!("Sending ping command");
        self.control_transfer(CMD_PING, 0, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Reset the device.
    pub fn reset(&self) -> Result<()> {
        debug!("Sending reset command");
        self.control_transfer(CMD_RESET, 0, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Get board ID.
    fn get_board_id(&self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.control_transfer_read(
            CMD_GET_BOARD_ID,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(buffer[0])
    }

    /// Get firmware version string.
    fn get_firmware_version(&self) -> Result<String> {
        let mut buffer = [0u8; 64];
        let len = self.control_transfer_read(
            CMD_GET_REV_NUM,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes.iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Get API version.
    fn get_api_version(&self) -> Result<String> {
        let mut buffer = [0u8; 4];
        let len = self.control_transfer_read(
            CMD_GET_API_VERSION,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;

        if len >= 4 {
            Ok(format!(
                "{}.{}.{}",
                buffer[0], buffer[1], buffer[2]
            ))
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Get serial number.
    fn get_serial_number(&self) -> Result<String> {
        let mut buffer = [0u8; 64];
        let len = self.control_transfer_read(
            CMD_GET_SERIAL,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes.iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Get compile information.
    fn get_compile_info(&self) -> Result<String> {
        let mut buffer = [0u8; 256];
        let len = self.control_transfer_read(
            CMD_GET_COMPILE_INFO,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;

        // Trim null bytes and convert to string (lossy to handle non-UTF8)
        let bytes = &buffer[..len];
        let trimmed = bytes.iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<u8>>();

        Ok(String::from_utf8_lossy(&trimmed).trim().to_string())
    }

    /// Set channel.
    pub fn set_channel(&self, channel: u8) -> Result<()> {
        debug!("Setting channel to {}", channel);
        self.control_transfer(
            CMD_SET_CHANNEL,
            channel as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(())
    }

    /// Get current channel.
    pub fn get_channel(&self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.control_transfer_read(
            CMD_GET_CHANNEL,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(buffer[0])
    }

    /// Set modulation type.
    pub fn set_modulation(&self, modulation: u8) -> Result<()> {
        debug!("Setting modulation to {}", modulation);
        self.control_transfer(
            CMD_SET_MODULATION,
            modulation as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(())
    }

    /// Set transmit power.
    pub fn set_power(&self, power_dbm: i8) -> Result<()> {
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
        )?;
        Ok(())
    }

    /// Set squelch level.
    pub fn set_squelch(&self, squelch: i8) -> Result<()> {
        debug!("Setting squelch to {}", squelch);
        self.control_transfer(
            CMD_SET_SQUELCH,
            squelch as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(())
    }

    /// Stop current operation.
    pub fn stop(&self) -> Result<()> {
        debug!("Sending stop command");
        self.control_transfer(CMD_STOP, 0, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }
}

impl Drop for UbertoothDevice {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

impl Default for UbertoothDevice {
    fn default() -> Self {
        Self::new().expect("Failed to create USB context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = UbertoothDevice::new();
        assert!(device.is_ok());
    }

    #[test]
    fn test_device_not_connected() {
        let device = UbertoothDevice::new().unwrap();
        assert!(!device.is_connected());
        assert!(device.device_info().is_none());
    }
}
