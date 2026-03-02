//! libusb async streaming for packet capture
//!
//! Efficient async USB bulk transfers using pure libusb-1.0 FFI.

use crate::constants::*;
use crate::error::{Result, UsbError};
use crate::libusb_ffi::*;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

/// Send-safe wrapper for raw pointers
struct SendablePtr(*mut c_void);
unsafe impl Send for SendablePtr {}
unsafe impl Sync for SendablePtr {}

/// High-level async streaming reader using pure libusb
pub struct LibusbAsyncReader {
    packet_rx: mpsc::Receiver<Vec<u8>>,
    packet_count: usize,
}

impl LibusbAsyncReader {
    /// Start async streaming from raw libusb handles
    pub async fn start(
        raw_handle: *mut c_void,
        raw_context: *mut c_void,
        endpoint: u8,
    ) -> Result<Self> {
        debug!("Starting libusb async streaming reader");

        let (packet_tx, packet_rx) = mpsc::channel(100);

        // Wrap pointers for Send
        let handle = SendablePtr(raw_handle);
        let context = SendablePtr(raw_context);

        // Spawn background streaming task using std::thread
        // (libusb event loop is truly blocking, doesn't benefit from tokio)
        std::thread::spawn(move || {
            debug!("Background streaming task started");
            match run_streaming_loop(handle, context, endpoint, packet_tx) {
                Ok(_) => debug!("Streaming loop completed successfully"),
                Err(e) => warn!("Streaming error: {}", e),
            }
            debug!("Background streaming task ending");
        });

        Ok(Self {
            packet_rx,
            packet_count: 0,
        })
    }

    /// Read the next packet (async)
    pub async fn read_packet(&mut self) -> Option<Vec<u8>> {
        match self.packet_rx.recv().await {
            Some(packet) => {
                self.packet_count += 1;
                trace!("Received packet #{}: {} bytes", self.packet_count, packet.len());
                Some(packet)
            }
            None => {
                debug!("Stream ended");
                None
            }
        }
    }

    /// Get total packets received
    pub fn packet_count(&self) -> usize {
        self.packet_count
    }
}

/// Transfer context for callback
struct TransferContext {
    buffer: Box<[u8; USB_PKT_SIZE]>,
    packet_tx: mpsc::Sender<Vec<u8>>,
    running: Arc<AtomicBool>,
}

/// Callback function called by libusb when transfer completes
extern "C" fn transfer_callback(transfer: *mut LibusbTransfer) {
    unsafe {
        if transfer.is_null() {
            return;
        }

        let t = &*transfer;

        // Get context
        if t.user_data.is_null() {
            return;
        }

        let ctx = &*(t.user_data as *const TransferContext);

        // Check if still running
        if !ctx.running.load(Ordering::Relaxed) {
            return;
        }

        // Handle completion
        if t.status == LIBUSB_TRANSFER_COMPLETED && t.actual_length > 0 {
            let data = std::slice::from_raw_parts(t.buffer, t.actual_length as usize).to_vec();

            // Try to send packet (non-blocking)
            match ctx.packet_tx.try_send(data) {
                Ok(_) => trace!("Packet sent successfully"),
                Err(e) => {
                    // Channel full or closed, but don't stop streaming
                    warn!("Failed to send packet: {:?}", e);
                }
            }
        } else if t.status == LIBUSB_TRANSFER_CANCELLED {
            debug!("Transfer cancelled");
            return;
        } else if t.status == LIBUSB_TRANSFER_TIMED_OUT {
            trace!("Transfer timed out (no data)");
        } else {
            warn!("Transfer completed with status: {}", t.status);
        }

        // Resubmit transfer
        if ctx.running.load(Ordering::Relaxed) {
            let ret = libusb_submit_transfer(transfer);
            if ret != LIBUSB_SUCCESS {
                warn!("Failed to resubmit transfer: {}", error_name(ret));
                ctx.running.store(false, Ordering::Relaxed);
            }
        }
    }
}

