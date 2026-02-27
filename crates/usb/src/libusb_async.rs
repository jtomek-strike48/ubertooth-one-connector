//! Direct libusb-1.0 FFI for async USB transfers.
//!
//! This module provides async USB transfers using libusb-1.0's proven async API.
//! We use FFI here because:
//! 1. Python ubertooth-btle works (proves libusb async works)
//! 2. rusb doesn't support async transfers
//! 3. nusb has protocol incompatibility with Ubertooth firmware
//!
//! Safety: All unsafe code is isolated to this module with safe wrappers.

use crate::constants::*;
use crate::error::{Result, UsbError};
use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

/// Send-safe wrapper for raw device handle pointer.
struct SendableHandle(*mut c_void);
unsafe impl Send for SendableHandle {}
unsafe impl Sync for SendableHandle {}

// Link to libusb-1.0
#[link(name = "usb-1.0")]
extern "C" {
    fn libusb_alloc_transfer(iso_packets: i32) -> *mut LibusbTransfer;
    fn libusb_free_transfer(transfer: *mut LibusbTransfer);
    fn libusb_submit_transfer(transfer: *mut LibusbTransfer) -> i32;
    fn libusb_cancel_transfer(transfer: *mut LibusbTransfer) -> i32;
    fn libusb_handle_events_timeout_completed(
        ctx: *mut c_void,
        tv: *const TimeVal,
        completed: *mut i32,
    ) -> i32;
}

