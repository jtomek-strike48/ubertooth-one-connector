//! Capture management methods for SidecarManager.

use crate::capture_store::CaptureStore;
use serde_json::{json, Value};
use std::path::PathBuf;
use ubertooth_core::error::{Result, UbertoothError};

use super::SidecarManager;

impl SidecarManager {
    /// List captures implementation.
    pub(super) async fn capture_list(&self, params: Value) -> Result<Value> {
        let store = CaptureStore::new()?;

        let filter_type = params.get("filter_type").and_then(|v| v.as_str());
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        // Get all captures
        let mut all_captures = store.list_captures()?;

        // Filter by type if specified
        if let Some(filter) = filter_type {
            all_captures.retain(|c| c.capture_type == filter);
        }

        let total_count = all_captures.len();

        // Apply pagination
        let captures: Vec<_> = all_captures
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                json!({
                    "capture_id": c.capture_id,
                    "timestamp": c.timestamp.to_rfc3339(),
                    "type": c.capture_type,
                    "packet_count": c.packet_count,
                    "duration_sec": c.duration_sec,
                    "file_size_bytes": c.file_size_bytes,
                    "pcap_path": c.pcap_path,
                    "tags": c.tags,
                    "description": c.description
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "captures": captures,
            "total_count": total_count,
            "offset": offset,
            "limit": limit
        }))
    }

    /// Get capture implementation.
    pub(super) async fn capture_get(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let store = CaptureStore::new()?;
        let metadata = store.load_metadata(capture_id)?;

        // For Phase 1, return metadata without parsing PCAP
        // PCAP parsing will be added in Phase 2
        let packets: Vec<Value> = Vec::new();
        let has_more = false;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "offset": offset,
            "limit": limit,
            "packet_count": metadata.packet_count,
            "packets": packets,
            "has_more": has_more,
            "note": "Phase 1: PCAP parsing not yet implemented. Use pcap_path to access raw file."
        }))
    }

    /// Delete capture implementation.
    pub(super) async fn capture_delete(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let store = CaptureStore::new()?;
        store.delete_capture(capture_id)?;

        Ok(json!({
            "success": true,
            "message": format!("Capture '{}' deleted", capture_id)
        }))
    }

    /// Tag capture implementation.
    pub(super) async fn capture_tag(&self, params: Value) -> Result<Value> {
        let capture_id = params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let new_tags = params.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        });

        let new_description = params.get("description").and_then(|v| v.as_str());
        let append_tags = params
            .get("append_tags")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let store = CaptureStore::new()?;
        let mut metadata = store.load_metadata(capture_id)?;

        // Update tags
        if let Some(tags) = new_tags {
            if append_tags {
                metadata.tags.extend(tags);
                metadata.tags.sort();
                metadata.tags.dedup();
            } else {
                metadata.tags = tags;
            }
        }

        // Update description
        if let Some(desc) = new_description {
            metadata.description = desc.to_string();
        }

        // Save updated metadata
        store.save_metadata(&metadata)?;

        Ok(json!({
            "success": true,
            "capture_id": capture_id,
            "tags": metadata.tags,
            "description": metadata.description
        }))
    }

    /// Export capture implementation.
    pub(super) async fn capture_export(&self, _params: Value) -> Result<Value> {
        let capture_id = _params
            .get("capture_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'capture_id'".to_string()))?;

        let format = _params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("pcap");
        let output_path = _params.get("output_path").and_then(|v| v.as_str());

        tracing::info!("Exporting capture {} to format {}", capture_id, format);

        let store = CaptureStore::new()?;
        let input_path = store.captures_dir().join(format!("{}.pcap", capture_id));

        if !input_path.exists() {
            return Err(UbertoothError::CaptureNotFound(capture_id.to_string()));
        }

        // Determine export path
        let export_path = if let Some(path) = output_path {
            PathBuf::from(path)
        } else {
            store
                .captures_dir()
                .join(format!("{}.{}", capture_id, format))
        };

        // Use tshark or editcap for format conversion
        match format {
            "pcap" => {
                // Just copy the file
                std::fs::copy(&input_path, &export_path)?;
            }
            "pcapng" => {
                // Use editcap to convert to pcapng
                self.execute_ubertooth_command(
                    "editcap",
                    &[
                        "-F",
                        "pcapng",
                        input_path.to_str().unwrap(),
                        export_path.to_str().unwrap(),
                    ],
                )
                .await?;
            }
            "json" => {
                // Use tshark to export to JSON
                let json_output = self
                    .execute_ubertooth_command(
                        "tshark",
                        &["-r", input_path.to_str().unwrap(), "-T", "json"],
                    )
                    .await?;
                std::fs::write(&export_path, json_output)?;
            }
            "csv" => {
                // Use tshark to export to CSV
                let csv_output = self
                    .execute_ubertooth_command(
                        "tshark",
                        &[
                            "-r",
                            input_path.to_str().unwrap(),
                            "-T",
                            "fields",
                            "-E",
                            "header=y",
                            "-E",
                            "separator=,",
                        ],
                    )
                    .await?;
                std::fs::write(&export_path, csv_output)?;
            }
            _ => {
                return Err(UbertoothError::InvalidParameter(format!(
                    "Unsupported format: {}",
                    format
                )));
            }
        }

        // Get packet count
        let metadata = store.load_metadata(capture_id)?;
        let packet_count = metadata.packet_count;

        // Get exported file size
        let file_size_bytes = if export_path.exists() {
            std::fs::metadata(&export_path)?.len()
        } else {
            0
        };

        Ok(json!({
            "success": true,
            "export_path": export_path.to_string_lossy(),
            "format": format,
            "packet_count": packet_count,
            "file_size_bytes": file_size_bytes
        }))
    }
}
