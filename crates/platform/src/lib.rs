//! Platform-specific implementations for the Ubertooth One connector.

pub mod backend;
pub mod capture_store;
pub mod config_store;
pub mod fingerprint;
pub mod session_manager;
pub mod sidecar;
pub mod streaming_buffer;
pub mod system_info;

#[cfg(feature = "rust-backend")]
pub mod rust_usb;

pub use backend::UbertoothBackendProvider;
pub use capture_store::{CaptureCategory, CaptureMetadata, CaptureStore, Tag};
pub use config_store::ConfigStore;
pub use fingerprint::{DeviceFingerprint, DeviceSignature, FingerprintEngine, MatchRule, PacketData as FingerprintPacketData};
pub use session_manager::{PacketFilter, SessionManager, SessionMetadata, SessionState, ViewState};
pub use sidecar::SidecarManager;
pub use streaming_buffer::{BufferStats, CaptureLimits, PacketData, StreamingBuffer};
pub use system_info::SystemInfo;

#[cfg(feature = "rust-backend")]
pub use rust_usb::RustUsbBackend;
