//! True asynchronous USB transfers using libusb-1.0 API.
//!
//! This module implements asynchronous USB bulk transfers using libusb's
//! transfer API, which is required for Ubertooth firmware packet capture.

use crate::error::{Result, UsbError};
use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, trace, warn};

// Link to libusb-1.0
#[link(name = "usb-1.0")]
extern "C" {
    fn libusb_alloc_transfer(iso_packets: i32) -> *mut libusb_transfer;
    fn libusb_free_transfer(transfer: *mut libusb_transfer);
    fn libusb_submit_transfer(transfer: *mut libusb_transfer) -> i32;
    fn libusb_cancel_transfer(transfer: *mut libusb_transfer) -> i32;
    fn libusb_handle_events_timeout(
        ctx: *mut c_void,
        tv: *const timeval,
    ) -> i32;
}

#[repr(C)]
struct timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct libusb_transfer {
    dev_handle: *mut c_void,
    flags: u8,
    endpoint: u8,
    transfer_type: u8,
    timeout: u32,
    status: i32,
    length: i32,
    actual_length: i32,
    callback: Option<extern "C" fn(*mut libusb_transfer)>,
    user_data: *mut c_void,
    buffer: *mut u8,
    num_iso_packets: i32,
}

// Transfer status constants
const LIBUSB_TRANSFER_COMPLETED: i32 = 0;
const LIBUSB_TRANSFER_ERROR: i32 = 1;
const LIBUSB_TRANSFER_TIMED_OUT: i32 = 2;
const LIBUSB_TRANSFER_CANCELLED: i32 = 3;
const LIBUSB_TRANSFER_STALL: i32 = 4;
const LIBUSB_TRANSFER_NO_DEVICE: i32 = 5;
const LIBUSB_TRANSFER_OVERFLOW: i32 = 6;

/// Asynchronous bulk transfer wrapper.
pub struct AsyncBulkTransfer {
    transfer: *mut libusb_transfer,
    buffer: Vec<u8>,
    completed: Arc<Mutex<Option<TransferResult>>>,
}

#[derive(Debug, Clone)]
struct TransferResult {
    status: i32,
    actual_length: usize,
}

impl AsyncBulkTransfer {
    /// Create a new async bulk transfer.
    ///
    /// # Arguments
    /// * `dev_handle` - Raw libusb device handle pointer
    /// * `endpoint` - USB endpoint address
    /// * `buffer_size` - Size of transfer buffer
    /// * `timeout_ms` - Transfer timeout in milliseconds
    ///
    /// # Safety
    /// The dev_handle must be a valid libusb device handle pointer.
    pub unsafe fn new(
        dev_handle: *mut c_void,
        endpoint: u8,
        buffer_size: usize,
        timeout_ms: u32,
    ) -> Result<Self> {
        // Allocate transfer struct
        let transfer = libusb_alloc_transfer(0);
        if transfer.is_null() {
            return Err(UsbError::Other("Failed to allocate transfer".to_string()));
        }

        // Allocate buffer
        let buffer = vec![0u8; buffer_size];

        // Create completion tracker
        let completed = Arc::new(Mutex::new(None));

        // Set up transfer
        let completed_ptr = Arc::into_raw(completed.clone()) as *mut c_void;

        (*transfer).dev_handle = dev_handle;
        (*transfer).endpoint = endpoint;
        (*transfer).transfer_type = 2; // LIBUSB_TRANSFER_TYPE_BULK
        (*transfer).timeout = timeout_ms;
        (*transfer).buffer = buffer.as_ptr() as *mut u8;
        (*transfer).length = buffer_size as i32;
        (*transfer).callback = Some(transfer_callback);
        (*transfer).user_data = completed_ptr;

        Ok(Self {
            transfer,
            buffer,
            completed,
        })
    }

    /// Submit the transfer for execution.
    pub fn submit(&mut self) -> Result<()> {
        unsafe {
            let ret = libusb_submit_transfer(self.transfer);
            if ret != 0 {
                return Err(UsbError::Other(format!(
                    "Failed to submit transfer: error {}",
                    ret
                )));
            }
        }

        debug!("Transfer submitted");
        Ok(())
    }

    /// Check if transfer has completed (non-blocking).
    pub fn is_complete(&self) -> bool {
        let completed = self.completed.lock().unwrap();
        completed.is_some()
    }

    /// Wait for transfer to complete and get result.
    ///
    /// Returns the number of bytes transferred on success.
    pub fn wait(&self, timeout: Duration) -> Result<usize> {
        let deadline = std::time::Instant::now() + timeout;

        loop {
            // Check if already completed
            {
                let completed = self.completed.lock().unwrap();
                if let Some(result) = completed.as_ref() {
                    return match result.status {
                        LIBUSB_TRANSFER_COMPLETED => Ok(result.actual_length),
                        LIBUSB_TRANSFER_TIMED_OUT => {
                            Err(UsbError::Timeout {
                                timeout_ms: timeout.as_millis() as u64,
                            })
                        }
                        LIBUSB_TRANSFER_CANCELLED => {
                            Err(UsbError::Other("Transfer cancelled".to_string()))
                        }
                        LIBUSB_TRANSFER_NO_DEVICE => Err(UsbError::Disconnected),
                        _ => Err(UsbError::Other(format!(
                            "Transfer failed with status {}",
                            result.status
                        ))),
                    };
                }
            }

            // Check timeout
            if std::time::Instant::now() >= deadline {
                return Err(UsbError::Timeout {
                    timeout_ms: timeout.as_millis() as u64,
                });
            }

            // Small sleep to avoid busy wait
            std::thread::sleep(Duration::from_micros(100));
        }
    }

    /// Get the transfer buffer.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Cancel the transfer.
    pub fn cancel(&mut self) -> Result<()> {
        unsafe {
            let ret = libusb_cancel_transfer(self.transfer);
            if ret != 0 {
                warn!("Failed to cancel transfer: error {}", ret);
            }
        }
        Ok(())
    }
}

impl Drop for AsyncBulkTransfer {
    fn drop(&mut self) {
        unsafe {
            if !self.transfer.is_null() {
                // Try to cancel if still active
                let _ = libusb_cancel_transfer(self.transfer);

                // Free user_data Arc
                if !(*self.transfer).user_data.is_null() {
                    let _ = Arc::from_raw(
                        (*self.transfer).user_data as *const Mutex<Option<TransferResult>>
                    );
                }

                // Free transfer
                libusb_free_transfer(self.transfer);
            }
        }
    }
}

/// Callback function called when transfer completes.
extern "C" fn transfer_callback(transfer: *mut libusb_transfer) {
    unsafe {
        if transfer.is_null() {
            return;
        }

        let t = &*transfer;
        let completed_ptr = t.user_data as *const Mutex<Option<TransferResult>>;
        if completed_ptr.is_null() {
            return;
        }

        let completed = Arc::from_raw(completed_ptr);
        let result = TransferResult {
            status: t.status,
            actual_length: t.actual_length as usize,
        };

        trace!(
            "Transfer completed: status={}, length={}",
            result.status,
            result.actual_length
        );

        {
            let mut guard = completed.lock().unwrap();
            *guard = Some(result);
        }

        // Don't drop the Arc - it's still owned by the AsyncBulkTransfer
        let _ = Arc::into_raw(completed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_compiles() {
        // Just verify the module compiles
        // Real testing requires USB hardware
    }
}
