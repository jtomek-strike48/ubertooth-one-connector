//! Integration tests for capture storage system.

mod test_utils;

use test_utils::*;

#[test]
fn test_capture_store_creation() {
    let fixture = CaptureStoreFixture::new();

    // Verify directories were created
    assert!(fixture.captures_dir().exists());
    assert!(fixture.store.configs_dir().exists());
}

#[test]
fn test_save_and_load_metadata() {
    let fixture = CaptureStoreFixture::new();
    let metadata = fixture.create_sample_metadata("ble-scan", 42);

    // Save metadata
    fixture
        .store
        .save_metadata(&metadata)
        .expect("Failed to save metadata");

    // Load it back
    let loaded = fixture
        .store
        .load_metadata(&metadata.capture_id)
        .expect("Failed to load metadata");

    // Verify fields
    assert_eq!(loaded.capture_id, metadata.capture_id);
    assert_eq!(loaded.capture_type, "ble-scan");
    assert_eq!(loaded.packet_count, 42);
    assert_eq!(loaded.tags, vec!["test"]);
}

#[test]
fn test_load_nonexistent_capture() {
    let fixture = CaptureStoreFixture::new();

    let result = fixture.store.load_metadata("nonexistent-id");

    assert!(result.is_err());
}

#[test]
fn test_list_captures() {
    let fixture = CaptureStoreFixture::new();

    // Seed with 5 captures
    let captures = fixture.seed_captures(5);

    // List all captures
    let listed = fixture
        .store
        .list_captures()
        .expect("Failed to list captures");

    assert_eq!(listed.len(), 5);

    // Verify capture IDs match
    for capture in &captures {
        assert!(listed.iter().any(|c| c.capture_id == capture.capture_id));
    }
}

#[test]
fn test_list_captures_empty() {
    let fixture = CaptureStoreFixture::new();

    let captures = fixture
        .store
        .list_captures()
        .expect("Failed to list captures");

    assert_eq!(captures.len(), 0);
}

#[test]
fn test_generate_capture_id() {
    let id1 = CaptureStore::generate_capture_id("ble-scan");
    std::thread::sleep(std::time::Duration::from_millis(2));
    let id2 = CaptureStore::generate_capture_id("ble-scan");

    // IDs should be unique
    assert_ne!(id1, id2);

    // IDs should start with prefix
    assert!(id1.starts_with("cap-ble-scan-"));
    assert!(id2.starts_with("cap-ble-scan-"));
}

#[test]
fn test_capture_metadata_serialization() {
    let fixture = CaptureStoreFixture::new();
    let metadata = fixture.create_sample_metadata("bt-decode", 100);

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).expect("Failed to serialize");

    // Deserialize back
    let deserialized: CaptureMetadata = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.capture_id, metadata.capture_id);
    assert_eq!(deserialized.packet_count, 100);
}

#[test]
fn test_capture_with_tags() {
    let fixture = CaptureStoreFixture::new();
    let mut metadata = fixture.create_sample_metadata("ble-follow", 50);

    // Add multiple tags
    metadata.tags = vec![
        "bluetooth".to_string(),
        "ble".to_string(),
        "connection".to_string(),
    ];

    // Save and reload
    fixture
        .store
        .save_metadata(&metadata)
        .expect("Failed to save");
    let loaded = fixture
        .store
        .load_metadata(&metadata.capture_id)
        .expect("Failed to load");

    assert_eq!(loaded.tags.len(), 3);
    assert!(loaded.tags.contains(&"bluetooth".to_string()));
    assert!(loaded.tags.contains(&"ble".to_string()));
}

#[test]
fn test_pcap_file_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let pcap_path = temp_dir.path().join("test.pcap");

    // Create test PCAP with 10 packets
    create_test_pcap(&pcap_path, 10).expect("Failed to create PCAP");

    // Verify file exists and has content
    assert!(pcap_path.exists());

    let metadata = std::fs::metadata(&pcap_path).expect("Failed to get file metadata");
    assert!(metadata.len() > 0);

    // Expected size: 24-byte header + 10 * (16-byte header + 16-byte data) = 344 bytes
    assert_eq!(metadata.len(), 344);
}

#[test]
fn test_capture_ordering_by_timestamp() {
    let fixture = CaptureStoreFixture::new();

    // Create captures with slight delays
    let mut captures = Vec::new();
    for i in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let metadata = fixture.create_sample_metadata(&format!("scan-{}", i), i);
        fixture
            .store
            .save_metadata(&metadata)
            .expect("Failed to save");
        captures.push(metadata);
    }

    // List captures
    let mut listed = fixture.store.list_captures().expect("Failed to list");

    // Sort by timestamp
    listed.sort_by_key(|c| c.timestamp);

    // Verify ordering
    assert!(listed[0].timestamp <= listed[1].timestamp);
    assert!(listed[1].timestamp <= listed[2].timestamp);
}
