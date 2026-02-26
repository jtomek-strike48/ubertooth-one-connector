//! Error types for the Ubertooth One connector.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UbertoothError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device already connected")]
    AlreadyConnected,

    #[error("No device connected")]
    NotConnected,

    #[error("USB error: {0}")]
    UsbError(String),

    #[error("Firmware too old: {current}, required: {required}")]
    FirmwareTooOld { current: String, required: String },

    #[error("Permission denied - check udev rules (run: sudo ubertooth-one-connector/scripts/install-udev-rules.sh)")]
    PermissionDenied,

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Capture not found: {0}")]
    CaptureNotFound(String),

    #[error("Authorization required for tool '{tool}' (category: {required})")]
    Unauthorized { tool: String, required: String },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, UbertoothError>;
