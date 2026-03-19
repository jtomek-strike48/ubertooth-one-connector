//! Device fingerprinting based on BLE advertising data and packet patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ubertooth_core::error::Result;

/// Device fingerprint with matching indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    /// Manufacturer name (e.g., "Apple Inc.", "Google LLC")
    pub manufacturer: String,
    /// Device type (e.g., "iPhone", "Android Phone", "Fitness Tracker")
    pub device_type: String,
    /// OS version if identifiable
    pub os_version: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Indicators that led to this match
    pub indicators: Vec<String>,
}

/// Device signature for matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSignature {
    /// Signature ID
    pub id: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Device type
    pub device_type: String,
    /// OS family (iOS, Android, etc.)
    pub os_family: Option<String>,
    /// Matching rules
    pub rules: Vec<MatchRule>,
}

/// Rule for matching devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MatchRule {
    /// Match by OUI (first 3 bytes of MAC)
    Oui { prefix: String },
    /// Match by manufacturer data prefix
    ManufacturerData { company_id: u16 },
    /// Match by service UUID presence
    ServiceUuid { uuid: String },
    /// Match by device name pattern
    DeviceName { pattern: String },
    /// Match by advertising interval range
    AdvertisingInterval { min_ms: u16, max_ms: u16 },
    /// Match by TX power value
    TxPower { value: i8 },
    /// Match by flags
    Flags { value: u8 },
}

/// Device fingerprinting engine.
pub struct FingerprintEngine {
    signatures: Vec<DeviceSignature>,
    oui_map: HashMap<String, String>, // OUI -> Manufacturer
}

impl FingerprintEngine {
    /// Create a new fingerprinting engine with default signatures.
    pub fn new() -> Self {
        let signatures = Self::load_default_signatures();
        let oui_map = Self::build_oui_map();

        Self {
            signatures,
            oui_map,
        }
    }

    /// Load signatures from JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let signatures: Vec<DeviceSignature> = serde_json::from_str(json)?;
        let oui_map = Self::build_oui_map();

