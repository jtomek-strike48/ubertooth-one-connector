//! Tool parameter form builder

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tui_textarea::TextArea;
use ubertooth_core::PentestTool;

/// Field type extracted from JSON schema
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
}

impl FieldType {
    fn from_schema(schema_type: &Value) -> Self {
        match schema_type.as_str() {
            Some("string") => FieldType::String,
            Some("number") => FieldType::Number,
            Some("integer") => FieldType::Integer,
            Some("boolean") => FieldType::Boolean,
            Some("array") => FieldType::Array,
            _ => FieldType::String, // Default to string
        }
    }
}

/// A form field definition
#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: FieldType,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
    pub dropdown_options: Option<Vec<String>>,
}

/// Field input mode
#[derive(Debug, Clone)]
pub enum FieldInputMode {
    /// Text input
    Text,
    /// Dropdown/select menu
    Dropdown { selected_index: usize },
}

/// Tool parameter form
pub struct ToolForm {
    /// Tool being configured
    tool: Arc<dyn PentestTool>,

    /// Form fields
    fields: Vec<FormField>,

    /// Text inputs for each field (used when not dropdown)
    inputs: Vec<TextArea<'static>>,

    /// Input modes for each field
    input_modes: Vec<FieldInputMode>,

    /// Currently focused field index
    focused_index: usize,
}

impl std::fmt::Debug for ToolForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolForm")
            .field("tool_name", &self.tool.name())
            .field("fields", &self.fields)
            .field("input_modes", &self.input_modes)
            .field("focused_index", &self.focused_index)
            .finish()
    }
}

impl ToolForm {
    /// Create a new form for the given tool
    pub fn new(tool: Arc<dyn PentestTool>) -> Result<Self> {
        let schema = tool.input_schema();
        let fields = Self::parse_schema(&schema)?;

        // Create input mode for each field (dropdown or text)
        let mut inputs = Vec::new();
        let mut input_modes = Vec::new();

        for field in &fields {
            if let Some(options) = &field.dropdown_options {
                // Dropdown field
                let default_index = if let Some(default) = &field.default {
                    options.iter().position(|o| o == default).unwrap_or(0)
                } else {
                    0
                };
                input_modes.push(FieldInputMode::Dropdown {
                    selected_index: default_index,
                });
                // Still create textarea for compatibility
                inputs.push(TextArea::default());
            } else {
                // Text input field
                let mut textarea = TextArea::default();
                if let Some(default) = &field.default {
                    textarea.insert_str(default);
                }
                inputs.push(textarea);
                input_modes.push(FieldInputMode::Text);
            }
        }

        Ok(Self {
            tool,
            fields,
            inputs,
            input_modes,
            focused_index: 0,
        })
    }

