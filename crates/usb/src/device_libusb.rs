//! Pure libusb-1.0 device implementation
//!
//! This module uses direct libusb-1.0 FFI for all USB operations,
//! matching the proven Python ubertooth-btle implementation.

use crate::constants::*;
use crate::error::{Result, UsbError};
use crate::libusb_ffi::*;
use crate::protocol::DeviceInfo;
use std::ffi::c_void;
use std::ptr;
use tracing::{debug, info, warn};

/// Ubertooth device using pure libusb-1.0
pub struct UbertoothDeviceLibusb {
    context: *mut c_void,
    handle: Option<*mut c_void>,
    device_info: Option<DeviceInfo>,
}

impl UbertoothDeviceLibusb {
    /// Create a new device instance
    pub fn new() -> Result<Self> {
        unsafe {
            let mut ctx: *mut c_void = ptr::null_mut();
            let ret = libusb_init(&mut ctx);
            if ret != LIBUSB_SUCCESS {
                return Err(UsbError::Other(format!(
                    "libusb_init failed: {}",
                    error_name(ret)
                )));
            }

            debug!("libusb context initialized");

            Ok(Self {
                context: ctx,
                handle: None,
                device_info: None,
            })
        }
    }

    /// Connect to an Ubertooth device
    pub fn connect(&mut self, device_index: usize) -> Result<()> {
        unsafe {
            info!("Searching for Ubertooth device (index: {})", device_index);

            // Get device list
            let mut list: *mut *mut c_void = ptr::null_mut();
            let count = libusb_get_device_list(self.context, &mut list);
            if count < 0 {
                return Err(UsbError::DeviceNotFound {
                    vid: USB_VENDOR_ID,
                    pid: USB_PRODUCT_ID,
                });
            }

            // Find Ubertooth devices
            let mut ubertooth_devices = Vec::new();
            for i in 0..count {
                let dev = *list.offset(i as isize);
                let mut desc = std::mem::zeroed::<DeviceDescriptor>();

                if libusb_get_device_descriptor(dev, &mut desc) == LIBUSB_SUCCESS {
                    if desc.id_vendor == USB_VENDOR_ID && desc.id_product == USB_PRODUCT_ID {
                        ubertooth_devices.push(dev);
                    }
                }
            }

            if ubertooth_devices.is_empty() {
                libusb_free_device_list(list, 1);
                return Err(UsbError::DeviceNotFound {
                    vid: USB_VENDOR_ID,
                    pid: USB_PRODUCT_ID,
                });
            }

            info!("Found {} Ubertooth device(s), connecting to index {}", ubertooth_devices.len(), device_index);

            if device_index >= ubertooth_devices.len() {
                libusb_free_device_list(list, 1);
                return Err(UsbError::InvalidParameter(format!(
                    "Device index {} out of range (found {} devices)",
                    device_index,
                    ubertooth_devices.len()
                )));
            }

            // Open device
            let target_device = ubertooth_devices[device_index];
            let mut handle: *mut c_void = ptr::null_mut();
            let ret = libusb_open(target_device, &mut handle);

            libusb_free_device_list(list, 1);

            if ret != LIBUSB_SUCCESS {
                return Err(UsbError::Other(format!(
                    "libusb_open failed: {}",
                    error_name(ret)
                )));
            }

            debug!("Device opened successfully");

            // NOTE: Skip libusb_set_configuration - C code doesn't call it
            // Calling it may reset device state
            // let ret = libusb_set_configuration(handle, 1);
            // debug!("libusb_set_configuration returned: {} ({})", ret, error_name(ret));
            // if ret != LIBUSB_SUCCESS && ret != LIBUSB_ERROR_BUSY {
            //     // BUSY is OK, means already configured
            //     warn!("Set configuration returned: {}", error_name(ret));
            // }

            // Detach kernel driver if active (Linux)
            #[cfg(target_os = "linux")]
            {
                let ret = libusb_kernel_driver_active(handle, 0);
                if ret == 1 {
                    debug!("Detaching kernel driver");
                    let ret = libusb_detach_kernel_driver(handle, 0);
                    if ret != LIBUSB_SUCCESS {
                        warn!("Failed to detach kernel driver: {}", error_name(ret));
                    }
                }
            }

            // Claim interface
            debug!("Claiming interface 0");
            let ret = libusb_claim_interface(handle, 0);
            if ret != LIBUSB_SUCCESS {
                libusb_close(handle);
                return Err(UsbError::Other(format!(
                    "Failed to claim interface: {}",
                    error_name(ret)
                )));
            }

            self.handle = Some(handle);
            info!("Successfully connected to Ubertooth device");

            // NOTE: Skip device_info during connect - doing control transfers here
            // seems to interfere with subsequent bulk transfers.
            // Device info can be queried later if needed.

            Ok(())
        }
    }

