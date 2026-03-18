#Visualizations

Advanced visualization capabilities for analyzing Bluetooth packet captures.

## Overview

The visualization system provides two complementary views:
1. **Channel Heatmap** - Spatial visualization of channel activity over time
2. **Packet Timeline** - Temporal visualization with packet distribution

Both views support real-time updates and historical analysis of captured data.

## Architecture

### Components

```
apps/cli/src/tui/views/
├── heatmap.rs      - Channel activity heatmap
└── timeline.rs     - Packet timeline and statistics
```

### Data Flow

```
PacketData (StreamingBuffer)
    ↓
HeatmapData / TimelineData (aggregation)
    ↓
Ratatui widgets (rendering)
    ↓
Terminal display
```

## Channel Heatmap

Visualizes channel activity as a 2D grid where:
- **X-axis:** Time bins
- **Y-axis:** Channel numbers
- **Color/Intensity:** Packet count

### Features

**Color Schemes:**
- **Heat** (default) - Black → Blue → Yellow → Red
- **Cool** - Black → Blue → Cyan → White
- **Activity** - Black → Green → Yellow → Light Yellow
- **Grayscale** - Black → Dark Gray → Gray → White

**Intensity Characters:**
```
 (space)  - 0-10% intensity
.         - 10-20%
:         - 20-30%
-         - 30-40%
=         - 40-50%
+         - 50-60%
*         - 60-70%
#         - 70-80%
%         - 80-90%
@         - 90-100%
```

**Statistics:**
- Total packets
- Active channels
- Maximum intensity
- Busiest channel

### Configuration

```rust
use ubertooth_cli::tui::views::{HeatmapConfig, ColorScheme};

let config = HeatmapConfig {
    time_window_secs: 60,           // 60-second window
    time_buckets: 60,               // 60 time bins
    channel_range: (0, 78),         // BLE channels 0-78
    color_scheme: ColorScheme::Heat,
};
```

### Usage

```rust
use ubertooth_cli::tui::views::{render_heatmap, HeatmapConfig};
use ubertooth_platform::PacketData;

// Prepare packet data
let packets: Vec<PacketData> = buffer.get_recent_packets(1000)?;

// Render heatmap
render_heatmap(
    &mut frame,
    area,
    &packets,
    &HeatmapConfig::default(),
    "Channel Activity Heatmap"
);
```

### Example Output

```
┌─ Heatmap ────────────────────────────────────┐
│ Channel Activity Heatmap                     │
│ Packets: 1234  Active Channels: 15  Max: 89 │
├──────────────────────────────────────────────┤
│ Ch78 .....:::::-----=====+++++****####%%%%@@@│
│ Ch75 .........::::::-----=====+++++*****#### │
│ Ch72 ............:::-----====++++****####%%% │
│ Ch69 ................----====+++****####%%%@ │
│ ...                                          │
│ Ch3  ........................................│
│ Ch0  ........................................│
└──────────────────────────────────────────────┘
┌─ Legend & Stats ─────────────────────────────┐
│ Intensity:   Low  == Medium  @@ High         │
│ Color Scheme: Heat                           │
│ Busiest Channel: Ch 37 (456 packets)         │
└──────────────────────────────────────────────┘
```

### Channel Ranges

**BLE Channels:**
- Advertising: 37, 38, 39 (2402, 2426, 2480 MHz)
- Data: 0-36 (2404-2478 MHz, skipping advertising)

**BLE Frequency Mapping:**
```
Channel = (Frequency_MHz - 2400) / 2
Frequency_MHz = 2400 + (Channel * 2)
```

## Packet Timeline

Visualizes packet distribution over time as a bar chart with statistics.

### Features

**Timeline Bins:**
- Configurable time window (default: 60 seconds)
- Adjustable number of bins (default: 30)
- Channel distribution per bin
- Average RSSI per bin

**Statistics:**
- Total packets
- Peak rate (packets/bin)
- Average rate (packets/bin)
- Unique channels
- Average RSSI

**Color Coding:**
- **Red** - High activity (>75% of max)
- **Yellow** - Medium-high (50-75%)
- **Green** - Medium (25-50%)
- **Blue** - Low (<25%)

### Configuration

```rust
use ubertooth_cli::tui::views::TimelineConfig;

let config = TimelineConfig {
    time_window_secs: 60,
    time_bins: 30,
    show_details: true,
    group_by_channel: false,
};
```

### Usage

```rust
use ubertooth_cli::tui::views::{render_timeline, TimelineConfig};

// Render timeline
render_timeline(
    &mut frame,
    area,
    &packets,
    &TimelineConfig::default(),
    "Packet Timeline"
);
```

### Example Output

```
┌─ Timeline ───────────────────────────────────┐
│ Packet Timeline                              │
│ Period: 14:32:00 - 14:33:00  Total: 1234    │
├──────────────────────────────────────────────┤
│ Packets Over Time                            │
│                                              │
│    ┌─█                                       │
│  █ │ █   █                                   │
│  █ │ █   █       █                           │
│  █ │ █ █ █   █   █   █       █               │
│  █ │ █ █ █ █ █ █ █ █ █   █   █               │
│  █ │ █ █ █ █ █ █ █ █ █ █ █ █ █   █           │
│  0s   5s  10s 15s 20s 25s 30s 35s 40s 45s... │
└──────────────────────────────────────────────┘
┌─ Statistics ─────────────────────────────────┐
│ Total Packets: 1234                          │
│ Peak Rate: 89 pkts/bin  Average: 41.1 pkts  │
│ Unique Channels: 15                          │
│ Average RSSI: -52.3 dBm                      │
└──────────────────────────────────────────────┘
```

## Integration with TUI

