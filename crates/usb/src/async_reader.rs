//! Asynchronous bulk data reader for streaming packet capture.
//!
//! The Ubertooth firmware expects asynchronous USB transfers (URB submission/reaping pattern).
//! This module implements a non-blocking polling reader that mimics the async behavior.

use crate::constants::*;
use crate::error::{Result, UsbError};
use crate::device::UbertoothDevice;
use crate::device_libusb::UbertoothDeviceLibusb;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{debug, trace, warn};

/// Asynchronous packet reader that polls the USB bulk endpoint.
///
/// Uses non-blocking reads with short timeouts to simulate async URB behavior.
pub struct AsyncPacketReader {
    /// Device handle
    device: Arc<Mutex<UbertoothDevice>>,

    /// Buffer size for each read
    buffer_size: usize,

    /// Polling interval (milliseconds)
    poll_interval_ms: u64,

    /// Read timeout (milliseconds) - should be very short
    read_timeout_ms: u64,
}

impl AsyncPacketReader {
    /// Create a new async packet reader.
    ///
    /// # Arguments
    /// * `device` - USB device handle
    /// * `buffer_size` - Size of read buffer (typically USB_PKT_SIZE)
    pub fn new(device: Arc<Mutex<UbertoothDevice>>, buffer_size: usize) -> Self {
        Self {
            device,
            buffer_size,
            poll_interval_ms: 1,      // Poll every 1ms
            read_timeout_ms: 10,       // Very short USB timeout
        }
    }

    /// Read a single packet with non-blocking behavior.
    ///
    /// Returns:
    /// - `Ok(Some(Vec<u8>))` - Packet received
    /// - `Ok(None)` - No data available (try again)
    /// - `Err(_)` - Fatal error
    pub async fn try_read_packet(&self) -> Result<Option<Vec<u8>>> {
        let device = self.device.lock().await;

        let mut buffer = vec![0u8; self.buffer_size];

        match device.bulk_read(&mut buffer, self.read_timeout_ms) {
            Ok(len) => {
                trace!("Read {} bytes from bulk endpoint", len);
                buffer.truncate(len);
                Ok(Some(buffer))
            }
            Err(UsbError::Timeout { .. }) => {
                // Timeout is expected when no data available - not an error
                trace!("No data available (timeout)");
                Ok(None)
            }
            Err(e) => {
                warn!("Bulk read error: {}", e);
                Err(e)
            }
        }
    }

    /// Read packets continuously, calling the callback for each packet.
    ///
    /// Runs until:
    /// - Duration expires
    /// - Callback returns false
    /// - Fatal error occurs
    pub async fn read_packets<F>(
        &self,
        duration: Duration,
        mut callback: F,
    ) -> Result<usize>
    where
        F: FnMut(Vec<u8>) -> bool,
    {
        let start = tokio::time::Instant::now();
        let mut packet_count = 0;
        let mut consecutive_errors = 0;
        let max_consecutive_errors = 100;

        debug!("Starting async packet capture (duration: {:?})", duration);

        while start.elapsed() < duration {
            match self.try_read_packet().await {
                Ok(Some(packet)) => {
                    consecutive_errors = 0;
                    packet_count += 1;

                    // Call callback - stop if it returns false
                    if !callback(packet) {
                        debug!("Callback requested stop");
                        break;
                    }
                }
                Ok(None) => {
                    // No data - wait a bit before next poll
                    consecutive_errors = 0;
                    sleep(Duration::from_millis(self.poll_interval_ms)).await;
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= max_consecutive_errors {
                        warn!("Too many consecutive errors ({}), stopping", consecutive_errors);
                        return Err(e);
                    }

                    // Brief pause before retrying
                    sleep(Duration::from_millis(10)).await;
                }
            }
        }

        debug!("Packet capture complete: {} packets received", packet_count);
        Ok(packet_count)
    }

    /// Read packets into a channel for async processing.
    pub async fn read_packets_to_channel(
        &self,
        duration: Duration,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<usize> {
        self.read_packets(duration, |packet| {
            // Try to send packet to channel
            match tx.try_send(packet) {
                Ok(_) => true,  // Continue reading
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    warn!("Channel full, dropping packet");
                    true  // Continue anyway
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                    debug!("Channel closed");
                    false  // Stop reading
                }
            }
        }).await
    }

    /// Configure polling parameters for performance tuning.
    pub fn set_polling_params(&mut self, poll_interval_ms: u64, read_timeout_ms: u64) {
        self.poll_interval_ms = poll_interval_ms;
        self.read_timeout_ms = read_timeout_ms;
    }
}

/// Helper function to prepare device for streaming (old rusb-based device).
///
/// Clears any stale data from the USB buffer.
pub async fn flush_usb_buffer(device: Arc<Mutex<UbertoothDevice>>) -> Result<()> {
    debug!("Flushing USB buffer...");

    let dev = device.lock().await;
    let mut buffer = vec![0u8; USB_PKT_SIZE];
    let mut flushed_bytes = 0;

    // Quick non-blocking reads to clear any stale data
    for _ in 0..10 {
        match dev.bulk_read(&mut buffer, 5) {  // 5ms timeout
            Ok(len) => {
                flushed_bytes += len;
                trace!("Flushed {} bytes", len);
            }
            Err(UsbError::Timeout { .. }) => {
                // No more data - good
                break;
            }
            Err(e) => {
                warn!("Error flushing buffer: {}", e);
                break;
            }
        }
    }

    if flushed_bytes > 0 {
        debug!("Flushed {} bytes of stale data", flushed_bytes);
    } else {
        debug!("USB buffer already clean");
    }

    Ok(())
}

/// Helper function to prepare device for streaming (libusb-based device).
///
/// Clears any stale data from the USB buffer.
pub async fn flush_usb_buffer_libusb(device: Arc<Mutex<UbertoothDeviceLibusb>>) -> Result<()> {
    debug!("Flushing USB buffer...");

    let dev = device.lock().await;
    let mut buffer = vec![0u8; USB_PKT_SIZE];
    let mut flushed_bytes = 0;

    // Quick non-blocking reads to clear any stale data
    for _ in 0..10 {
        match dev.bulk_read(&mut buffer, 5) {  // 5ms timeout
            Ok(len) => {
                flushed_bytes += len;
                trace!("Flushed {} bytes", len);
            }
            Err(_) => {
                // Timeout or error - buffer is clear
                break;
            }
        }
    }

    if flushed_bytes > 0 {
        debug!("Flushed {} bytes of stale data", flushed_bytes);
    } else {
        debug!("USB buffer already clean");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reader_creation() {
        // This test just verifies compilation
        // Real testing requires hardware
    }
}
