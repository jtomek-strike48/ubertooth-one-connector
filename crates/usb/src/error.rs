//! USB-specific error types.

use thiserror::Error;

/// USB operation errors specific to Ubertooth.
#[derive(Debug, Error)]
pub enum UsbError {
    /// USB library error
    #[error("USB error: {0}")]
    UsbLibrary(#[from] rusb::Error),

    /// Device not found
    #[error("Ubertooth device not found (VID:0x{vid:04x} PID:0x{pid:04x})")]
    DeviceNotFound { vid: u16, pid: u16 },

    /// Multiple devices found
    #[error("Multiple Ubertooth devices found - specify device index")]
    MultipleDevices { count: usize },

    /// Device already open
    #[error("Device already connected")]
    AlreadyOpen,

    /// Device not open
    #[error("Device not connected - call device_connect first")]
    NotOpen,

    /// Permission denied (udev rules issue)
    #[error("Permission denied - run: sudo ubertooth-one-connector/scripts/install-udev-rules.sh")]
    PermissionDenied,

    /// USB timeout
    #[error("USB operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Control transfer failed
    #[error("Control transfer failed (cmd={cmd:02x}): {details}")]
    ControlTransferFailed { cmd: u8, details: String },

    /// Bulk transfer failed
    #[error("Bulk transfer failed (endpoint={endpoint:02x}): {details}")]
    BulkTransferFailed { endpoint: u8, details: String },

    /// Invalid packet
    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    /// Firmware version too old
    #[error("Firmware too old: {current}, required: {required}")]
    FirmwareTooOld { current: String, required: String },

    /// Unsupported board
    #[error("Unsupported board ID: {0}")]
    UnsupportedBoard(u8),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Device disconnected during operation
    #[error("Device disconnected")]
    Disconnected,

    /// Streaming error
    #[error("Streaming error: {0}")]
    StreamingError(String),

    /// PCAP generation error
    #[error("PCAP generation error: {0}")]
    PcapError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convert USB errors to core Ubertooth errors.
impl From<UsbError> for ubertooth_core::error::UbertoothError {
    fn from(err: UsbError) -> Self {
        match err {
            UsbError::DeviceNotFound { .. } => {
                ubertooth_core::error::UbertoothError::DeviceNotFound
            }
            UsbError::AlreadyOpen => ubertooth_core::error::UbertoothError::AlreadyConnected,
            UsbError::NotOpen => ubertooth_core::error::UbertoothError::NotConnected,
            UsbError::PermissionDenied => ubertooth_core::error::UbertoothError::PermissionDenied,
            UsbError::FirmwareTooOld { current, required } => {
                ubertooth_core::error::UbertoothError::FirmwareTooOld { current, required }
            }
            UsbError::InvalidParameter(msg) => {
                ubertooth_core::error::UbertoothError::InvalidParameter(msg)
            }
            UsbError::Io(e) => ubertooth_core::error::UbertoothError::Io(e),
            other => ubertooth_core::error::UbertoothError::UsbError(other.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, UsbError>;
