# Multi-Capture Comparison

Advanced analysis system for comparing multiple Bluetooth/BLE packet captures to identify similarities, differences, and patterns.

## Overview

The multi-capture comparison system enables:
- **Device Presence Analysis** - Track devices across captures
- **Traffic Pattern Comparison** - Analyze channel usage and packet distribution
- **Similarity Assessment** - Quantify capture similarity with scores
- **Automated Insights** - Generate recommendations based on analysis

## Use Cases

### 1. Replay Attack Detection
Compare pre-attack and attack captures to identify replayed packets.

### 2. Environment Fingerprinting
Analyze device presence patterns across different locations or times.

### 3. Protocol Analysis
Compare captures from different devices to understand protocol variations.

### 4. Performance Testing
Compare captures under different conditions to identify performance issues.

### 5. Security Auditing
Identify unauthorized devices by comparing baseline and current captures.

## Architecture

```
CaptureData (multiple captures)
    ↓
ComparisonEngine::compare_captures()
    ├── Device Presence Analysis
    ├── Traffic Pattern Analysis
    ├── Similarity Calculation
    └── Summary Generation
    ↓
ComparisonResult
```

### Core Components

**ComparisonEngine** (`crates/platform/src/comparison.rs`)
- Multi-capture analysis
- Pairwise similarity matrix
- Automated insight generation

**Data Structures:**
- `ComparisonResult` - Complete comparison output
- `DevicePresenceComparison` - Device analysis
- `TrafficPatternComparison` - Traffic analysis
- `ComparisonSummary` - Key findings and recommendations

## API Reference

### Basic Comparison

```rust
use ubertooth_platform::{ComparisonEngine, CaptureData, PacketInfo, comparison::DeviceInfo};

// Prepare capture data
let captures = vec![
    CaptureData {
        capture_id: "cap-morning".to_string(),
        packets: vec![/* ... */],
        devices: vec![/* ... */],
    },
    CaptureData {
        capture_id: "cap-evening".to_string(),
        packets: vec![/* ... */],
        devices: vec![/* ... */],
    },
];

// Compare captures
let result = ComparisonEngine::compare_captures(captures)?;

// Access results
println!("Common devices: {}", result.device_presence.common_devices.len());
println!("Similarity: {:?}", result.summary.overall_similarity);
```

### Device Presence Analysis

```rust
// Common devices (present in all captures)
for device in &result.device_presence.common_devices {
    println!("Device: {} ({})", device.mac_address, device.name.as_deref().unwrap_or("Unknown"));
    println!("Packets: {}", device.packet_count);
    if let Some(rssi) = device.avg_rssi {
        println!("Avg RSSI: {:.1} dBm", rssi);
    }
}

// Unique devices (per capture)
for (capture_id, devices) in &result.device_presence.unique_devices {
    println!("Unique to {}: {} device(s)", capture_id, devices.len());
    for device in devices {
        println!("  - {}", device.mac_address);
    }
}

// Total unique devices
println!("Total unique: {}", result.device_presence.total_unique_devices);
```

### Traffic Pattern Analysis

```rust
// Channel usage per capture
for (capture_id, usage) in &result.traffic_patterns.channel_usage {
    println!("Capture: {}", capture_id);
    for ch in usage {
        println!("  Ch {}: {} packets ({:.1}%)",
            ch.channel, ch.packet_count, ch.percentage);
    }
}

// Packet type distribution
for (capture_id, types) in &result.traffic_patterns.packet_types {
    println!("Capture: {}", capture_id);
    for pt in types {
        println!("  {}: {} packets ({:.1}%)",
            pt.packet_type, pt.count, pt.percentage);
    }
}

// Temporal patterns
for pattern in &result.traffic_patterns.temporal_patterns {
    println!("Capture: {}", pattern.capture_id);
    println!("  Duration: {:.1}s", pattern.duration_seconds);
    println!("  Rate: {:.1} pps (peak: {:.1})",
        pattern.packets_per_second, pattern.peak_rate);
    println!("  Avg interval: {:.1}ms", pattern.average_interval_ms);
}
```

### Similarity Matrix

```rust
// Pairwise similarity scores
let n = result.capture_ids.len();
for i in 0..n {
    for j in (i + 1)..n {
        let similarity = result.similarity_matrix[i][j];
        println!("{} <-> {}: {:.1}% similar",
            result.capture_ids[i],
            result.capture_ids[j],
            similarity * 100.0
        );
    }
}

// Average similarity
let total: f64 = result.similarity_matrix.iter()
    .enumerate()
    .flat_map(|(i, row)| {
        row.iter().skip(i + 1).copied()
    })
    .sum();
let count = (n * (n - 1)) / 2;
let avg_similarity = if count > 0 { total / count as f64 } else { 0.0 };
println!("Average similarity: {:.1}%", avg_similarity * 100.0);
```

### Comparison Summary

