//! Type definitions for sidecar operations.

/// PCAP analysis results structure.
#[derive(Debug)]
pub(super) struct PcapAnalysis {
    pub packet_count: usize,
    pub total_bytes: usize,
    pub duration_sec: f64,
    pub packets_per_sec: f64,
    pub avg_packet_size: f64,
    pub devices: Vec<BleDevice>,
    pub timing: TimingAnalysis,
    pub security: SecurityAnalysis,
}

/// Timing analysis results.
#[derive(Debug)]
pub(super) struct TimingAnalysis {
    pub avg_interval_ms: f64,
    pub min_interval_ms: f64,
    pub max_interval_ms: f64,
    pub intervals_count: usize,
}

/// Security observation from analysis.
#[derive(Debug, Clone)]
pub(super) struct SecurityObservation {
    pub observation_type: String,
    pub severity: String,
    pub description: String,
    pub affected_device: Option<String>,
}

/// Security analysis results.
#[derive(Debug)]
pub(super) struct SecurityAnalysis {
    pub observations: Vec<SecurityObservation>,
    pub privacy_enabled_count: usize,
    pub public_address_count: usize,
    pub connection_requests: usize,
    pub scan_requests: usize,
}

/// BLE device information extracted from packets.
#[derive(Debug, Clone)]
pub(super) struct BleDevice {
    pub mac_address: String,
    pub name: Option<String>,
    pub rssi: i8,
    pub pdu_type: String,
    pub first_seen: f64,
    pub last_seen: f64,
    pub packet_count: usize,
}
