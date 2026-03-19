//! PCAP import/export functionality.

use crate::capture_store::{CaptureMetadata, CaptureStore};
use chrono::Utc;
use pcap_file::pcap::{PcapReader, PcapWriter};
use pcap_file::pcapng::{PcapNgReader, PcapNgWriter};
use pcap_file::PcapError;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use ubertooth_core::error::{Result, UbertoothError};
use uuid::Uuid;

/// PCAP format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PcapFormat {
    /// Classic PCAP format (.pcap)
    Pcap,
    /// PCAP Next Generation format (.pcapng)
    PcapNg,
}

impl PcapFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Result<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("pcap") => Ok(Self::Pcap),
            Some("pcapng") => Ok(Self::PcapNg),
            _ => Err(UbertoothError::InvalidParameter(
                "Unknown PCAP extension (expected .pcap or .pcapng)".to_string(),
            )),
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            Self::Pcap => "pcap",
            Self::PcapNg => "pcapng",
        }
    }
}

/// PCAP import/export manager
pub struct PcapManager {
    capture_store: CaptureStore,
}

impl PcapManager {
    /// Create new PCAP manager
    pub fn new(capture_store: CaptureStore) -> Self {
        Self { capture_store }
    }

    /// Import PCAP file into capture store
    pub fn import_pcap(&mut self, pcap_path: &Path, description: Option<String>) -> Result<String> {
        // Detect format
        let format = PcapFormat::from_extension(pcap_path)?;

        // Validate file exists
        if !pcap_path.exists() {
            return Err(UbertoothError::InvalidParameter(format!(
                "PCAP file not found: {:?}",
                pcap_path
            )));
        }

        // Read and count packets
        let packet_count = match format {
            PcapFormat::Pcap => self.count_pcap_packets(pcap_path)?,
            PcapFormat::PcapNg => self.count_pcapng_packets(pcap_path)?,
        };

        // Get file size
        let file_size_bytes = std::fs::metadata(pcap_path)?.len();

        // Generate capture ID
        let capture_id = Uuid::new_v4().to_string();

        // Copy PCAP to capture store directory
        let captures_dir = self.capture_store.captures_dir();
        let capture_subdir = captures_dir.join(&capture_id);
        std::fs::create_dir_all(&capture_subdir)?;

        let dest_pcap = capture_subdir.join(format!("capture.{}", format.extension()));
        std::fs::copy(pcap_path, &dest_pcap)?;

        // Create metadata
        let metadata = CaptureMetadata {
            capture_id: capture_id.clone(),
            timestamp: Utc::now(),
            capture_type: "imported".to_string(),
            packet_count,
            duration_sec: None,
            file_size_bytes,
            pcap_path: dest_pcap.to_string_lossy().to_string(),
            tags: vec!["imported".to_string()],
            description: description.unwrap_or_else(|| {
                format!("Imported from {}", pcap_path.file_name().unwrap_or_default().to_string_lossy())
            }),
            category: None,
            notes: None,
        };

        // Save metadata to JSON file
        let metadata_path = capture_subdir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(metadata_path, metadata_json)?;

        tracing::info!("Imported {} packets from {:?}", packet_count, pcap_path);

        Ok(capture_id)
    }

    /// Export capture to PCAP file
    pub fn export_pcap(
        &self,
        capture_id: &str,
        output_path: &Path,
    ) -> Result<()> {
        // Get capture metadata
        let captures_dir = self.capture_store.captures_dir();
        let metadata_path = captures_dir.join(capture_id).join("metadata.json");
        let metadata_json = std::fs::read_to_string(metadata_path)?;
        let metadata: CaptureMetadata = serde_json::from_str(&metadata_json)?;

        // Source PCAP path
        let source_pcap = PathBuf::from(&metadata.pcap_path);
        if !source_pcap.exists() {
            return Err(UbertoothError::InvalidParameter(format!(
                "Source PCAP not found: {:?}",
                source_pcap
            )));
        }

        // Copy file
        std::fs::copy(&source_pcap, output_path)?;
        tracing::info!("Exported capture {} to {:?}", capture_id, output_path);

        Ok(())
    }

    /// Count packets in PCAP file
    fn count_pcap_packets(&self, path: &Path) -> Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut pcap_reader = PcapReader::new(reader)
            .map_err(|e| UbertoothError::ParseError(format!("Failed to open PCAP: {}", e)))?;