/// Background streaming loop
fn run_streaming_loop(
    handle: SendablePtr,
    context: SendablePtr,
    endpoint: u8,
    packet_tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    const NUM_TRANSFERS: usize = 8;
    const TIMEOUT_MS: u32 = 5000;

    debug!("run_streaming_loop entered");

    let raw_handle = handle.0;
    let raw_context = context.0;

    debug!("Setting up {} concurrent transfers", NUM_TRANSFERS);
    debug!("  raw_handle: {:?}", raw_handle);
    debug!("  raw_context: {:?}", raw_context);
    debug!("  endpoint: 0x{:02X}", endpoint);

    let running = Arc::new(AtomicBool::new(true));
    let mut transfers = Vec::new();
    let mut contexts = Vec::new();

    unsafe {
        // Allocate and submit transfers
        for i in 0..NUM_TRANSFERS {
            let transfer = libusb_alloc_transfer(0);
            if transfer.is_null() {
                return Err(UsbError::Other("Failed to allocate transfer".to_string()));
            }

            // Allocate buffer (leaked, will be cleaned up on exit)
            let buffer = Box::leak(Box::new([0u8; USB_PKT_SIZE]));

            // Create context
            let ctx = Box::leak(Box::new(TransferContext {
                buffer: Box::new([0u8; USB_PKT_SIZE]),
                packet_tx: packet_tx.clone(),
                running: Arc::clone(&running),
            }));

            // Setup transfer
            (*transfer).dev_handle = raw_handle;
            (*transfer).endpoint = endpoint;
            (*transfer).transfer_type = LIBUSB_TRANSFER_TYPE_BULK;
            (*transfer).timeout = TIMEOUT_MS;
            (*transfer).buffer = buffer.as_mut_ptr();
            (*transfer).length = USB_PKT_SIZE as i32;
            (*transfer).callback = Some(transfer_callback);
            (*transfer).user_data = ctx as *mut TransferContext as *mut c_void;

            let ret = libusb_submit_transfer(transfer);
            if ret != LIBUSB_SUCCESS {
                return Err(UsbError::Other(format!(
                    "Failed to submit transfer {}: {}",
                    i,
                    error_name(ret)
                )));
            }

            transfers.push(transfer);
            contexts.push(ctx);
        }

        debug!("All {} transfers submitted", NUM_TRANSFERS);

        // Event loop
        debug!("Starting libusb event loop");
        let mut packet_count = 0;

        while running.load(Ordering::Relaxed) {
            let mut completed = 0;
            let timeout = TimeVal {
                tv_sec: 0,
                tv_usec: 100_000, // 100ms
            };

            let ret = libusb_handle_events_timeout_completed(
                raw_context,
                &timeout,
                &mut completed,
            );

            if ret < 0 {
                warn!("libusb_handle_events error: {}", error_name(ret));
                if ret == LIBUSB_ERROR_NO_DEVICE {
                    running.store(false, Ordering::Relaxed);
                    return Err(UsbError::Disconnected);
                }
            }

            // Check if channel is still open
            if packet_tx.is_closed() {
                debug!("Packet channel closed, stopping event loop");
                running.store(false, Ordering::Relaxed);
                break;
            }
        }

        debug!("Event loop stopped (processed {} packets)", packet_count);

        // Cancel all transfers
        for transfer in &transfers {
            let ret = libusb_cancel_transfer(*transfer);
            if ret != LIBUSB_SUCCESS && ret != LIBUSB_ERROR_NOT_FOUND {
                debug!("Cancel transfer returned: {}", error_name(ret));
            }
        }

        // Process cancellations through event loop
        for _ in 0..10 {
            let mut completed = 0;
            let timeout = TimeVal {
                tv_sec: 0,
                tv_usec: 10_000, // 10ms
            };
            libusb_handle_events_timeout_completed(raw_context, &timeout, &mut completed);
        }

        // Cleanup transfers
        for transfer in transfers {
            libusb_free_transfer(transfer);
        }

        debug!("Streaming cleanup complete");
        Ok(())
    }
}
