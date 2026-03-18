//! Test utilities and fixtures for integration tests.

use chrono::Utc;
use std::path::PathBuf;
use tempfile::TempDir;

/// Capture metadata structure for testing.
#[derive(Debug, Clone)]
pub struct CaptureMetadata {
    pub capture_id: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub capture_type: String,
    pub packet_count: usize,
    pub duration_sec: Option<u64>,
    pub file_size_bytes: u64,
    pub pcap_path: String,
    pub tags: Vec<String>,
    pub description: String,
}

/// Capture store for testing.
pub struct CaptureStore {
    pub base_path: PathBuf,
}

impl CaptureStore {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(base_path.join("captures")).ok();
        std::fs::create_dir_all(base_path.join("configs")).ok();
        Self { base_path }
    }

    pub fn captures_dir(&self) -> PathBuf {
        self.base_path.join("captures")
    }

    pub fn configs_dir(&self) -> PathBuf {
        self.base_path.join("configs")
    }

    pub fn save_metadata(&self, metadata: &CaptureMetadata) -> Result<(), std::io::Error> {
        let path = self.captures_dir().join(format!("{}.json", metadata.capture_id));
        let json = serde_json::to_string_pretty(metadata)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_metadata(&self, capture_id: &str) -> Result<CaptureMetadata, std::io::Error> {
        let path = self.captures_dir().join(format!("{}.json", capture_id));
        let json = std::fs::read_to_string(path)?;
        let metadata: CaptureMetadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    pub fn list_captures(&self) -> Result<Vec<CaptureMetadata>, std::io::Error> {
        let mut captures = Vec::new();
        for entry in std::fs::read_dir(self.captures_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json = std::fs::read_to_string(&path)?;
                if let Ok(metadata) = serde_json::from_str::<CaptureMetadata>(&json) {
                    captures.push(metadata);
                }
            }
        }
        Ok(captures)
    }

    pub fn generate_capture_id(prefix: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("cap-{}-{}", prefix, timestamp)
    }
}

impl serde::Serialize for CaptureMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CaptureMetadata", 9)?;
        state.serialize_field("capture_id", &self.capture_id)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("capture_type", &self.capture_type)?;
        state.serialize_field("packet_count", &self.packet_count)?;
        state.serialize_field("duration_sec", &self.duration_sec)?;
        state.serialize_field("file_size_bytes", &self.file_size_bytes)?;
        state.serialize_field("pcap_path", &self.pcap_path)?;
        state.serialize_field("tags", &self.tags)?;
        state.serialize_field("description", &self.description)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for CaptureMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Helper {
            capture_id: String,
            timestamp: chrono::DateTime<Utc>,
            capture_type: String,
            packet_count: usize,
            duration_sec: Option<u64>,
            file_size_bytes: u64,
            pcap_path: String,
            tags: Vec<String>,
            description: String,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(CaptureMetadata {
            capture_id: helper.capture_id,
            timestamp: helper.timestamp,
            capture_type: helper.capture_type,
            packet_count: helper.packet_count,
            duration_sec: helper.duration_sec,
            file_size_bytes: helper.file_size_bytes,
            pcap_path: helper.pcap_path,
            tags: helper.tags,
            description: helper.description,
        })
    }
}

/// Test fixture for CaptureStore with temporary directory.
pub struct CaptureStoreFixture {
    pub temp_dir: TempDir,
    pub store: CaptureStore,
}

impl CaptureStoreFixture {
    /// Create a new test fixture with temporary storage.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = CaptureStore::new(temp_dir.path().to_path_buf());
        Self { temp_dir, store }
    }

    /// Get the path to the captures directory.
    pub fn captures_dir(&self) -> PathBuf {
        self.store.captures_dir()
    }

    /// Create a sample capture metadata for testing.
    pub fn create_sample_metadata(&self, capture_type: &str, packet_count: usize) -> CaptureMetadata {
        CaptureMetadata {
            capture_id: CaptureStore::generate_capture_id(capture_type),
            timestamp: Utc::now(),
            capture_type: capture_type.to_string(),
            packet_count,
            duration_sec: Some(60),
            file_size_bytes: 1024,
            pcap_path: format!("/tmp/test-{}.pcap", capture_type),
            tags: vec!["test".to_string()],
            description: format!("Test capture of type {}", capture_type),
        }
    }

    /// Save multiple test captures.
    pub fn seed_captures(&self, count: usize) -> Vec<CaptureMetadata> {
        let mut captures = Vec::new();

        for i in 0..count {
            let metadata = self.create_sample_metadata(&format!("ble-scan-{}", i), 10 + i * 5);
            self.store.save_metadata(&metadata).expect("Failed to save metadata");
            captures.push(metadata);
        }

        captures
    }
}

/// Mock USB device for testing without hardware.
pub struct MockUsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub firmware_version: String,
}

impl MockUsbDevice {
    pub fn ubertooth_one() -> Self {
        Self {
            vendor_id: 0x1d50,
            product_id: 0x6002,
            firmware_version: "2020-12-R1".to_string(),
        }
    }

    pub fn is_ubertooth(&self) -> bool {
        self.vendor_id == 0x1d50 && self.product_id == 0x6002
    }
}

/// Create a temporary PCAP file with sample data.
pub fn create_test_pcap(path: &PathBuf, packet_count: usize) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;

    // Write minimal PCAP header
    let header = [
        0xd4, 0xc3, 0xb2, 0xa1, // Magic number
        0x02, 0x00, 0x04, 0x00, // Version 2.4
        0x00, 0x00, 0x00, 0x00, // Timezone
        0x00, 0x00, 0x00, 0x00, // Accuracy
        0xff, 0xff, 0x00, 0x00, // Snaplen
        0x01, 0x00, 0x00, 0x00, // Network
    ];
    file.write_all(&header)?;

    // Write dummy packets
    for _ in 0..packet_count {
        // Packet header (16 bytes)
        let packet_header = [
            0x00, 0x00, 0x00, 0x00, // Timestamp seconds
            0x00, 0x00, 0x00, 0x00, // Timestamp microseconds
            0x10, 0x00, 0x00, 0x00, // Captured length
            0x10, 0x00, 0x00, 0x00, // Original length
        ];
        file.write_all(&packet_header)?;

        // Dummy packet data (16 bytes)
        let packet_data = [0xAA; 16];
        file.write_all(&packet_data)?;
    }

    Ok(())
}

/// Assert that a result is an error containing the expected message.
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => {
                let error_msg = format!("{:?}", e);
                assert!(
                    error_msg.contains($expected),
                    "Error message '{}' does not contain '{}'",
                    error_msg,
                    $expected
                );
            }
        }
    };
}
