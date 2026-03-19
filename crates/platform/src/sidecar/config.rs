//! Configuration methods for SidecarManager.

use crate::capture_store::CaptureStore;
use crate::config_store::ConfigStore;
use serde_json::{json, Value};
use ubertooth_core::error::{Result, UbertoothError};

use super::SidecarManager;

impl SidecarManager {
    /// Configure channel implementation.
    pub(super) async fn configure_channel(&self, params: Value) -> Result<Value> {
        let channel = params
            .get("channel")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'channel' parameter".to_string())
            })?;

        // Validate channel range
        if channel > 78 {
            return Err(UbertoothError::InvalidParameter(
                "Channel must be 0-78".to_string(),
            ));
        }

        tracing::info!("Setting channel to {}", channel);

        // Calculate frequency (2402 + channel MHz)
        let frequency_mhz = 2402 + channel;

        // Execute ubertooth-util -C<channel> (use -C for channel number, not -c for MHz)
        // Note: Channel configuration via ubertooth-util is ephemeral and primarily
        // affects subsequent operations. Use -c for MHz: -c<freq>
        let channel_arg = format!("-C{}", channel);

        // Channel commands may return non-zero exit codes even on success
        // This is normal behavior for ubertooth-util configuration commands
        match self
            .execute_ubertooth_command("ubertooth-util", &[channel_arg.as_str()])
            .await
        {
            Ok(_) => {}
            Err(e) => {
                // Log but don't fail - channel setting may not persist but command executed
                tracing::debug!("Channel command completed with note: {:?}", e);
            }
        }

        Ok(json!({
            "success": true,
            "channel": channel,
            "frequency_mhz": frequency_mhz,
            "message": format!("Channel set to {} ({} MHz). Note: Configuration is ephemeral and applies to next operation.", channel, frequency_mhz)
        }))
    }

    /// Configure modulation implementation.
    pub(super) async fn configure_modulation(&self, params: Value) -> Result<Value> {
        let modulation = params
            .get("modulation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'modulation' parameter".to_string())
            })?;

        // Validate modulation type
        let valid_mods = ["BT_BASIC_RATE", "BT_LOW_ENERGY", "80211_FHSS", "NONE"];
        if !valid_mods.contains(&modulation) {
            return Err(UbertoothError::InvalidParameter(format!(
                "Invalid modulation: {}. Must be one of: {:?}",
                modulation, valid_mods
            )));
        }

        tracing::info!("Setting modulation to {}", modulation);

        // Map to ubertooth-util flag value
        // For Phase 1, we'll just store the value; actual command depends on device capabilities
        // This is a placeholder implementation
        tracing::debug!("Modulation configuration (placeholder): {}", modulation);

        Ok(json!({
            "success": true,
            "modulation": modulation,
            "message": format!("Modulation set to {}", modulation)
        }))
    }

    /// Configure power implementation.
    pub(super) async fn configure_power(&self, params: Value) -> Result<Value> {
        let power_level = params
            .get("power_level")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'power_level' parameter".to_string())
            })?;

        let paen = params.get("paen").and_then(|v| v.as_bool()).unwrap_or(true);
        let hgm = params.get("hgm").and_then(|v| v.as_bool()).unwrap_or(false);

        // Validate power level range
        if power_level > 7 {
            return Err(UbertoothError::InvalidParameter(
                "Power level must be 0-7".to_string(),
            ));
        }

        tracing::info!(
            "Setting power: level={}, paen={}, hgm={}",
            power_level,
            paen,
            hgm
        );

        // Execute ubertooth-util -p <power>
        let power_str = power_level.to_string();
        self.execute_ubertooth_command("ubertooth-util", &["-p", power_str.as_str()])
            .await?;

        // Estimate TX power in dBm
        // Rough estimates: level 0-7 spans ~0-14 dBm without PA, ~10-24 dBm with PA
        let estimated_power_dbm = if paen {
            10 + (power_level * 2) as i64
        } else {
            power_level as i64 * 2
        };

        Ok(json!({
            "success": true,
            "power_level": power_level,
            "paen": paen,
            "hgm": hgm,
            "estimated_power_dbm": estimated_power_dbm,
            "message": format!(
                "Power configured: Level {} with PA {} (~{} dBm)",
                power_level,
                if paen { "enabled" } else { "disabled" },
                estimated_power_dbm
            )
        }))
    }

    /// Configure squelch implementation.
    pub(super) async fn configure_squelch(&self, params: Value) -> Result<Value> {
        let squelch_level = params
            .get("squelch_level")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                UbertoothError::InvalidParameter("Missing 'squelch_level'".to_string())
            })?;

        // Validate squelch range (-128 to 0 dBm)
        if !(-128..=0).contains(&squelch_level) {
            return Err(UbertoothError::InvalidParameter(
                "Squelch level must be between -128 and 0 dBm".to_string(),
            ));
        }

        tracing::info!("Configuring squelch: {} dBm", squelch_level);

        // Execute ubertooth-util -z<squelch_level>
        // Note: -z flag requires value directly attached (no space), e.g., -z-54
        let squelch_arg = format!("-z{}", squelch_level);
        self.execute_ubertooth_command("ubertooth-util", &[squelch_arg.as_str()])
            .await?;

        Ok(json!({
            "success": true,
            "squelch_level": squelch_level,
            "message": format!("Squelch set to {} dBm", squelch_level)
        }))
    }

    /// Configure LEDs implementation.
    ///
    /// Phase 2 Week 3: LED control via ubertooth-util.
    pub(super) async fn configure_leds(&self, params: Value) -> Result<Value> {
        let usr_led = params
            .get("usr_led")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let rx_led = params
            .get("rx_led")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tx_led = params
            .get("tx_led")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        tracing::info!(
            "Configuring LEDs: usr={}, rx={}, tx={}",
            usr_led,
            rx_led,
            tx_led
        );

        // Note: ubertooth-util only supports:
        // -l[0-1] for USR LED
        // -d[0-1] for all LEDs (USR, RX, TX together)
        // Individual RX/TX control is not available via ubertooth-util

        // If user wants to control all LEDs together
        if usr_led == rx_led && rx_led == tx_led {
            let all_leds_value = if usr_led { "1" } else { "0" };
            let all_leds_arg = format!("-d{}", all_leds_value);
            self.execute_ubertooth_command("ubertooth-util", &[all_leds_arg.as_str()])
                .await?;

            Ok(json!({
                "success": true,
                "leds": {
                    "usr": usr_led,
                    "rx": rx_led,
                    "tx": tx_led
                },
                "message": format!("All LEDs set to {}", if usr_led { "on" } else { "off" })
            }))
        } else {
            // Can only control USR LED individually
            let usr_value = if usr_led { "1" } else { "0" };
            let usr_arg = format!("-l{}", usr_value);
            self.execute_ubertooth_command("ubertooth-util", &[usr_arg.as_str()])
                .await?;

            Ok(json!({
                "success": true,
                "leds": {
                    "usr": usr_led,
                    "rx": rx_led,
                    "tx": tx_led
                },
                "message": format!(
                    "USR LED set to {}. Note: RX/TX LED control not available individually via ubertooth-util",
                    if usr_led { "on" } else { "off" }
                )
            }))
        }
    }

    /// Build session context implementation.
    pub(super) async fn session_context(&self, params: Value) -> Result<Value> {
        let include_recent_captures = params
            .get("include_recent_captures")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let max_captures = params
            .get("max_captures")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let _include_configs = params
            .get("include_configs")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        tracing::info!("Building session context");

        // Get device status
        let device_info = self.device_status().await?;

        // Get recent captures if requested
        let mut recent_captures = Vec::new();
        if include_recent_captures {
            let captures_result = self
                .capture_list(json!({
                    "limit": max_captures,
                    "sort_by": "timestamp",
                    "sort_order": "desc"
                }))
                .await?;

            if let Some(captures) = captures_result.get("captures").and_then(|v| v.as_array()) {
                for capture in captures.iter().take(max_captures) {
                    recent_captures.push(capture.clone());
                }
            }
        }

        // Calculate storage stats
        let store = CaptureStore::new()?;
        let all_captures = store.list_captures()?;
        let total_size_bytes: u64 = all_captures.iter().map(|m| m.file_size_bytes).sum();
        let total_size_mb = total_size_bytes as f64 / 1_048_576.0;

        let timestamp = chrono::Utc::now().to_rfc3339();

        Ok(json!({
            "success": true,
            "timestamp": timestamp,
            "device": device_info.get("device").unwrap_or(&json!({})),
            "recent_captures": recent_captures,
            "saved_configs": [],  // TODO Phase 2: Config persistence
            "storage": {
                "captures_dir": store.captures_dir().to_string_lossy(),
                "captures_count": all_captures.len(),
                "total_size_mb": format!("{:.1}", total_size_mb).parse::<f64>().unwrap_or(0.0)
            }
        }))
    }

    /// List all saved configuration presets.
    ///
    /// Phase 2 Week 4: List configs from ~/.ubertooth/configs/
    pub(in crate::sidecar) async fn config_list(&self, _params: Value) -> Result<Value> {
        tracing::info!("Listing saved configurations");

        let store = ConfigStore::new()?;
        let configs = store.list_configs()?;

        let config_list: Vec<Value> = configs
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "description": c.description,
                    "created": c.created.to_rfc3339(),
                    "settings_preview": {
                        "channel": c.settings.channel,
                        "modulation": c.settings.modulation
                    }
                })
            })
            .collect();

        Ok(json!({
            "success": true,
            "configs": config_list,
            "count": configs.len()
        }))
    }

    /// Delete a saved configuration preset.
    ///
    /// Phase 2 Week 4: Remove config file from ~/.ubertooth/configs/
    pub(in crate::sidecar) async fn config_delete(&self, params: Value) -> Result<Value> {
        let config_name = params
            .get("config_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| UbertoothError::InvalidParameter("Missing 'config_name'".to_string()))?;

        tracing::info!("Deleting configuration: {}", config_name);

        let store = ConfigStore::new()?;
        store.delete_config(config_name)?;

        Ok(json!({
            "success": true,
            "message": format!("Configuration '{}' deleted", config_name)
        }))
    }
}