        Ok(Self {
            signatures,
            oui_map,
        })
    }

    /// Fingerprint a device based on packet data.
    pub fn fingerprint(&self, packet_data: &PacketData) -> Option<DeviceFingerprint> {
        let mut best_match: Option<(DeviceFingerprint, f64)> = None;

        for signature in &self.signatures {
            if let Some(fingerprint) = self.match_signature(signature, packet_data) {
                let score = fingerprint.confidence;

                if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                    best_match = Some((fingerprint, score));
                }
            }
        }

        best_match.map(|(fp, _)| fp)
    }

    /// Match a single signature against packet data.
    fn match_signature(
        &self,
        signature: &DeviceSignature,
        data: &PacketData,
    ) -> Option<DeviceFingerprint> {
        let mut matches = 0;
        let mut indicators = Vec::new();
        let total_rules = signature.rules.len();

        for rule in &signature.rules {
            if self.match_rule(rule, data, &mut indicators) {
                matches += 1;
            }
        }

        if matches == 0 {
            return None;
        }

        let confidence = matches as f64 / total_rules as f64;

        // Require at least 50% match
        if confidence < 0.5 {
            return None;
        }

        Some(DeviceFingerprint {
            manufacturer: signature.manufacturer.clone(),
            device_type: signature.device_type.clone(),
            os_version: signature.os_family.clone(),
            confidence,
            indicators,
        })
    }

    /// Match a single rule.
    fn match_rule(
        &self,
        rule: &MatchRule,
        data: &PacketData,
        indicators: &mut Vec<String>,
    ) -> bool {
        match rule {
            MatchRule::Oui { prefix } => {
                if let Some(ref mac) = data.mac_address {
                    let mac_prefix = mac.replace(':', "").to_uppercase();
                    if mac_prefix.starts_with(&prefix.replace(':', "").to_uppercase()) {
                        indicators.push(format!("OUI match: {}", prefix));
                        return true;
                    }
                }
                false
            }
            MatchRule::ManufacturerData { company_id } => {
                if let Some(ref mfg_data) = data.manufacturer_data {
                    if mfg_data.company_id == *company_id {
                        indicators.push(format!("Manufacturer data: 0x{:04X}", company_id));
                        return true;
                    }
                }
                false
            }
            MatchRule::ServiceUuid { uuid } => {
                if let Some(ref uuids) = data.service_uuids {
                    if uuids.iter().any(|u| u.eq_ignore_ascii_case(uuid)) {
                        indicators.push(format!("Service UUID: {}", uuid));
                        return true;
                    }
                }
                false
            }
            MatchRule::DeviceName { pattern } => {
                if let Some(ref name) = data.device_name {
                    if name.contains(pattern) {
                        indicators.push(format!("Device name pattern: {}", pattern));
                        return true;
                    }
                }
                false
            }
            MatchRule::AdvertisingInterval { min_ms, max_ms } => {
                if let Some(interval) = data.advertising_interval_ms {
                    if interval >= *min_ms && interval <= *max_ms {
                        indicators.push(format!("Advertising interval: {}ms", interval));
                        return true;
                    }
                }
                false
            }
            MatchRule::TxPower { value } => {
                if let Some(tx_power) = data.tx_power {
                    if tx_power == *value {
                        indicators.push(format!("TX power: {}dBm", value));
                        return true;
                    }
                }
                false
            }
            MatchRule::Flags { value } => {
                if let Some(flags) = data.flags {
                    if flags == *value {
                        indicators.push(format!("Flags: 0x{:02X}", value));
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Load default signatures (Apple, Google, Samsung, etc.)
    fn load_default_signatures() -> Vec<DeviceSignature> {
        vec![
            // Apple devices
            DeviceSignature {
                id: "apple-iphone".to_string(),
                manufacturer: "Apple Inc.".to_string(),
                device_type: "iPhone".to_string(),
                os_family: Some("iOS".to_string()),
                rules: vec![
                    MatchRule::ManufacturerData { company_id: 0x004C }, // Apple company ID
                    MatchRule::Oui {
                        prefix: "00:17:F2".to_string(),
                    }, // Apple OUI
                ],
            },
            DeviceSignature {
                id: "apple-airpods".to_string(),
                manufacturer: "Apple Inc.".to_string(),
                device_type: "AirPods".to_string(),
                os_family: None,
                rules: vec![
                    MatchRule::ManufacturerData { company_id: 0x004C },
                    MatchRule::DeviceName {
                        pattern: "AirPods".to_string(),
                    },
                ],
            },
            DeviceSignature {
                id: "apple-watch".to_string(),
                manufacturer: "Apple Inc.".to_string(),
                device_type: "Apple Watch".to_string(),
                os_family: Some("watchOS".to_string()),
                rules: vec![
                    MatchRule::ManufacturerData { company_id: 0x004C },
                    MatchRule::DeviceName {
                        pattern: "Watch".to_string(),
                    },
                ],
            },
            // Google/Android devices
            DeviceSignature {
                id: "google-pixel".to_string(),
                manufacturer: "Google LLC".to_string(),
                device_type: "Pixel Phone".to_string(),
                os_family: Some("Android".to_string()),
                rules: vec![
                    MatchRule::Oui {
                        prefix: "F4:F5:E8".to_string(),
                    }, // Google OUI
                    MatchRule::DeviceName {
                        pattern: "Pixel".to_string(),
                    },
                ],
            },
            // Samsung devices
            DeviceSignature {
                id: "samsung-galaxy".to_string(),
                manufacturer: "Samsung Electronics".to_string(),
                device_type: "Galaxy Phone".to_string(),
                os_family: Some("Android".to_string()),
                rules: vec![
                    MatchRule::Oui {
                        prefix: "C8:F2:30".to_string(),
                    }, // Samsung OUI
                ],
            },
            DeviceSignature {
                id: "samsung-galaxy-watch".to_string(),
                manufacturer: "Samsung Electronics".to_string(),
                device_type: "Galaxy Watch".to_string(),
                os_family: Some("Wear OS".to_string()),
                rules: vec![
                    MatchRule::Oui {
                        prefix: "C8:F2:30".to_string(),
                    },
                    MatchRule::DeviceName {
                        pattern: "Galaxy Watch".to_string(),
                    },
                ],
            },
            // Fitness trackers
            DeviceSignature {
                id: "fitbit-tracker".to_string(),
                manufacturer: "Fitbit".to_string(),
                device_type: "Fitness Tracker".to_string(),
                os_family: None,
                rules: vec![
                    MatchRule::Oui {
                        prefix: "D4:E8:B2".to_string(),
                    }, // Fitbit OUI
                    MatchRule::DeviceName {
                        pattern: "Fitbit".to_string(),
                    },
                ],
            },
            // Xiaomi devices
            DeviceSignature {
                id: "xiaomi-miband".to_string(),
                manufacturer: "Xiaomi".to_string(),
                device_type: "Mi Band".to_string(),
                os_family: None,
                rules: vec![
                    MatchRule::Oui {
                        prefix: "C8:0F:10".to_string(),
                    }, // Xiaomi OUI
                    MatchRule::DeviceName {
                        pattern: "Mi Band".to_string(),
                    },
                ],
            },
        ]
    }

    /// Build OUI to manufacturer mapping.
    fn build_oui_map() -> HashMap<String, String> {
        let mut map = HashMap::new();

        // Major manufacturers
        map.insert("00:17:F2".to_string(), "Apple Inc.".to_string());
        map.insert("F4:F5:E8".to_string(), "Google LLC".to_string());
        map.insert("C8:F2:30".to_string(), "Samsung Electronics".to_string());
        map.insert("D4:E8:B2".to_string(), "Fitbit".to_string());
        map.insert("C8:0F:10".to_string(), "Xiaomi".to_string());
        map.insert("00:1B:63".to_string(), "Apple Inc.".to_string());
        map.insert("00:26:BB".to_string(), "Apple Inc.".to_string());

        map
    }

    /// Lookup manufacturer by OUI.
    pub fn lookup_manufacturer(&self, mac_address: &str) -> Option<String> {
        let mac_clean = mac_address.replace(':', "").to_uppercase();
        if mac_clean.len() < 6 {
            return None;
        }

        let oui = format!(
            "{}:{}:{}",
            &mac_clean[0..2],
            &mac_clean[2..4],
            &mac_clean[4..6]
        );

        self.oui_map.get(&oui).cloned()
    }
}

impl Default for FingerprintEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet data for fingerprinting.
#[derive(Debug, Clone, Default)]
pub struct PacketData {
    /// MAC address
    pub mac_address: Option<String>,
    /// Device name from advertising
    pub device_name: Option<String>,
    /// Manufacturer-specific data
    pub manufacturer_data: Option<ManufacturerData>,
    /// Service UUIDs
    pub service_uuids: Option<Vec<String>>,
    /// Advertising interval in milliseconds
    pub advertising_interval_ms: Option<u16>,
    /// TX power level
    pub tx_power: Option<i8>,
    /// Advertising flags
    pub flags: Option<u8>,
}

/// Manufacturer-specific data.
#[derive(Debug, Clone)]
pub struct ManufacturerData {
    /// Company identifier (Bluetooth SIG assigned)
    pub company_id: u16,
    /// Raw manufacturer data
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_apple_iphone() {
        let engine = FingerprintEngine::new();

        let packet = PacketData {
            mac_address: Some("00:17:F2:11:22:33".to_string()),
            manufacturer_data: Some(ManufacturerData {
                company_id: 0x004C,
                data: vec![0x02, 0x01, 0x05],
            }),
            ..Default::default()
        };

        let result = engine.fingerprint(&packet);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.manufacturer, "Apple Inc.");
        assert_eq!(fp.device_type, "iPhone");
        assert!(fp.confidence >= 0.5);
    }

    #[test]
    fn test_fingerprint_airpods() {
        let engine = FingerprintEngine::new();

        let packet = PacketData {
            device_name: Some("AirPods Pro".to_string()),
            manufacturer_data: Some(ManufacturerData {
                company_id: 0x004C,
                data: vec![],
            }),
            ..Default::default()
        };

        let result = engine.fingerprint(&packet);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.device_type, "AirPods");
        assert!(fp.indicators.len() > 0);
    }

    #[test]
    fn test_lookup_manufacturer() {
        let engine = FingerprintEngine::new();

        let mfg = engine.lookup_manufacturer("00:17:F2:11:22:33");
        assert_eq!(mfg, Some("Apple Inc.".to_string()));

        let mfg2 = engine.lookup_manufacturer("C8:F2:30:AA:BB:CC");
        assert_eq!(mfg2, Some("Samsung Electronics".to_string()));

        let unknown = engine.lookup_manufacturer("FF:FF:FF:AA:BB:CC");
        assert_eq!(unknown, None);
    }

    #[test]
    fn test_match_rule_oui() {
        let engine = FingerprintEngine::new();
        let mut indicators = Vec::new();

        let rule = MatchRule::Oui {
            prefix: "00:17:F2".to_string(),
        };

        let packet = PacketData {
            mac_address: Some("00:17:F2:11:22:33".to_string()),
            ..Default::default()
        };

        assert!(engine.match_rule(&rule, &packet, &mut indicators));
        assert_eq!(indicators.len(), 1);
    }

    #[test]
    fn test_confidence_scoring() {
        let engine = FingerprintEngine::new();

        // Full match (both rules)
        let packet1 = PacketData {
            mac_address: Some("00:17:F2:11:22:33".to_string()),
            manufacturer_data: Some(ManufacturerData {
                company_id: 0x004C,
                data: vec![],
            }),
            ..Default::default()
        };

        let result1 = engine.fingerprint(&packet1).unwrap();
        assert!(result1.confidence >= 0.9); // Both rules match

        // Partial match (only OUI)
        let packet2 = PacketData {
            mac_address: Some("00:17:F2:11:22:33".to_string()),
            ..Default::default()
        };

        let result2 = engine.fingerprint(&packet2).unwrap();
        assert!(result2.confidence >= 0.5 && result2.confidence < 0.9); // Only one rule
    }
}
