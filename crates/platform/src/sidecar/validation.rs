//! Validation utilities for sidecar operations.

use std::process::Command;
use ubertooth_core::error::{Result, UbertoothError};

/// Check if ubertooth-tools are installed on the system.
pub fn check_ubertooth_installed() -> Result<()> {
    // Check for ubertooth-util (core utility)
    let output = Command::new("which")
        .arg("ubertooth-util")
        .output()
        .map_err(|e| {
            UbertoothError::BackendError(format!("Failed to check for ubertooth-util: {}", e))
        })?;

    if !output.status.success() {
        return Err(UbertoothError::BackendError(
            "ubertooth-tools not found. Please install:\n\
             Ubuntu/Debian: sudo apt-get install ubertooth\n\
             Arch: sudo pacman -S ubertooth\n\
             macOS: brew install ubertooth\n\
             From source: https://github.com/greatscottgadgets/ubertooth"
                .to_string(),
        ));
    }

    Ok(())
}
