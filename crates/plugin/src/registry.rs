//! Plugin registry for managing loaded plugins.

use crate::{
    loader::PluginLoader, Plugin, PluginCapability, PluginContext, PluginError, PluginResult,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin registry
pub struct PluginRegistry {
    /// Loaded plugins
    plugins: HashMap<String, PluginEntry>,
    /// Plugin directory
    plugin_dir: PathBuf,
}

/// Plugin entry
struct PluginEntry {
    /// Plugin instance
    plugin: Box<dyn Plugin>,
    /// Plugin loader (keeps library loaded)
    #[allow(dead_code)]
    loader: PluginLoader,
}

impl PluginRegistry {
    /// Create new plugin registry
    pub fn new<P: AsRef<Path>>(plugin_dir: P) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }

    /// Load plugin from file
    pub fn load_plugin<P: AsRef<Path>>(
        &mut self,
        path: P,
        context: PluginContext,
    ) -> PluginResult<String> {
        // Validate plugin file
        PluginLoader::validate(&path)?;

        // Load plugin
        let (loader, mut plugin) = unsafe { PluginLoader::load(path)? };

        // Get plugin name before initialization
        let name = plugin.metadata().name.clone();

        // Check if already loaded
        if self.plugins.contains_key(&name) {
            return Err(PluginError::LoadError(format!(
                "Plugin '{}' is already loaded",
                name
            )));
        }

        // Initialize plugin
        plugin.initialize(context).map_err(|e| {
            PluginError::InitError(format!("Failed to initialize plugin '{}': {}", name, e))
        })?;

        tracing::info!(
            "Loaded plugin: {} v{}",
            plugin.metadata().name,
            plugin.metadata().version
        );

        // Store plugin
        self.plugins
            .insert(name.clone(), PluginEntry { plugin, loader });

        Ok(name)
    }

    /// Load all plugins from directory
    pub fn load_all(&mut self, context: PluginContext) -> Vec<PluginResult<String>> {
        let mut results = Vec::new();

        if !self.plugin_dir.exists() {
            tracing::warn!("Plugin directory does not exist: {:?}", self.plugin_dir);
            return results;
        }

        let entries = match std::fs::read_dir(&self.plugin_dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::error!("Failed to read plugin directory: {}", e);
                return results;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Check extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap_or("");
                    if ext_str == "so" || ext_str == "dylib" || ext_str == "dll" {
                        let result = self.load_plugin(&path, context.clone());
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|entry| &*entry.plugin)
    }

    /// Get mutable plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut Box<dyn Plugin>> {
        self.plugins.get_mut(name).map(|entry| &mut entry.plugin)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get plugins with specific capability
    pub fn get_plugins_with_capability(&self, capability: PluginCapability) -> Vec<&str> {
        self.plugins
            .iter()
            .filter(|(_, entry)| entry.plugin.capabilities().contains(&capability))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Unload plugin
    pub fn unload_plugin(&mut self, name: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .remove(name)
            .ok_or_else(|| PluginError::NotFound(format!("Plugin '{}' not found", name)))?;

        // Shutdown plugin
        let mut plugin = entry.plugin;
        plugin.shutdown()?;

        tracing::info!("Unloaded plugin: {}", name);

        Ok(())
    }

    /// Unload all plugins
    pub fn unload_all(&mut self) -> Vec<PluginResult<()>> {
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();
        let mut results = Vec::new();

        for name in plugin_names {
            results.push(self.unload_plugin(&name));
        }

        results
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Drop for PluginRegistry {
    fn drop(&mut self) {
        // Shutdown all plugins
        let _ = self.unload_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_registry_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_list_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        assert!(registry.list_plugins().is_empty());
    }

    #[test]
    fn test_load_all_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path());

        let context = PluginContext::new(HashMap::new(), temp_dir.path().to_path_buf());
        let results = registry.load_all(context);

        assert!(results.is_empty());
    }

    #[test]
    fn test_get_nonexistent_plugin() {
        let temp_dir = tempfile::tempdir().unwrap();
        let registry = PluginRegistry::new(temp_dir.path());
        assert!(registry.get_plugin("nonexistent").is_none());
    }
}