```rust
// Overall similarity
match result.summary.overall_similarity {
    SimilarityLevel::Identical => println!("Captures are virtually identical"),
    SimilarityLevel::VerySimilar => println!("Captures are very similar"),
    SimilarityLevel::Similar => println!("Captures share common patterns"),
    SimilarityLevel::Somewhat => println!("Captures have some similarities"),
    SimilarityLevel::Different => println!("Captures are quite different"),
    SimilarityLevel::VeryDifferent => println!("Captures are very different"),
}

// Key differences
println!("\nKey Differences:");
for diff in &result.summary.key_differences {
    println!("  • {}", diff);
}

// Recommendations
println!("\nRecommendations:");
for rec in &result.summary.recommendations {
    println!("  → {}", rec);
}
```

## Similarity Calculation

### Algorithm

Similarity between two captures is calculated using three metrics:

1. **Device Overlap** - Jaccard similarity of device sets
2. **Channel Overlap** - Jaccard similarity of channel usage
3. **Packet Type Overlap** - Jaccard similarity of packet types

```
Similarity = Average(Device Score, Channel Score, Type Score)

Jaccard(A, B) = |A ∩ B| / |A ∪ B|
```

### Similarity Levels

| Level | Score Range | Description |
|-------|-------------|-------------|
| Identical | ≥ 95% | Virtually identical captures |
| VerySimilar | 80-95% | Very similar, minor differences |
| Similar | 60-80% | Share common patterns |
| Somewhat | 40-60% | Some similarities |
| Different | 20-40% | Quite different |
| VeryDifferent | < 20% | Very different captures |

### Example Scenarios

**Identical (>95%)**
- Same device, same time, duplicate captures
- Replay attacks with minimal variation

**VerySimilar (80-95%)**
- Same device, different times
- Same environment, same day

**Similar (60-80%)**
- Same environment, different days
- Same device type, different instances

**Different (20-40%)**
- Different environments
- Different device types

**VeryDifferent (<20%)**
- Completely different scenarios
- No common devices or patterns

## Statistics Provided

### Capture Statistics

```rust
pub struct CaptureStatistics {
    pub capture_id: String,
    pub total_packets: usize,
    pub unique_devices: usize,
    pub channels_used: usize,
    pub packet_types: usize,
    pub duration_seconds: f64,
    pub avg_rssi: Option<f32>,
}
```

### Device Presence

- **Common Devices** - Present in all captures
- **Unique Devices** - Present in only one capture
- **Total Unique** - All unique devices across captures

### Traffic Patterns

- **Channel Usage** - Distribution per capture
- **Packet Types** - Type distribution per capture
- **Temporal Patterns** - Rates, peaks, intervals

## Integration with bt_compare Tool

The comparison engine enhances the existing `bt_compare` tool:

```rust
// Enhanced bt_compare with multi-capture support
async fn bt_compare_multi(&self, params: Value) -> Result<Value> {
    let capture_ids: Vec<String> = params["capture_ids"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // Load capture data
    let mut captures = Vec::new();
    for id in &capture_ids {
        let data = self.load_capture_data(id)?;
        captures.push(data);
    }

    // Compare
    let result = ComparisonEngine::compare_captures(captures)?;

    // Serialize result
    Ok(serde_json::to_value(&result)?)
}
```

## Output Format

### JSON Response

```json
{
  "capture_ids": ["cap-1", "cap-2", "cap-3"],
  "device_presence": {
    "common_devices": [
      {
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "name": "Device 1",
        "packet_count": 150,
        "avg_rssi": -52.3
      }
    ],
    "unique_devices": {
      "cap-1": [
        {
          "mac_address": "11:22:33:44:55:66",
          "name": null,
          "packet_count": 50,
          "avg_rssi": -60.0
        }
      ]
    },
    "total_unique_devices": 5
  },
  "traffic_patterns": {
    "channel_usage": {
      "cap-1": [
        {"channel": 37, "packet_count": 100, "percentage": 50.0},
        {"channel": 38, "packet_count": 75, "percentage": 37.5},
        {"channel": 39, "packet_count": 25, "percentage": 12.5}
      ]
    },
    "packet_types": {
      "cap-1": [
        {"packet_type": "LE_ADV", "count": 150, "percentage": 75.0},
        {"packet_type": "LE_DATA", "count": 50, "percentage": 25.0}
      ]
    },
    "temporal_patterns": [
      {
        "capture_id": "cap-1",
        "duration_seconds": 60.0,
        "packets_per_second": 3.33,
        "peak_rate": 10.0,
        "average_interval_ms": 300.0
      }
    ]
  },
  "statistics": [
    {
      "capture_id": "cap-1",
      "total_packets": 200,
      "unique_devices": 3,
      "channels_used": 3,
      "packet_types": 2,
      "duration_seconds": 60.0,
      "avg_rssi": -55.5
    }
  ],
  "similarity_matrix": [
    [1.0, 0.85, 0.72],
    [0.85, 1.0, 0.68],
    [0.72, 0.68, 1.0]
  ],
  "summary": {
    "key_differences": [
      "3 common devices found across all captures",
      "2 unique device(s) in capture cap-1",
      "Significant packet count variation: 100 - 300"
    ],
    "overall_similarity": "Similar",
    "recommendations": [
      "Captures show moderate differences - review device presence and traffic patterns",
      "Consider normalizing capture durations for better comparison"
    ]
  }
}
```