    /// Disconnect from device
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            info!("Disconnecting from Ubertooth device");
            unsafe {
                libusb_release_interface(handle, 0);
                libusb_close(handle);
            }
        }
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.handle.is_some()
    }

    /// Get device info
    pub fn device_info(&self) -> Option<&DeviceInfo> {
        self.device_info.as_ref()
    }

    /// Perform a control transfer
    pub fn control_transfer(
        &self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let handle = self.handle.ok_or(UsbError::NotOpen)?;

        unsafe {
            let ret = if data.is_empty() {
                libusb_control_transfer(
                    handle,
                    LIBUSB_REQUEST_TYPE_VENDOR | LIBUSB_ENDPOINT_OUT,
                    request,
                    value,
                    index,
                    ptr::null_mut(),
                    0,
                    timeout_ms as u32,
                )
            } else {
                libusb_control_transfer(
                    handle,
                    LIBUSB_REQUEST_TYPE_VENDOR | LIBUSB_ENDPOINT_OUT,
                    request,
                    value,
                    index,
                    data.as_ptr() as *mut u8,
                    data.len() as u16,
                    timeout_ms as u32,
                )
            };

            if ret < 0 {
                return Err(UsbError::ControlTransferFailed {
                    cmd: request,
                    details: error_name(ret).to_string(),
                });
            }

            Ok(ret as usize)
        }
    }

    /// Perform a control transfer with response data
    pub fn control_transfer_in(
        &self,
        request: u8,
        value: u16,
        index: u16,
        buffer: &mut [u8],
        timeout_ms: u64,
    ) -> Result<usize> {
        let handle = self.handle.ok_or(UsbError::NotOpen)?;

        unsafe {
            let ret = libusb_control_transfer(
                handle,
                LIBUSB_REQUEST_TYPE_VENDOR | LIBUSB_ENDPOINT_IN,
                request,
                value,
                index,
                buffer.as_mut_ptr(),
                buffer.len() as u16,
                timeout_ms as u32,
            );

            if ret < 0 {
                return Err(UsbError::ControlTransferFailed {
                    cmd: request,
                    details: format!("in: {}", error_name(ret)),
                });
            }

            Ok(ret as usize)
        }
    }

    /// Perform a synchronous bulk read from endpoint 0x82
    pub fn bulk_read(&self, buffer: &mut [u8], timeout_ms: u64) -> Result<usize> {
        let handle = self.handle.ok_or(UsbError::NotOpen)?;

        unsafe {
            let mut transferred: i32 = 0;
            let ret = libusb_bulk_transfer(
                handle,
                ENDPOINT_DATA_IN,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                &mut transferred,
                timeout_ms as u32,
            );

            if ret < 0 {
                // Timeout is not necessarily an error for bulk reads
                if ret == LIBUSB_ERROR_TIMEOUT {
                    return Ok(0);
                }
                return Err(UsbError::BulkTransferFailed {
                    endpoint: ENDPOINT_DATA_IN,
                    details: error_name(ret).to_string(),
                });
            }

            Ok(transferred as usize)
        }
    }

    /// Ping device
    pub fn ping(&self) -> Result<()> {
        debug!("Sending ping command");
        self.control_transfer(CMD_PING, 0, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Stop current operation
    pub fn stop(&self) -> Result<()> {
        debug!("Sending stop command");
        self.control_transfer(CMD_STOP, 0, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Set modulation mode
    pub fn set_modulation(&self, mode: u8) -> Result<()> {
        debug!("Setting modulation to {} (CMD={})", mode, CMD_SET_MODULATION);
        self.control_transfer(CMD_SET_MODULATION, mode as u16, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Set channel
    pub fn set_channel(&self, channel: u8) -> Result<()> {
        debug!("Setting channel to {}", channel);
        self.control_transfer(CMD_SET_CHANNEL, channel as u16, 0, &[], USB_TIMEOUT_SHORT_MS)?;
        Ok(())
    }

    /// Set transmit power
    pub fn set_power(&self, power_dbm: i8) -> Result<()> {
        debug!("Setting power to {} dBm", power_dbm);
        self.control_transfer(
            CMD_SET_POWER,
            power_dbm as u16,
            0,
            &[],
            USB_TIMEOUT_SHORT_MS,
        )?;
        Ok(())
    }

    /// Get raw device handle for async operations
    pub(crate) fn raw_handle(&self) -> Option<*mut c_void> {
        self.handle
    }

    /// Get raw context for async operations
    pub(crate) fn raw_context(&self) -> *mut c_void {
        self.context
    }

    /// Create an async stream reader for bulk packet capture
    pub async fn create_async_stream_reader(&self) -> Result<crate::libusb_stream::LibusbAsyncReader> {
        let raw_handle = self.raw_handle().ok_or(UsbError::NotOpen)?;
        let raw_context = self.raw_context();
        crate::libusb_stream::LibusbAsyncReader::start(
            raw_handle,
            raw_context,
            ENDPOINT_DATA_IN,
        ).await
    }

    /// Refresh device information
    fn refresh_device_info(&mut self) -> Result<()> {
        let mut buffer = [0u8; 256];

        // Try to get compile info (firmware version)
        let firmware_version = match self.control_transfer_in(
            CMD_GET_COMPILE_INFO,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        ) {
            Ok(len) if len > 0 => {
                String::from_utf8_lossy(&buffer[..len]).trim().to_string()
            }
            _ => {
                debug!("Failed to get compile info, using default");
                "unknown".to_string()
            }
        };

        // Get board ID
        let board_id = match self.control_transfer_in(
            CMD_GET_BOARD_ID,
            0,
            0,
            &mut buffer,
            USB_TIMEOUT_SHORT_MS,
        ) {
            Ok(len) if len > 0 => buffer[0],
            _ => 0xFF,
        };

        // Get serial number
        let serial_number = match self.control_transfer_in(
            CMD_GET_SERIAL,
            0,
            0,
            &mut buffer[..17],
            USB_TIMEOUT_SHORT_MS,
        ) {
            Ok(len) if len >= 17 => {
                // Parse serial number from 17 bytes
                let mut serial = String::new();
                for i in 1..17 {
                    serial.push_str(&format!("{:02x}", buffer[i]));
                }
                serial
            }
            _ => "unknown".to_string(),
        };

        self.device_info = Some(DeviceInfo {
            firmware_version: firmware_version.clone(),
            api_version: "1.07".to_string(), // Current Ubertooth API version
            board_id,
            serial_number,
            compile_info: firmware_version.clone(),
        });

        info!("Device: {} ({})",
            self.device_info.as_ref().unwrap().board_name(),
            firmware_version
        );

        Ok(())
    }
}

impl Drop for UbertoothDeviceLibusb {
    fn drop(&mut self) {
        let _ = self.disconnect();

        unsafe {
            if !self.context.is_null() {
                libusb_exit(self.context);
            }
        }
    }
}

// Safety: The libusb context and handle are thread-safe when properly synchronized
unsafe impl Send for UbertoothDeviceLibusb {}
unsafe impl Sync for UbertoothDeviceLibusb {}
