//! Capture storage management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use ubertooth_core::error::{Result, UbertoothError};
use uuid::Uuid;

/// Capture metadata structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub capture_id: String,
    pub timestamp: DateTime<Utc>,
    pub capture_type: String,
    pub packet_count: usize,
    pub duration_sec: Option<u64>,
    pub file_size_bytes: u64,
    pub pcap_path: String,
    pub tags: Vec<String>,
    pub description: String,
}

/// Capture storage manager.
pub struct CaptureStore {
    base_path: PathBuf,
}

impl CaptureStore {
    /// Create a new capture store at ~/.ubertooth/
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| UbertoothError::BackendError("Could not determine home directory".to_string()))?;

        let base_path = home.join(".ubertooth");

        // Create directories
        fs::create_dir_all(base_path.join("captures"))?;
        fs::create_dir_all(base_path.join("configs"))?;

        Ok(Self { base_path })
    }

    /// Get the captures directory path.
    pub fn captures_dir(&self) -> PathBuf {
        self.base_path.join("captures")
    }

    /// Get the configs directory path.
    pub fn configs_dir(&self) -> PathBuf {
        self.base_path.join("configs")
    }

    /// Generate a new capture ID.
    pub fn generate_capture_id(prefix: &str) -> String {
        format!("cap-{}-{}", prefix, Uuid::new_v4())
    }

    /// Save capture metadata.
    pub fn save_metadata(&self, metadata: &CaptureMetadata) -> Result<()> {
        let path = self
            .captures_dir()
            .join(format!("{}.json", metadata.capture_id));

        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Load capture metadata.
    pub fn load_metadata(&self, capture_id: &str) -> Result<CaptureMetadata> {
        let path = self.captures_dir().join(format!("{}.json", capture_id));

        if !path.exists() {
            return Err(UbertoothError::CaptureNotFound(capture_id.to_string()));
        }

        let json = fs::read_to_string(path)?;
        let metadata: CaptureMetadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }

    /// List all captures.
    pub fn list_captures(&self) -> Result<Vec<CaptureMetadata>> {
        let mut captures = Vec::new();

        for entry in fs::read_dir(self.captures_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json = fs::read_to_string(&path)?;
                if let Ok(metadata) = serde_json::from_str::<CaptureMetadata>(&json) {
                    captures.push(metadata);
                }
            }
        }

        // Sort by timestamp (newest first)
        captures.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(captures)
    }

    /// Delete a capture (both PCAP and metadata).
    pub fn delete_capture(&self, capture_id: &str) -> Result<()> {
        // Delete metadata JSON
        let json_path = self.captures_dir().join(format!("{}.json", capture_id));
        if json_path.exists() {
            fs::remove_file(json_path)?;
        }

        // Delete PCAP file
        let pcap_path = self.captures_dir().join(format!("{}.pcap", capture_id));
        if pcap_path.exists() {
            fs::remove_file(pcap_path)?;
        }

        Ok(())
    }
}