        let mut count = 0;
        while let Some(pkt) = pcap_reader.next_packet() {
            match pkt {
                Ok(_) => count += 1,
                Err(PcapError::IncompleteBuffer) => break,
                Err(e) => {
                    tracing::warn!("Error reading packet {}: {}", count, e);
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Count packets in PCAPNG file
    fn count_pcapng_packets(&self, path: &Path) -> Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut pcapng_reader = PcapNgReader::new(reader)
            .map_err(|e| UbertoothError::ParseError(format!("Failed to open PCAPNG: {}", e)))?;

        let mut count = 0;
        while let Some(block) = pcapng_reader.next_block() {
            match block {
                Ok(pcap_file::pcapng::Block::EnhancedPacket(_)) => count += 1,
                Ok(pcap_file::pcapng::Block::SimplePacket(_)) => count += 1,
                Ok(_) => {}, // Other block types
                Err(PcapError::IncompleteBuffer) => break,
                Err(e) => {
                    tracing::warn!("Error reading block: {}", e);
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Validate PCAP file integrity
    pub fn validate_pcap(path: &Path) -> Result<PcapValidation> {
        let format = PcapFormat::from_extension(path)?;

        if !path.exists() {
            return Err(UbertoothError::InvalidParameter(format!(
                "PCAP file not found: {:?}",
                path
            )));
        }

        let file_size = std::fs::metadata(path)?.len();

        match format {
            PcapFormat::Pcap => Self::validate_pcap_format(path, file_size),
            PcapFormat::PcapNg => Self::validate_pcapng_format(path, file_size),
        }
    }

    fn validate_pcap_format(path: &Path, file_size: u64) -> Result<PcapValidation> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut pcap_reader = PcapReader::new(reader)
            .map_err(|e| UbertoothError::ParseError(format!("Failed to open PCAP: {}", e)))?;

        let mut packet_count = 0;
        let mut errors = Vec::new();

        while let Some(pkt) = pcap_reader.next_packet() {
            match pkt {
                Ok(_) => packet_count += 1,
                Err(PcapError::IncompleteBuffer) => break,
                Err(e) => {
                    errors.push(format!("Packet {}: {}", packet_count, e));
                    if errors.len() >= 10 {
                        errors.push("... (truncated)".to_string());
                        break;
                    }
                }
            }
        }

        Ok(PcapValidation {
            format: PcapFormat::Pcap,
            file_size,
            packet_count,
            is_valid: errors.is_empty(),
            errors,
        })
    }

    fn validate_pcapng_format(path: &Path, file_size: u64) -> Result<PcapValidation> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut pcapng_reader = PcapNgReader::new(reader)
            .map_err(|e| UbertoothError::ParseError(format!("Failed to open PCAPNG: {}", e)))?;

        let mut packet_count = 0;
        let mut errors = Vec::new();

        while let Some(block) = pcapng_reader.next_block() {
            match block {
                Ok(pcap_file::pcapng::Block::EnhancedPacket(_)) => packet_count += 1,
                Ok(pcap_file::pcapng::Block::SimplePacket(_)) => packet_count += 1,
                Ok(_) => {},
                Err(PcapError::IncompleteBuffer) => break,
                Err(e) => {
                    errors.push(format!("Block: {}", e));
                    if errors.len() >= 10 {
                        errors.push("... (truncated)".to_string());
                        break;
                    }
                }
            }
        }

        Ok(PcapValidation {
            format: PcapFormat::PcapNg,
            file_size,
            packet_count,
            is_valid: errors.is_empty(),
            errors,
        })
    }
}

/// PCAP validation result
#[derive(Debug)]
pub struct PcapValidation {
    pub format: PcapFormat,
    pub file_size: u64,
    pub packet_count: usize,
    pub is_valid: bool,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> CaptureStore {
        CaptureStore::new().unwrap()
    }

    #[test]
    fn test_pcap_format_detection() {
        assert_eq!(
            PcapFormat::from_extension(Path::new("test.pcap")).unwrap(),
            PcapFormat::Pcap
        );
        assert_eq!(
            PcapFormat::from_extension(Path::new("test.pcapng")).unwrap(),
            PcapFormat::PcapNg
        );
        assert!(PcapFormat::from_extension(Path::new("test.txt")).is_err());
    }

    #[test]
    fn test_pcap_format_extension() {
        assert_eq!(PcapFormat::Pcap.extension(), "pcap");
        assert_eq!(PcapFormat::PcapNg.extension(), "pcapng");
    }

    #[test]
    fn test_validate_missing_file() {
        let result = PcapManager::validate_pcap(Path::new("/nonexistent.pcap"));
        assert!(result.is_err());
    }

    #[test]
    fn test_import_missing_file() {
        let store = create_test_store();
        let mut manager = PcapManager::new(store);

        let result = manager.import_pcap(Path::new("/nonexistent.pcap"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_nonexistent_capture() {
        let store = create_test_store();
        let manager = PcapManager::new(store);

        let temp_out = tempfile::NamedTempFile::new().unwrap();
        let result = manager.export_pcap(
            "nonexistent-id",
            temp_out.path(),
        );
        assert!(result.is_err());
    }
}