    /// Determine if a field should have dropdown options
    fn get_dropdown_options(name: &str, field_type: &FieldType, prop: &Value) -> Option<Vec<String>> {
        // Check for explicit enum in schema
        if let Some(enum_values) = prop.get("enum").and_then(|e| e.as_array()) {
            let options: Vec<String> = enum_values
                .iter()
                .filter_map(|v| {
                    // Handle both string and integer enum values
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(n) = v.as_i64() {
                        Some(n.to_string())
                    } else if let Some(n) = v.as_u64() {
                        Some(n.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if !options.is_empty() {
                return Some(options);
            }
        }

        // Boolean fields → Yes/No dropdown
        if matches!(field_type, FieldType::Boolean) {
            return Some(vec!["true".to_string(), "false".to_string()]);
        }

        // Channel field → BLE advertising channels
        if name == "channel" {
            return Some(vec!["37".to_string(), "38".to_string(), "39".to_string()]);
        }

        // Duration presets (common values)
        if name.contains("duration") && matches!(field_type, FieldType::Integer) {
            return Some(vec![
                "5".to_string(),
                "10".to_string(),
                "30".to_string(),
                "60".to_string(),
                "120".to_string(),
                "300".to_string(),
            ]);
        }

        // Save/capture boolean flags
        if name.starts_with("save_") || name.ends_with("_pcap") {
            return Some(vec!["true".to_string(), "false".to_string()]);
        }

        // Capture ID field → populate with available captures
        if name == "capture_id" {
            if let Some(captures) = Self::get_available_captures() {
                if !captures.is_empty() {
                    let mut options = captures;
                    options.push("Other (manual)".to_string());
                    return Some(options);
                }
            }
        }

        None
    }

    /// Get list of available capture IDs from ~/.ubertooth/captures/
    fn get_available_captures() -> Option<Vec<String>> {
        use std::fs;
        use std::path::PathBuf;

        // Get captures directory
        let mut captures_dir = PathBuf::from(std::env::var("HOME").ok()?);
        captures_dir.push(".ubertooth");
        captures_dir.push("captures");

        if !captures_dir.exists() {
            return None;
        }

        // Read all .json metadata files
        let mut captures = Vec::new();
        if let Ok(entries) = fs::read_dir(&captures_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    // Read metadata to get capture_id
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(id) = metadata.get("capture_id").and_then(|v| v.as_str()) {
                                captures.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Sort by most recent (reverse alphabetical by ID which includes timestamp)
        captures.sort();
        captures.reverse();

        // Limit to most recent 20 to keep dropdown manageable
        captures.truncate(20);

        if captures.is_empty() {
            None
        } else {
            Some(captures)
        }
    }

    /// Parse JSON schema into form fields
    fn parse_schema(schema: &Value) -> Result<Vec<FormField>> {
        let mut fields = Vec::new();

        let empty_map = serde_json::Map::new();
        let properties = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap_or(&empty_map);

        let required_fields = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for (name, prop) in properties {
            let field_type = FieldType::from_schema(prop.get("type").unwrap_or(&json!("string")));
            let description = prop
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            let required = required_fields.contains(&name.as_str());
            // Get default value, handling both strings and integers
            let default = prop.get("default").map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else if let Some(n) = v.as_i64() {
                    n.to_string()
                } else if let Some(n) = v.as_u64() {
                    n.to_string()
                } else if let Some(b) = v.as_bool() {
                    b.to_string()
                } else {
                    v.to_string().trim_matches('"').to_string()
                }
            });

            // Detect dropdown options
            let dropdown_options = Self::get_dropdown_options(name, &field_type, prop);

            fields.push(FormField {
                name: name.clone(),
                field_type,
                description,
                required,
                default,
                dropdown_options,
            });
        }

        Ok(fields)
    }

    /// Get tool name
    pub fn tool_name(&self) -> &str {
        self.tool.name()
    }

    /// Get tool description
    pub fn tool_description(&self) -> &str {
        self.tool.description()
    }

    /// Get all fields
    pub fn fields(&self) -> &[FormField] {
        &self.fields
    }

    /// Get all text inputs
    pub fn inputs(&self) -> &[TextArea<'static>] {
        &self.inputs
    }

    /// Get input modes
    pub fn input_modes(&self) -> &[FieldInputMode] {
        &self.input_modes
    }

    /// Get currently focused field index
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Navigate dropdown up
    pub fn dropdown_prev(&mut self) {
        if let Some(FieldInputMode::Dropdown { selected_index }) = self.input_modes.get_mut(self.focused_index) {
            if let Some(options) = &self.fields[self.focused_index].dropdown_options {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            }
        }
    }

    /// Navigate dropdown down
    pub fn dropdown_next(&mut self) {
        if let Some(FieldInputMode::Dropdown { selected_index }) = self.input_modes.get_mut(self.focused_index) {
            if let Some(options) = &self.fields[self.focused_index].dropdown_options {
                if *selected_index < options.len() - 1 {
                    *selected_index += 1;
                }
            }
        }
    }

    /// Move focus to next field
    pub fn focus_next(&mut self) {
        if !self.fields.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.fields.len();
        }
    }

    /// Move focus to previous field
    pub fn focus_prev(&mut self) {
        if !self.fields.is_empty() {
            self.focused_index = if self.focused_index == 0 {
                self.fields.len() - 1
            } else {
                self.focused_index - 1
            };
        }
    }

    /// Get mutable reference to focused input
    pub fn focused_input_mut(&mut self) -> Option<&mut TextArea<'static>> {
        self.inputs.get_mut(self.focused_index)
    }

    /// Validate the form
    pub fn validate(&self) -> Result<(), String> {
        for (i, field) in self.fields.iter().enumerate() {
            // Get value from dropdown or text input
            let value = match &self.input_modes[i] {
                FieldInputMode::Dropdown { selected_index } => {
                    // Get value from dropdown selection
                    if let Some(options) = &field.dropdown_options {
                        options.get(*selected_index).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                FieldInputMode::Text => {
                    // Get value from text input
                    self.inputs[i].lines().join("")
                }
            };

            if field.required && value.trim().is_empty() {
                return Err(format!("Field '{}' is required", field.name));
            }

            // Type validation
            match field.field_type {
                FieldType::Integer => {
                    if !value.is_empty() && value.parse::<i64>().is_err() {
                        return Err(format!("Field '{}' must be an integer", field.name));
                    }
                }
                FieldType::Number => {
                    if !value.is_empty() && value.parse::<f64>().is_err() {
                        return Err(format!("Field '{}' must be a number", field.name));
                    }
                }
                FieldType::Boolean => {
                    if !value.is_empty() && value != "true" && value != "false" {
                        return Err(format!("Field '{}' must be 'true' or 'false'", field.name));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Build parameters JSON from form inputs
    pub fn build_params(&self) -> Value {
        let mut params = serde_json::Map::new();

        for (i, field) in self.fields.iter().enumerate() {
            // Get value from dropdown or text input
            let value = match &self.input_modes[i] {
                FieldInputMode::Dropdown { selected_index } => {
                    // Get value from dropdown selection
                    if let Some(options) = &field.dropdown_options {
                        options.get(*selected_index).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                FieldInputMode::Text => {
                    // Get value from text input
                    self.inputs[i].lines().join("")
                }
            };

            if value.trim().is_empty() {
                continue; // Skip empty optional fields
            }

            let json_value = match field.field_type {
                FieldType::Integer => {
                    json!(value.parse::<i64>().unwrap_or(0))
                }
                FieldType::Number => {
                    json!(value.parse::<f64>().unwrap_or(0.0))
                }
                FieldType::Boolean => {
                    json!(value == "true")
                }
                FieldType::Array => {
                    // Simple comma-separated array
                    json!(value.split(',').map(|s| s.trim()).collect::<Vec<_>>())
                }
                FieldType::String => {
                    json!(value)
                }
            };

            params.insert(field.name.clone(), json_value);
        }

        Value::Object(params)
    }

    /// Execute the tool with current parameters
    pub async fn execute(&self) -> Result<Value> {
        let params = self.build_params();
        self.tool.execute(params).await.map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Get the tool reference (for async execution)
    pub fn get_tool(&self) -> Arc<dyn PentestTool> {
        self.tool.clone()
    }
}