// libusb structures
#[repr(C)]
struct TimeVal {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct LibusbTransfer {
    dev_handle: *mut c_void,
    flags: u8,
    endpoint: u8,
    transfer_type: u8,
    timeout: u32,
    status: i32,
    length: i32,
    actual_length: i32,
    callback: Option<extern "C" fn(*mut LibusbTransfer)>,
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

// Transfer type constants
const LIBUSB_TRANSFER_TYPE_BULK: u8 = 2;

/// Completion state for a transfer.
struct TransferCompletion {
    status: i32,
    actual_length: usize,
    data: Vec<u8>,
}

/// Async USB transfer using libusb-1.0.
pub struct AsyncTransfer {
    transfer: *mut LibusbTransfer,
    buffer: Vec<u8>,
    completion: Arc<Mutex<Option<TransferCompletion>>>,
}

unsafe impl Send for AsyncTransfer {}
unsafe impl Sync for AsyncTransfer {}

impl AsyncTransfer {
    /// Create a new async bulk transfer.
    ///
    /// # Safety
    /// The dev_handle must be a valid libusb device handle pointer from rusb.
    pub unsafe fn new_bulk_in(
        dev_handle: *mut c_void,
        endpoint: u8,
        buffer_size: usize,
        timeout_ms: u32,
    ) -> Result<Self> {
        // Allocate transfer
        let transfer = libusb_alloc_transfer(0);
        if transfer.is_null() {
            return Err(UsbError::Other("Failed to allocate transfer".to_string()));
        }

        // Allocate buffer
        let buffer = vec![0u8; buffer_size];

        // Create completion tracker
        let completion = Arc::new(Mutex::new(None));
        let completion_ptr = Arc::into_raw(Arc::clone(&completion)) as *mut c_void;

        // Fill transfer structure
        (*transfer).dev_handle = dev_handle;
        (*transfer).endpoint = endpoint;
        (*transfer).transfer_type = LIBUSB_TRANSFER_TYPE_BULK;
        (*transfer).timeout = timeout_ms;
        (*transfer).buffer = buffer.as_ptr() as *mut u8;
        (*transfer).length = buffer_size as i32;
        (*transfer).callback = Some(transfer_callback);
        (*transfer).user_data = completion_ptr;

        debug!("Created async transfer: endpoint=0x{:02x}, size={}, timeout={}ms",
               endpoint, buffer_size, timeout_ms);

        Ok(Self {
            transfer,
            buffer,
            completion,
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

        trace!("Transfer submitted");
        Ok(())
    }

    /// Check if transfer has completed (non-blocking).
    pub fn is_complete(&self) -> bool {
        let completion = self.completion.lock().unwrap();
        completion.is_some()
    }

    /// Take the completion result if ready.
    pub fn try_take_completion(&mut self) -> Option<TransferCompletion> {
        let mut completion = self.completion.lock().unwrap();
        completion.take()
    }

    /// Cancel the transfer.
    pub fn cancel(&mut self) {
        unsafe {
            let ret = libusb_cancel_transfer(self.transfer);
            if ret != 0 && ret != -5 {  // -5 = NOT_FOUND (already completed)
                warn!("Failed to cancel transfer: error {}", ret);
            }
        }
    }

    /// Get the transfer buffer.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Reset for resubmission with new buffer.
    pub fn reset_buffer(&mut self, new_buffer: Vec<u8>) {
        self.buffer = new_buffer;
        unsafe {
            (*self.transfer).buffer = self.buffer.as_ptr() as *mut u8;
            (*self.transfer).length = self.buffer.len() as i32;
        }
    }
}

impl Drop for AsyncTransfer {
    fn drop(&mut self) {
        unsafe {
            if !self.transfer.is_null() {
                // Cancel if still active
                let _ = libusb_cancel_transfer(self.transfer);

                // Free user_data Arc
                if !(*self.transfer).user_data.is_null() {
                    let _ = Arc::from_raw(
                        (*self.transfer).user_data as *const Mutex<Option<TransferCompletion>>
                    );
                }

                // Free transfer
                libusb_free_transfer(self.transfer);
                self.transfer = ptr::null_mut();
            }
        }
    }
}

/// Callback function called by libusb when transfer completes.
extern "C" fn transfer_callback(transfer: *mut LibusbTransfer) {
    eprintln!("[libusb_async] transfer_callback called!");

    unsafe {
        if transfer.is_null() {
            eprintln!("[libusb_async] transfer_callback: transfer is NULL");
            return;
        }

        let t = &*transfer;
        eprintln!("[libusb_async] transfer_callback: status={}, actual_length={}", t.status, t.actual_length);

        let completion_ptr = t.user_data as *const Mutex<Option<TransferCompletion>>;
        if completion_ptr.is_null() {
            eprintln!("[libusb_async] transfer_callback: user_data is NULL");
            return;
        }

        // Reconstruct Arc temporarily (don't drop it - still owned by AsyncTransfer)
        let completion_arc = Arc::from_raw(completion_ptr);

        // Copy buffer data before it potentially gets reused
        let data = if t.actual_length > 0 {
            std::slice::from_raw_parts(t.buffer, t.actual_length as usize).to_vec()
        } else {
            Vec::new()
        };

        let result = TransferCompletion {
            status: t.status,
            actual_length: t.actual_length as usize,
            data,
        };

        eprintln!("[libusb_async] transfer_callback: storing completion");
        trace!(
            "Transfer callback: status={}, length={}",
            result.status,
            result.actual_length
        );

        {
            let mut guard = completion_arc.lock().unwrap();
            *guard = Some(result);
        }

        // Don't drop the Arc - it's still owned by AsyncTransfer
        let _ = Arc::into_raw(completion_arc);
    }
}

/// Manager for handling multiple async transfers with event loop.
pub struct AsyncTransferManager {
    context: *mut c_void,
    packet_tx: mpsc::Sender<Vec<u8>>,
    running: Arc<Mutex<bool>>,
}

unsafe impl Send for AsyncTransferManager {}
unsafe impl Sync for AsyncTransferManager {}

impl AsyncTransferManager {
    /// Create a new transfer manager.
    ///
    /// # Safety
    /// The context must be a valid libusb context pointer from rusb.
    pub unsafe fn new(
        context: *mut c_void,
        packet_tx: mpsc::Sender<Vec<u8>>,
    ) -> Self {
        Self {
            context,
            packet_tx,
            running: Arc::new(Mutex::new(true)),
        }
    }

    /// Run the event loop (blocking).
    ///
    /// This should be called in a background thread.
    pub fn run_event_loop(&self) -> Result<()> {
        eprintln!("[libusb_async] run_event_loop starting");
        debug!("Starting libusb event loop");

        let mut completed = 0;
        let timeout = TimeVal {
            tv_sec: 0,
            tv_usec: 100_000,  // 100ms
        };

        let mut loop_count = 0;
        while *self.running.lock().unwrap() {
            loop_count += 1;
            if loop_count == 1 || loop_count % 100 == 0 {
                eprintln!("[libusb_async] Event loop iteration {}", loop_count);
            }

            unsafe {
                let ret = libusb_handle_events_timeout_completed(
                    self.context,
                    &timeout,
                    &mut completed,
                );

                if loop_count % 100 == 0 {
                    eprintln!("[libusb_async] libusb_handle_events returned: {}, completed: {}", ret, completed);
                }

                if ret < 0 {
                    eprintln!("[libusb_async] libusb_handle_events error: {}", ret);
                    warn!("libusb_handle_events error: {}", ret);
                    if ret == -1 {  // LIBUSB_ERROR_IO
                        return Err(UsbError::Disconnected);
                    }
                }
            }

            // Small yield to prevent busy wait
            std::thread::yield_now();
        }

        eprintln!("[libusb_async] Event loop stopping");
        debug!("Event loop stopped");
        Ok(())
    }

    /// Stop the event loop.
    pub fn stop(&self) {
        debug!("Stopping event loop");
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Send a packet to the channel.
    pub async fn send_packet(&self, data: Vec<u8>) -> Result<()> {
        self.packet_tx
            .send(data)
            .await
            .map_err(|_| UsbError::Other("Channel closed".to_string()))
    }
}

/// High-level streaming reader using libusb async transfers.
pub struct LibusbStreamReader {
    packet_rx: mpsc::Receiver<Vec<u8>>,
    packet_count: usize,
}

impl LibusbStreamReader {
    /// Start streaming from raw handle and context pointers.
    ///
    /// This spawns background tasks for event handling and transfer management.
    pub async fn start_from_raw(
        raw_handle: *mut c_void,
        raw_context: *mut c_void,
        endpoint: u8,
    ) -> Result<Self> {
        debug!("Starting libusb streaming reader from raw pointers");

        // Wrap for Send
        let sendable_handle = SendableHandle(raw_handle);
        let sendable_context = SendableHandle(raw_context);

        let (packet_tx, packet_rx) = mpsc::channel(100);

        // Spawn background streaming task
        tokio::task::spawn_blocking(move || {
            if let Err(e) = run_streaming_raw(sendable_handle, sendable_context, endpoint, packet_tx) {
                warn!("Streaming error: {}", e);
            }
        });

        Ok(Self {
            packet_rx,
            packet_count: 0,
        })
    }

    /// Start streaming from a rusb device handle (legacy).
    ///
    /// This spawns background tasks for event handling and transfer management.
    pub async fn start(
        device_handle: Arc<Mutex<rusb::DeviceHandle<rusb::Context>>>,
        endpoint: u8,
    ) -> Result<Self> {
        debug!("Starting libusb streaming reader");

        let (packet_tx, packet_rx) = mpsc::channel(100);

        // Spawn background streaming task
        tokio::task::spawn_blocking(move || {
            if let Err(e) = run_streaming(device_handle, endpoint, packet_tx) {
                warn!("Streaming error: {}", e);
            }
        });

        Ok(Self {
            packet_rx,
            packet_count: 0,
        })
    }

    /// Read the next packet (async).
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

    /// Get total packets received.
    pub fn packet_count(&self) -> usize {
        self.packet_count
    }
}

/// Background function that manages streaming with raw handle pointer.
fn run_streaming_raw(
    raw_handle: SendableHandle,
    raw_context: SendableHandle,
    endpoint: u8,
    packet_tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    let raw_handle = raw_handle.0;  // Extract the pointer
    let raw_context = raw_context.0;  // Extract the context pointer
    const NUM_TRANSFERS: usize = 8;
    const TRANSFER_SIZE: usize = USB_PKT_SIZE;
    const TIMEOUT_MS: u32 = 5000;

    eprintln!("[libusb_async] Starting run_streaming_raw");
    eprintln!("[libusb_async] raw_handle: {:?}, raw_context: {:?}, endpoint: 0x{:02x}",
              raw_handle, raw_context, endpoint);
    debug!("Setting up {} concurrent transfers with raw handle", NUM_TRANSFERS);

    // Create transfer manager
    let manager = unsafe { AsyncTransferManager::new(raw_context, packet_tx.clone()) };

    // Create and submit initial transfers
    eprintln!("[libusb_async] Creating {} transfers...", NUM_TRANSFERS);
    let mut transfers = Vec::new();
    for i in 0..NUM_TRANSFERS {
        let mut transfer = unsafe {
            AsyncTransfer::new_bulk_in(raw_handle, endpoint, TRANSFER_SIZE, TIMEOUT_MS)?
        };

        transfer.submit()?;
        eprintln!("[libusb_async] Submitted transfer {}/{}", i + 1, NUM_TRANSFERS);
        debug!("Submitted transfer {}/{}", i + 1, NUM_TRANSFERS);
        transfers.push(transfer);
    }
    eprintln!("[libusb_async] All transfers submitted");

    // Spawn event loop in background thread
    eprintln!("[libusb_async] About to spawn event loop thread");
    let manager_clone = Arc::new(manager);
    let manager_for_loop = Arc::clone(&manager_clone);

    eprintln!("[libusb_async] Spawning event loop thread");
    std::thread::spawn(move || {
        eprintln!("[libusb_async] Event loop thread started");
        if let Err(e) = manager_for_loop.run_event_loop() {
            eprintln!("[libusb_async] Event loop error: {}", e);
            warn!("Event loop error: {}", e);
        }
        eprintln!("[libusb_async] Event loop thread exiting");
    });

    eprintln!("[libusb_async] Event loop thread spawned, starting main loop");

    // Main transfer management loop
    eprintln!("[libusb_async] Entering main transfer management loop");
    let mut packet_count = 0;
    let mut loop_count = 0;
    loop {
        loop_count += 1;
        if loop_count == 1 || loop_count % 1000 == 0 {
            eprintln!("[libusb_async] Loop iteration {}, packets: {}", loop_count, packet_count);
        }

        // Check all transfers for completion
        let mut any_completed = false;

        for transfer in &mut transfers {
            if let Some(completion) = transfer.try_take_completion() {
                any_completed = true;

                match completion.status {
                    LIBUSB_TRANSFER_COMPLETED => {
                        if completion.actual_length > 0 {
                            packet_count += 1;
                            debug!("Packet #{}: {} bytes", packet_count, completion.actual_length);

                            // Send packet
                            if let Err(e) = packet_tx.blocking_send(completion.data) {
                                warn!("Failed to send packet: {}", e);
                                manager_clone.stop();
                                return Ok(());
                            }
                        }
                    }
                    LIBUSB_TRANSFER_CANCELLED => {
                        debug!("Transfer cancelled");
                        manager_clone.stop();
                        return Ok(());
                    }
                    LIBUSB_TRANSFER_TIMED_OUT => {
                        trace!("Transfer timed out (normal for no data)");
                    }
                    status => {
                        warn!("Transfer error: status {}", status);
                    }
                }

                // Resubmit transfer
                if let Err(e) = transfer.submit() {
                    warn!("Failed to resubmit transfer: {}", e);
                    manager_clone.stop();
                    return Err(e);
                }
            }
        }

        if !any_completed {
            // Sleep briefly if no completions to avoid busy wait
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Background function that manages the streaming transfers.
fn run_streaming(
    device_handle: Arc<Mutex<rusb::DeviceHandle<rusb::Context>>>,
    endpoint: u8,
    packet_tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    const NUM_TRANSFERS: usize = 8;
    const TRANSFER_SIZE: usize = USB_PKT_SIZE;
    const TIMEOUT_MS: u32 = 5000;

    debug!("Setting up {} concurrent transfers", NUM_TRANSFERS);

    // Get raw pointers (this is safe because we hold Arc<Mutex<>>)
    let handle_guard = device_handle.lock().unwrap();
    let raw_handle = handle_guard.as_raw() as *mut c_void;

    // Get context from the device
    // Note: rusb doesn't expose context directly, but we can use NULL
    // and libusb will use the default context
    let raw_context = ptr::null_mut();

    drop(handle_guard);  // Release lock

    // Create transfer manager
    let manager = unsafe { AsyncTransferManager::new(raw_context, packet_tx.clone()) };

    // Create and submit initial transfers
    eprintln!("[libusb_async] Creating {} transfers...", NUM_TRANSFERS);
    let mut transfers = Vec::new();
    for i in 0..NUM_TRANSFERS {
        let mut transfer = unsafe {
            AsyncTransfer::new_bulk_in(raw_handle, endpoint, TRANSFER_SIZE, TIMEOUT_MS)?
        };

        transfer.submit()?;
        eprintln!("[libusb_async] Submitted transfer {}/{}", i + 1, NUM_TRANSFERS);
        debug!("Submitted transfer {}/{}", i + 1, NUM_TRANSFERS);
        transfers.push(transfer);
    }
    eprintln!("[libusb_async] All transfers submitted");

    // Spawn event loop in background thread
    eprintln!("[libusb_async] About to spawn event loop thread");
    let manager_clone = Arc::new(manager);
    let manager_for_loop = Arc::clone(&manager_clone);

    eprintln!("[libusb_async] Spawning event loop thread");
    std::thread::spawn(move || {
        eprintln!("[libusb_async] Event loop thread started");
        if let Err(e) = manager_for_loop.run_event_loop() {
            eprintln!("[libusb_async] Event loop error: {}", e);
            warn!("Event loop error: {}", e);
        }
        eprintln!("[libusb_async] Event loop thread exiting");
    });

    eprintln!("[libusb_async] Event loop thread spawned, starting main loop");

    // Main transfer management loop
    eprintln!("[libusb_async] Entering main transfer management loop");
    let mut packet_count = 0;
    let mut loop_count = 0;
    loop {
        loop_count += 1;
        if loop_count == 1 || loop_count % 1000 == 0 {
            eprintln!("[libusb_async] Loop iteration {}, packets: {}", loop_count, packet_count);
        }

        // Check all transfers for completion
        let mut any_completed = false;

        for transfer in &mut transfers {
            if let Some(completion) = transfer.try_take_completion() {
                any_completed = true;

                match completion.status {
                    LIBUSB_TRANSFER_COMPLETED => {
                        if completion.actual_length > 0 {
                            packet_count += 1;
                            debug!("Packet #{}: {} bytes", packet_count, completion.actual_length);

                            // Send packet
                            if let Err(e) = packet_tx.blocking_send(completion.data) {
                                warn!("Failed to send packet: {}", e);
                                manager_clone.stop();
                                return Ok(());
                            }
                        }
                    }
                    LIBUSB_TRANSFER_CANCELLED => {
                        debug!("Transfer cancelled");
                        manager_clone.stop();
                        return Ok(());
                    }
                    LIBUSB_TRANSFER_TIMED_OUT => {
                        trace!("Transfer timed out (normal for no data)");
                    }
                    status => {
                        warn!("Transfer error: status {}", status);
                    }
                }

                // Resubmit transfer
                if let Err(e) = transfer.submit() {
                    warn!("Failed to resubmit transfer: {}", e);
                    manager_clone.stop();
                    return Err(e);
                }
            }
        }

        if !any_completed {
            // Sleep briefly if no completions to avoid busy wait
            std::thread::sleep(Duration::from_millis(1));
        }
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
