//! Capture storage management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use ubertooth_core::error::{Result, UbertoothError};
use uuid::Uuid;

/// Capture metadata structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptureMetadata {
    pub capture_id: String,
    pub timestamp: DateTime<Utc>,
    pub capture_type: String,
    pub packet_count: usize,
    pub duration_sec: Option<u64>,
    pub file_size_bytes: u64,
    pub pcap_path: String,
    /// Hierarchical tags (e.g., "device/phone/android", "attack/mitm")
    pub tags: Vec<String>,
    pub description: String,
    /// Category for grouping captures
    pub category: Option<String>,
    /// User notes (supports markdown)
    pub notes: Option<String>,
}

/// Tag with hierarchy information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag {
    /// Full tag path (e.g., "device/phone/android")
    pub path: String,
}

impl Tag {
    /// Create a new tag from path
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Get tag components
    pub fn components(&self) -> Vec<&str> {
        self.path.split('/').collect()
    }

    /// Get the parent tag path
    pub fn parent(&self) -> Option<String> {
        let parts: Vec<&str> = self.path.rsplitn(2, '/').collect();
        if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }

    /// Get the tag name (last component)
    pub fn name(&self) -> &str {
        self.path.split('/').next_back().unwrap_or(&self.path)
    }

    /// Check if this is a child of another tag
    pub fn is_child_of(&self, parent: &str) -> bool {
        self.path.starts_with(&format!("{}/", parent)) || self.path == parent
    }

    /// Get depth (number of slashes)
    pub fn depth(&self) -> usize {
        self.path.matches('/').count()
    }
}

/// Capture category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureCategory {
    /// Reconnaissance and scanning
    Reconnaissance,
    /// Active attacks
    Attack,
    /// Defense and monitoring
    Defense,
    /// Analysis and investigation
    Analysis,
    /// Testing and development
    Testing,
    /// Custom category
    Custom(String),
}

impl CaptureCategory {
    /// Get all predefined categories
    pub fn predefined() -> Vec<Self> {
        vec![
            Self::Reconnaissance,
            Self::Attack,
            Self::Defense,
            Self::Analysis,
            Self::Testing,
        ]
    }

    /// Get category name
    pub fn name(&self) -> &str {
        match self {
            Self::Reconnaissance => "Reconnaissance",
            Self::Attack => "Attack",
            Self::Defense => "Defense",
            Self::Analysis => "Analysis",
            Self::Testing => "Testing",
            Self::Custom(name) => name,
        }
    }
}

impl FromStr for CaptureCategory {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "Reconnaissance" => Self::Reconnaissance,
            "Attack" => Self::Attack,
            "Defense" => Self::Defense,
            "Analysis" => Self::Analysis,
            "Testing" => Self::Testing,
            custom => Self::Custom(custom.to_string()),
        })
    }
}

/// Capture storage manager.
pub struct CaptureStore {
    base_path: PathBuf,
}