## Performance Considerations

### Time Complexity

- **Device Analysis:** O(n × m) where n = captures, m = avg devices per capture
- **Traffic Analysis:** O(n × p) where p = avg packets per capture
- **Similarity Matrix:** O(n²) for n captures

### Memory Usage

```
Memory ≈ (packets × 40 bytes) + (devices × 100 bytes) + similarity_matrix
```

For 3 captures with 1000 packets and 10 devices each:
```
(3 × 1000 × 40) + (3 × 10 × 100) + (3 × 3 × 8) = ~123 KB
```

### Optimization Tips

1. **Limit Capture Count** - Compare 2-4 captures for interactive analysis
2. **Sample Large Captures** - Use sampling for captures with >10K packets
3. **Parallel Analysis** - Device and traffic analysis can run in parallel
4. **Cache Results** - Cache comparison results for frequently accessed pairs

## Use Case Examples

### Example 1: Replay Attack Detection

```rust
// Compare baseline and suspicious captures
let captures = vec![
    load_capture("baseline-2024-01-01"),
    load_capture("suspicious-2024-01-15"),
];

let result = ComparisonEngine::compare_captures(captures)?;

if result.summary.overall_similarity == SimilarityLevel::Identical {
    println!("⚠️  ALERT: Potential replay attack detected!");
    println!("Captures are virtually identical despite time difference");
}

// Check for identical packet sequences
if result.similarity_matrix[0][1] > 0.95 {
    println!("Packet-level similarity: {:.1}%", result.similarity_matrix[0][1] * 100.0);
}
```

### Example 2: Environment Monitoring

```rust
// Compare captures from different times
let captures = vec![
    load_capture("office-morning"),
    load_capture("office-afternoon"),
    load_capture("office-evening"),
];

let result = ComparisonEngine::compare_captures(captures)?;

// Identify persistent devices (present throughout the day)
println!("Persistent devices: {}", result.device_presence.common_devices.len());

// Identify transient devices
for (time, devices) in &result.device_presence.unique_devices {
    println!("{}: {} transient devices", time, devices.len());
}
```

### Example 3: Protocol Compliance

```rust
// Compare captures from compliant and non-compliant devices
let captures = vec![
    load_capture("compliant-device-1"),
    load_capture("compliant-device-2"),
    load_capture("suspect-device"),
];

let result = ComparisonEngine::compare_captures(captures)?;

// Check if suspect device behaves similarly to compliant devices
let suspect_vs_1 = result.similarity_matrix[0][2];
let suspect_vs_2 = result.similarity_matrix[1][2];

if suspect_vs_1 < 0.5 && suspect_vs_2 < 0.5 {
    println!("⚠️  Suspect device exhibits non-compliant behavior");
    println!("Traffic patterns differ significantly from known-good devices");
}
```

## Testing

### Unit Tests

```bash
# Run comparison engine tests
cargo test --package ubertooth-platform --lib comparison

# Expected output:
# test comparison::tests::test_device_presence_common ... ok
# test comparison::tests::test_device_presence_unique ... ok
# test comparison::tests::test_similarity_calculation ... ok
# test comparison::tests::test_similarity_level ... ok
# test comparison::tests::test_traffic_pattern_analysis ... ok
```

### Integration Testing

```rust
// Create test captures
let test_captures = vec![
    create_test_capture("test1", vec!["AA:BB:CC:DD:EE:FF"], 100),
    create_test_capture("test2", vec!["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"], 150),
];

// Run comparison
let result = ComparisonEngine::compare_captures(test_captures)?;

// Assertions
assert_eq!(result.device_presence.common_devices.len(), 1);
assert!(result.similarity_matrix[0][1] > 0.5);
```

## Future Enhancements

- [ ] **Deep Packet Inspection** - Compare packet payloads byte-by-byte
- [ ] **Temporal Alignment** - Auto-align captures by timestamp or event
- [ ] **Statistical Tests** - Chi-square, KS tests for distribution comparison
- [ ] **Machine Learning** - Anomaly detection using ML models
- [ ] **Export Reports** - PDF/HTML report generation
- [ ] **Visualization** - Interactive comparison visualizations
- [ ] **Streaming Comparison** - Real-time comparison with live captures

## References

- [Jaccard Similarity](https://en.wikipedia.org/wiki/Jaccard_index)
- [Bluetooth Core Specification](https://www.bluetooth.com/specifications/specs/core-specification/)
- [Statistical Similarity Measures](https://en.wikipedia.org/wiki/Similarity_measure)

---

**Last Updated:** 2026-03-18
**Phase:** 3.3 - Multi-Capture Comparison
