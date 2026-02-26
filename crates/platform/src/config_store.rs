//! Configuration preset storage and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use ubertooth_core::error::{Result, UbertoothError};

/// Configuration preset metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub name: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub settings: ConfigSettings,
}

/// Radio configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modulation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgm: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squelch: Option<i32>,
}

/// Configuration storage manager.
pub struct ConfigStore {
    configs_dir: PathBuf,
}

impl ConfigStore {
    /// Create a new config store at ~/.ubertooth/configs/
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| UbertoothError::BackendError("Cannot find home directory".to_string()))?;

        let ubertooth_dir = home.join(".ubertooth");
        let configs_dir = ubertooth_dir.join("configs");

        // Create directory if it doesn't exist
        fs::create_dir_all(&configs_dir).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to create configs directory: {}", e))
        })?;

        Ok(Self { configs_dir })
    }

    /// Get the configs directory path.
    pub fn configs_dir(&self) -> &PathBuf {
        &self.configs_dir
    }

    /// Save a configuration preset.
    pub fn save_config(&self, config: &ConfigMetadata, overwrite: bool) -> Result<PathBuf> {
        let config_path = self.configs_dir.join(format!("{}.json", config.name));

        // Check if config already exists
        if config_path.exists() && !overwrite {
            return Err(UbertoothError::BackendError(format!(
                "Configuration '{}' already exists. Use overwrite=true to replace.",
                config.name
            )));
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(config).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to serialize config: {}", e))
        })?;

        // Write to file
        fs::write(&config_path, json).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to write config file: {}", e))
        })?;

        Ok(config_path)
    }

    /// Load a configuration preset by name.
    pub fn load_config(&self, name: &str) -> Result<ConfigMetadata> {
        let config_path = self.configs_dir.join(format!("{}.json", name));

        if !config_path.exists() {
            return Err(UbertoothError::BackendError(format!(
                "Configuration '{}' not found",
                name
            )));
        }

        // Read file
        let json = fs::read_to_string(&config_path).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to read config file: {}", e))
        })?;

        // Deserialize
        let config: ConfigMetadata = serde_json::from_str(&json).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// List all saved configurations.
    pub fn list_configs(&self) -> Result<Vec<ConfigMetadata>> {
        let mut configs = Vec::new();

        // Read directory entries
        let entries = fs::read_dir(&self.configs_dir).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to read configs directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                UbertoothError::BackendError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<ConfigMetadata>(&json) {
                        configs.push(config);
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        configs.sort_by(|a, b| b.created.cmp(&a.created));

        Ok(configs)
    }

    /// Delete a configuration preset by name.
    pub fn delete_config(&self, name: &str) -> Result<()> {
        let config_path = self.configs_dir.join(format!("{}.json", name));

        if !config_path.exists() {
            return Err(UbertoothError::BackendError(format!(
                "Configuration '{}' not found",
                name
            )));
        }

        fs::remove_file(&config_path).map_err(|e| {
            UbertoothError::BackendError(format!("Failed to delete config file: {}", e))
        })?;

        Ok(())
    }

    /// Check if a configuration exists.
    pub fn config_exists(&self, name: &str) -> bool {
        self.configs_dir.join(format!("{}.json", name)).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_store_creation() {
        let store = ConfigStore::new();
        assert!(store.is_ok());
    }

    #[test]
    fn test_config_settings_serialization() {
        let settings = ConfigSettings {
            channel: Some(37),
            modulation: Some("BT_LOW_ENERGY".to_string()),
            power_level: Some(7),
            paen: Some(true),
            hgm: Some(false),
            squelch: Some(-90),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"channel\":37"));
        assert!(json.contains("\"modulation\":\"BT_LOW_ENERGY\""));
    }
}
