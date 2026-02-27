//! Native Rust USB implementation for Ubertooth One (Phase 3).
//!
//! This crate provides direct libusb access to Ubertooth One devices for
//! high-performance operations, achieving 100-200x speedup over Python backend.
//!
//! ## Architecture
//!
//! - `device`: UbertoothDevice struct with connection management
//! - `commands`: High-level USB command implementations
//! - `protocol`: USB packet structures and parsing
//! - `error`: USB-specific error types
//! - `constants`: USB IDs, endpoints, command opcodes
//!
//! ## Usage
//!
//! ```no_run
//! use ubertooth_usb::{UbertoothDevice, UbertoothCommands};
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and connect to device
//!     let mut device = UbertoothDevice::new()?;
//!     device.connect(0)?;
//!
//!     // Create command executor
//!     let device = Arc::new(Mutex::new(device));
//!     let commands = UbertoothCommands::new(device);
//!
//!     // Execute commands
//!     let result = commands.device_status(serde_json::json!({})).await?;
//!     println!("{}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod constants;
pub mod device;
pub mod error;
pub mod protocol;
pub mod commands;

// Re-exports for convenience
pub use constants::*;
pub use device::UbertoothDevice;
pub use error::{Result, UsbError};
pub use protocol::{BlePacket, DeviceInfo, UsbPacket};
pub use commands::UbertoothCommands;
