//! Streaming packet reader using nusb's optimized multi-transfer approach.
//!
//! This implements the recommended nusb pattern for high-throughput streaming:
//! - Keep multiple transfers pending simultaneously
//! - Resubmit each transfer as it completes
//! - Process packets in a callback

use crate::constants::*;
use crate::error::{Result, UsbError};
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

const NUM_CONCURRENT_TRANSFERS: usize = 8;
const TRANSFER_SIZE: usize = USB_PKT_SIZE;

/// Streaming packet reader using nusb multi-transfer pattern.
pub struct StreamingPacketReader {
    /// Packet receiver channel
    rx: mpsc::Receiver<Vec<u8>>,

    /// Number of packets received
    packet_count: usize,
}

impl StreamingPacketReader {
    /// Start streaming from a nusb endpoint.
    ///
    /// Returns a reader that will receive packets as they arrive.
    pub fn start(interface: nusb::Interface) -> Result<Self> {
        debug!("StreamingPacketReader::start called");

        let (tx, rx) = mpsc::channel(100);

        // Spawn background task to handle streaming
        tokio::spawn(async move {
            debug!("Background streaming task started");
            match stream_packets(interface, tx).await {
                Ok(_) => debug!("Streaming completed successfully"),
                Err(e) => warn!("Streaming error: {}", e),
            }
        });

        debug!("StreamingPacketReader created");

        Ok(Self {
            rx,
            packet_count: 0,
        })
    }

    /// Read the next packet (async).
    ///
    /// Returns `None` when stream ends.
    pub async fn read_packet(&mut self) -> Option<Vec<u8>> {
        match self.rx.recv().await {
            Some(packet) => {
                self.packet_count += 1;
                trace!("Packet #{}: {} bytes", self.packet_count, packet.len());
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

/// Background task that handles the streaming.
async fn stream_packets(
    interface: nusb::Interface,
    tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    debug!("Starting packet stream");

    // Open the bulk IN endpoint
    let mut endpoint = interface
        .endpoint::<nusb::transfer::Bulk, nusb::transfer::In>(ENDPOINT_DATA_IN)
        .map_err(UsbError::from_nusb)?;

    debug!("Endpoint opened, submitting {} initial transfers", NUM_CONCURRENT_TRANSFERS);

    // Submit initial transfers
    for i in 0..NUM_CONCURRENT_TRANSFERS {
        let buffer = nusb::transfer::Buffer::new(TRANSFER_SIZE);
        endpoint.submit(buffer);
        trace!("Submitted transfer #{}", i + 1);
    }

    debug!("Entering streaming loop");

    let mut loop_count = 0;

    // Streaming loop: as transfers complete, process them and resubmit
    loop {
        loop_count += 1;

        // Wait for next completion
        debug!("Waiting for completion #{}", loop_count);
        let completion = endpoint.next_complete().await;

        debug!(
            "Transfer #{} completed: {} bytes, status: {:?}",
            loop_count,
            completion.actual_len,
            completion.status
        );

        // Check for errors
        if let Err(e) = completion.status {
            match e {
                nusb::transfer::TransferError::Cancelled => {
                    debug!("Transfer cancelled");
                    break;
                }
                _ => {
                    warn!("Transfer error: {}", e);
                    // Continue anyway - resubmit
                }
            }
        } else if completion.actual_len > 0 {
            // We got data - send to channel
            let data = completion.buffer[..completion.actual_len].to_vec();

            if tx.send(data).await.is_err() {
                debug!("Channel closed, stopping stream");
                break;
            }
        }

        // Resubmit the transfer
        endpoint.submit(completion.buffer);
    }

    debug!("Packet stream ended");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_compiles() {
        // Just verify compilation
    }
}