### App State

Views can be integrated into the main TUI app state:

```rust
pub enum VisualizationView {
    Heatmap,
    Timeline,
    PacketDetails,
}

pub struct VisualizationState {
    current_view: VisualizationView,
    heatmap_config: HeatmapConfig,
    timeline_config: TimelineConfig,
    packets: Vec<PacketData>,
}
```

### Keyboard Shortcuts

Suggested hotkeys for visualization views:

| Key | Action |
|-----|--------|
| `h` | Toggle heatmap view |
| `t` | Toggle timeline view |
| `c` | Cycle color schemes |
| `+/-` | Zoom in/out (time window) |
| `[/]` | Adjust time bins |
| `r` | Reset view |

### View Switching

```rust
match key.code {
    KeyCode::Char('h') => {
        app.visualization_state.current_view = VisualizationView::Heatmap;
    }
    KeyCode::Char('t') => {
        app.visualization_state.current_view = VisualizationView::Timeline;
    }
    KeyCode::Char('c') => {
        // Cycle color scheme
        app.visualization_state.heatmap_config.color_scheme =
            next_color_scheme(app.visualization_state.heatmap_config.color_scheme);
    }
    _ => {}
}
```

## Performance Considerations

### Memory Usage

**Heatmap:**
```
Memory = time_buckets × num_channels × 4 bytes (u32)
Example: 60 × 79 × 4 = 18.96 KB
```

**Timeline:**
```
Memory = time_bins × (base_size + channels_map)
Example: 30 × (~200 bytes) = ~6 KB
```

### Rendering Performance

- **Heatmap:** O(time_buckets × channels)
- **Timeline:** O(time_bins)

**Optimization tips:**
1. Limit time window for real-time views (30-60 seconds)
2. Reduce time bins for large datasets
3. Use efficient data structures (VecDeque for sliding windows)
4. Pre-compute aggregations

### Packet Processing

```rust
// Efficient packet aggregation
let heatmap = HeatmapData::from_packets(&packets, &config);
// Single pass through packets: O(n)
```

## Data Aggregation

### Time Bucketing

Packets are grouped into time bins:

```
Bin Duration = Total Duration / Number of Bins
Bin Index = (Packet Time - Start Time) / Bin Duration
```

### Channel Aggregation

For each bin, count packets per channel:

```rust
for packet in packets {
    let bin = time_bucket(packet.timestamp);
    let channel = packet.channel;
    bins[bin].channels[channel] += 1;
}
```

### RSSI Averaging

Rolling average per bin:

```
New Average = (Old Average × (N-1) + New RSSI) / N
```

## Use Cases

### 1. Channel Hopping Detection

**Heatmap view** reveals patterns of devices switching channels:
- Vertical lines indicate channel hopping sequences
- Horizontal bands show persistent channel use

### 2. Interference Analysis

**Heatmap color intensity** highlights interference:
- High activity on adjacent channels
- Wi-Fi coexistence issues (BLE ch 0-10 overlap with Wi-Fi ch 1)

### 3. Traffic Patterns

**Timeline view** shows temporal patterns:
- Periodic advertising (even spacing)
- Burst activity (data transfers)
- Idle periods

### 4. Device Fingerprinting

Combine heatmap + timeline:
- Advertising interval (timeline peaks)
- Channel preferences (heatmap hotspots)
- RSSI patterns (signal strength variation)

### 5. Performance Monitoring

Real-time capture analysis:
- Packet loss detection (gaps in timeline)
- Buffer utilization (packets per bin)
- Channel congestion (heatmap intensity)

## Export Capabilities

### Future Enhancements

- **PNG/SVG Export** - Save visualizations as images
- **CSV Export** - Export aggregated data
- **JSON Export** - Export visualization data structure

```rust
// Planned API
heatmap.export_to_png("heatmap.png")?;
timeline.export_to_csv("timeline.csv")?;
```

## Testing

### Unit Tests

Both visualization modules include comprehensive tests:

```bash
# Test heatmap functionality
cargo test --package ubertooth-cli heatmap

# Test timeline functionality
cargo test --package ubertooth-cli timeline
```

### Test Coverage

**Heatmap Tests:**
- Color scheme intensity mapping
- Heatmap data creation from packets
- Summary statistics calculation

**Timeline Tests:**
- Timeline creation from packets
- Statistics aggregation
- RSSI averaging

### Integration Testing

Create test data:

```rust
use chrono::Utc;
use ubertooth_platform::PacketData;

let packets = vec![
    PacketData {
        sequence: 1,
        timestamp: Utc::now(),
        data: vec![],
        packet_type: "LE_ADV".to_string(),
        channel: 37,
        rssi: Some(-50),
        metadata: serde_json::json!({}),
    },
    // ... more packets
];
```

## Troubleshooting

### Issue: Heatmap shows all black

**Cause:** No packets or all in single bin
**Solution:** Check packet timestamps, increase time window

### Issue: Timeline bars too small

**Cause:** Too many bins for available width
**Solution:** Reduce time_bins or increase terminal width

### Issue: Colors not visible

**Cause:** Terminal color scheme conflicts
**Solution:** Switch to Grayscale color scheme

### Issue: Performance lag

**Cause:** Too many packets or bins
**Solution:** Limit packet count, reduce bins, increase aggregation

## References

- [Ratatui Widgets](https://docs.rs/ratatui/latest/ratatui/widgets/)
- [BLE Channel Allocation](https://www.bluetooth.com/specifications/specs/core-specification/)
- [Data Visualization Best Practices](https://www.interaction-design.org/literature/article/data-visualization-best-practices)

---

**Last Updated:** 2026-03-18
**Phase:** 3.2 - Advanced Visualizations
