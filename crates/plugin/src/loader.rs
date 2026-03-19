//! Plugin dynamic loading functionality.

use crate::{Plugin, PluginError, PluginResult};
use libloading::{Library, Symbol};
use std::path::Path;

/// Plugin loader for dynamic libraries
pub struct PluginLoader {
    /// Loaded library
    #[allow(dead_code)]
    library: Library,
}

impl PluginLoader {
    /// Load plugin from path
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> PluginResult<(Self, Box<dyn Plugin>)> {
        let library = Library::new(path.as_ref()).map_err(|e| {
            PluginError::LoadError(format!("Failed to load library: {}", e))
        })?;

        // Get the plugin constructor function
        let constructor: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = library
            .get(b"_plugin_create")
            .map_err(|e| {
                PluginError::LoadError(format!("Failed to find _plugin_create symbol: {}", e))
            })?;

        // Call constructor
        let plugin_ptr = constructor();
        if plugin_ptr.is_null() {
            return Err(PluginError::LoadError(
                "Plugin constructor returned null".to_string(),
            ));
        }

        let plugin = Box::from_raw(plugin_ptr);

        let loader = Self { library };

        Ok((loader, plugin))
    }

    /// Validate plugin library before loading
    pub fn validate<P: AsRef<Path>>(path: P) -> PluginResult<bool> {
        if !path.as_ref().exists() {
            return Err(PluginError::NotFound(format!(
                "Plugin file not found: {:?}",
                path.as_ref()
            )));
        }

        // Check file extension
        let extension = path.as_ref().extension().and_then(|e| e.to_str());
        let valid = match extension {
            Some("so") => true,  // Linux
            Some("dylib") => true, // macOS
            Some("dll") => true, // Windows
            _ => false,
        };

        if !valid {
            return Err(PluginError::LoadError(format!(
                "Invalid plugin file extension: {:?}",
                extension
            )));
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_nonexistent() {
        let result = PluginLoader::validate("/nonexistent/plugin.so");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_wrong_extension() {
        use std::fs::File;
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let plugin_path = temp_dir.path().join("plugin.txt");
        let mut file = File::create(&plugin_path).unwrap();
        file.write_all(b"test").unwrap();

        let result = PluginLoader::validate(&plugin_path);
        assert!(result.is_err());
    }
}