impl CaptureStore {
    /// Create a new capture store at ~/.ubertooth/
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            UbertoothError::BackendError("Could not determine home directory".to_string())
        })?;

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

    /// Filter captures by tags (supports hierarchical matching)
    pub fn filter_by_tags(&self, tag_filters: &[String]) -> Result<Vec<CaptureMetadata>> {
        let all_captures = self.list_captures()?;

        if tag_filters.is_empty() {
            return Ok(all_captures);
        }

        let filtered: Vec<CaptureMetadata> = all_captures
            .into_iter()
            .filter(|capture| {
                // Check if capture has any of the filter tags (or their children)
                tag_filters.iter().any(|filter| {
                    capture
                        .tags
                        .iter()
                        .any(|tag| tag == filter || tag.starts_with(&format!("{}/", filter)))
                })
            })
            .collect();

        Ok(filtered)
    }

    /// Filter captures by category
    pub fn filter_by_category(&self, category: &str) -> Result<Vec<CaptureMetadata>> {
        let all_captures = self.list_captures()?;

        let filtered: Vec<CaptureMetadata> = all_captures
            .into_iter()
            .filter(|capture| capture.category.as_ref().is_some_and(|c| c == category))
            .collect();

        Ok(filtered)
    }

    /// Get all unique tags across all captures
    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let captures = self.list_captures()?;
        let mut tags = std::collections::HashSet::new();

        for capture in captures {
            for tag_str in capture.tags {
                tags.insert(Tag::new(tag_str));
            }
        }

        let mut tag_vec: Vec<Tag> = tags.into_iter().collect();
        tag_vec.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(tag_vec)
    }

    /// Get tag suggestions based on partial input
    pub fn suggest_tags(&self, partial: &str) -> Result<Vec<String>> {
        let all_tags = self.get_all_tags()?;

        let suggestions: Vec<String> = all_tags
            .into_iter()
            .filter(|tag| tag.path.starts_with(partial))
            .map(|tag| tag.path)
            .take(10) // Limit to 10 suggestions
            .collect();

        Ok(suggestions)
    }

    /// Add tag to capture
    pub fn add_tag(&self, capture_id: &str, tag: String) -> Result<()> {
        let mut metadata = self.load_metadata(capture_id)?;

        if !metadata.tags.contains(&tag) {
            metadata.tags.push(tag);
            self.save_metadata(&metadata)?;
        }

        Ok(())
    }

    /// Remove tag from capture
    pub fn remove_tag(&self, capture_id: &str, tag: &str) -> Result<()> {
        let mut metadata = self.load_metadata(capture_id)?;

        metadata.tags.retain(|t| t != tag);
        self.save_metadata(&metadata)?;

        Ok(())
    }

    /// Set capture category
    pub fn set_category(&self, capture_id: &str, category: Option<String>) -> Result<()> {
        let mut metadata = self.load_metadata(capture_id)?;

        metadata.category = category;
        self.save_metadata(&metadata)?;

        Ok(())
    }

    /// Update capture notes
    pub fn update_notes(&self, capture_id: &str, notes: Option<String>) -> Result<()> {
        let mut metadata = self.load_metadata(capture_id)?;

        metadata.notes = notes;
        self.save_metadata(&metadata)?;

        Ok(())
    }

    /// Get captures grouped by category
    pub fn group_by_category(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<CaptureMetadata>>> {
        let captures = self.list_captures()?;
        let mut grouped: std::collections::HashMap<String, Vec<CaptureMetadata>> =
            std::collections::HashMap::new();

        for capture in captures {
            let category = capture
                .category
                .clone()
                .unwrap_or_else(|| "Uncategorized".to_string());
            grouped.entry(category).or_default().push(capture);
        }

        Ok(grouped)
    }

    /// Search captures by description or notes
    pub fn search(&self, query: &str) -> Result<Vec<CaptureMetadata>> {
        let all_captures = self.list_captures()?;
        let query_lower = query.to_lowercase();

        let results: Vec<CaptureMetadata> = all_captures
            .into_iter()
            .filter(|capture| {
                capture.description.to_lowercase().contains(&query_lower)
                    || capture
                        .notes
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower))
                    || capture
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_components() {
        let tag = Tag::new("device/phone/android".to_string());
        assert_eq!(tag.components(), vec!["device", "phone", "android"]);
        assert_eq!(tag.name(), "android");
        assert_eq!(tag.depth(), 2);
    }

    #[test]
    fn test_tag_parent() {
        let tag = Tag::new("device/phone/android".to_string());
        assert_eq!(tag.parent(), Some("device/phone".to_string()));

        let root_tag = Tag::new("device".to_string());
        assert_eq!(root_tag.parent(), None);
    }

    #[test]
    fn test_tag_is_child_of() {
        let tag = Tag::new("device/phone/android".to_string());

        assert!(tag.is_child_of("device"));
        assert!(tag.is_child_of("device/phone"));
        assert!(tag.is_child_of("device/phone/android"));
        assert!(!tag.is_child_of("attack"));
    }

    #[test]
    fn test_capture_category() {
        let cat = CaptureCategory::Reconnaissance;
        assert_eq!(cat.name(), "Reconnaissance");

        let custom = CaptureCategory::Custom("My Category".to_string());
        assert_eq!(custom.name(), "My Category");
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(
            CaptureCategory::from_str("Attack").unwrap(),
            CaptureCategory::Attack
        );
        assert_eq!(
            CaptureCategory::from_str("Custom").unwrap(),
            CaptureCategory::Custom("Custom".to_string())
        );
    }
}
