# PCAP Integration Guide

Guide for importing and exporting PCAP files for use with Wireshark and other tools.

## Overview

The PCAP integration enables:
- **Import** - Load existing PCAP/PCAPNG files into the capture store
- **Export** - Export captures to PCAP format for analysis in Wireshark
- **Validation** - Verify PCAP file integrity and format
- **Format Support** - Both classic PCAP (.pcap) and PCAPNG (.pcapng) formats

## Quick Start

### Importing a PCAP File

```rust
use ubertooth_platform::{CaptureStore, PcapManager};

// Create PCAP manager
let capture_store = CaptureStore::new()?;
let mut pcap_manager = PcapManager::new(capture_store);

// Import PCAP file
let capture_id = pcap_manager.import_pcap(
    Path::new("/path/to/capture.pcap"),
    Some("Imported from Wireshark".to_string()),
)?;

println!("Imported capture: {}", capture_id);
```

### Exporting a Capture

```rust
// Export capture to PCAP file
pcap_manager.export_pcap(
    &capture_id,
    Path::new("/path/to/output.pcap"),
)?;

println!("Exported to output.pcap");
```

### Validating a PCAP File

```rust
use ubertooth_platform::PcapManager;

// Validate PCAP file
let validation = PcapManager::validate_pcap(Path::new("/path/to/capture.pcap"))?;

println!("Format: {:?}", validation.format);
println!("Packets: {}", validation.packet_count);
println!("File size: {} bytes", validation.file_size);
println!("Valid: {}", validation.is_valid);

if !validation.errors.is_empty() {
    println!("Errors:");
    for error in &validation.errors {
        println!("  - {}", error);
    }
}
```

## API Reference

### PcapManager

Manager for PCAP import/export operations.

```rust
pub struct PcapManager {
    capture_store: CaptureStore,
}

impl PcapManager {
    pub fn new(capture_store: CaptureStore) -> Self;

    pub fn import_pcap(
        &mut self,
        pcap_path: &Path,
        description: Option<String>,
    ) -> Result<String>;

    pub fn export_pcap(
        &self,
        capture_id: &str,
        output_path: &Path,
    ) -> Result<()>;

    pub fn validate_pcap(path: &Path) -> Result<PcapValidation>;
}
```

### PcapFormat

Supported PCAP formats.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PcapFormat {
    /// Classic PCAP format (.pcap)
    Pcap,
    /// PCAP Next Generation format (.pcapng)
    PcapNg,
}

impl PcapFormat {
    pub fn from_extension(path: &Path) -> Result<Self>;
    pub fn extension(&self) -> &str;
}
```

**Format Detection:**
- `.pcap` extension → `PcapFormat::Pcap`
- `.pcapng` extension → `PcapFormat::PcapNg`
- Other extensions → Error

### PcapValidation

Validation result structure.

```rust
#[derive(Debug)]
pub struct PcapValidation {
    pub format: PcapFormat,        // Detected format
    pub file_size: u64,            // File size in bytes
    pub packet_count: usize,       // Number of packets
    pub is_valid: bool,            // Overall validity
    pub errors: Vec<String>,       // Error messages
}
```

## Import Process

### What Happens During Import

1. **Format Detection** - Automatically detects .pcap or .pcapng format
2. **Packet Counting** - Counts total packets in file
3. **File Copying** - Copies PCAP to `~/.ubertooth/captures/{capture-id}/`
4. **Metadata Creation** - Generates capture metadata
5. **Storage** - Saves metadata to `metadata.json`

### Import Metadata

```json
{
  "capture_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-03-19T10:30:00Z",
  "capture_type": "imported",
  "packet_count": 1523,
  "duration_sec": null,
  "file_size_bytes": 245760,
  "pcap_path": "/home/user/.ubertooth/captures/550e.../capture.pcap",
  "tags": ["imported"],
  "description": "Imported from wireshark_capture.pcap",
  "category": null,
  "notes": null
}
```

### Automatic Tagging

Imported captures are automatically tagged with:
- `"imported"` tag for easy filtering

## Export Process

### What Happens During Export

1. **Metadata Lookup** - Finds capture by ID
2. **File Location** - Locates source PCAP file
3. **File Copy** - Copies PCAP to output path
4. **Format Preservation** - Maintains original format

### Export Options

```rust
// Export to specific location
pcap_manager.export_pcap(
    "capture-id",
    Path::new("/tmp/exported.pcap"),
)?;

// Export preserves original format
// If imported as .pcap → exports as .pcap
// If imported as .pcapng → exports as .pcapng
```

## Validation

### File Integrity Checking

The validation process:
- Opens PCAP file
- Attempts to read all packets/blocks
- Collects any errors encountered
- Returns summary with error details

### Common Validation Errors

**Truncated File:**
```
Error: Packet 1234: Incomplete buffer
```

**Corrupted Packet:**
```
Error: Packet 567: Invalid packet length
```

**Invalid Format:**
```
Error: Failed to open PCAP: Invalid magic number
```

### Validation Example

```rust
let validation = PcapManager::validate_pcap(Path::new("capture.pcap"))?;

if !validation.is_valid {
    eprintln!("File has {} errors:", validation.errors.len());
    for error in &validation.errors {
        eprintln!("  {}", error);
    }
} else {
    println!("✓ File is valid");
    println!("  {} packets", validation.packet_count);
    println!("  {} bytes", validation.file_size);
}
```

## Format Support

### Classic PCAP (.pcap)

**Characteristics:**
- Older format (libpcap)
- Single interface per file
- 24-byte packet header
- Widely supported

**Use When:**
- Maximum compatibility needed
- Working with older tools
- Simpler format requirements

### PCAP Next Generation (.pcapng)

**Characteristics:**
- Modern format (2004+)
- Multiple interfaces per file
- Enhanced packet blocks
- Extensible metadata
- Better for complex captures

**Use When:**
- Capturing from multiple devices
- Need rich metadata
- Using modern Wireshark versions
- Advanced analysis required

## Integration with Wireshark

### Opening Ubertooth Captures in Wireshark

1. **Export Capture:**
```bash
# From CLI (future feature)
ubertooth-cli export --capture-id <id> --output capture.pcap
```

2. **Open in Wireshark:**
```bash
wireshark capture.pcap
```

3. **Bluetooth LE Dissection:**
   - Wireshark automatically recognizes Bluetooth LE packets
   - Use display filter: `btle` or `bluetooth.le`
   - Apply specific filters: `btle.advertising_address`, `btle.connection_address`

### Importing Wireshark Captures

1. **Capture in Wireshark** with Bluetooth LE interface

2. **Save Capture:**
   - File → Save As
   - Choose `.pcap` or `.pcapng` format

3. **Import to Ubertooth:**
```rust
let capture_id = pcap_manager.import_pcap(
    Path::new("/path/to/wireshark_capture.pcap"),
    Some("Captured with Wireshark".to_string()),
)?;
```

## Command-Line Usage (Future)

Planned CLI integration:

```bash
# Import PCAP
ubertooth-cli import --pcap /path/to/capture.pcap --description "My capture"

# Export capture
ubertooth-cli export --capture-id <id> --output /tmp/exported.pcap

# Validate PCAP
ubertooth-cli validate --pcap /path/to/capture.pcap

# List imported captures
ubertooth-cli list --filter tag:imported
```

## Use Cases

### 1. Analyzing Third-Party Captures

```rust
// Import capture from another tool
let capture_id = pcap_manager.import_pcap(
    Path::new("/data/external_capture.pcap"),
    Some("External BLE scan".to_string()),
)?;

// Now use Ubertooth analysis tools
// - Device fingerprinting
// - Multi-capture comparison
// - Advanced visualizations
```

### 2. Exporting for External Analysis

```rust
// Capture with Ubertooth
// ... perform capture ...

// Export for Wireshark analysis
pcap_manager.export_pcap(
    &capture_id,
    Path::new("/tmp/for_wireshark.pcap"),
)?;

// Analyze in Wireshark with additional dissectors
```

### 3. Capture Migration

```rust
// Import existing capture library
let pcap_files = glob::glob("/old_captures/*.pcap")?;

for entry in pcap_files {
    let path = entry?;
    let description = format!("Migrated from {}", path.display());

    match pcap_manager.import_pcap(&path, Some(description)) {
        Ok(id) => println!("✓ Imported: {} → {}", path.display(), id),
        Err(e) => eprintln!("✗ Failed {}: {}", path.display(), e),
    }
}
```

### 4. Batch Validation

```rust
// Validate all PCAP files in directory
for entry in std::fs::read_dir("/captures")? {
    let path = entry?.path();
    if path.extension().map_or(false, |e| e == "pcap") {
        match PcapManager::validate_pcap(&path) {
            Ok(v) if v.is_valid => {
                println!("✓ {} - {} packets", path.display(), v.packet_count);
            }
            Ok(v) => {
                eprintln!("✗ {} - {} errors", path.display(), v.errors.len());
            }
            Err(e) => {
                eprintln!("✗ {} - {}", path.display(), e);
            }
        }
    }
}
```

## Error Handling

### Common Errors

**File Not Found:**
```rust
Err(UbertoothError::InvalidParameter("PCAP file not found: ..."))
```

**Invalid Format:**
```rust
Err(UbertoothError::ParseError("Failed to open PCAP: ..."))
```

**Capture Not Found (Export):**
```rust
Err(UbertoothError::Io(...))  // When metadata.json missing
```

### Best Practices

1. **Validate Before Import:**
```rust
// Check file is valid before importing
if PcapManager::validate_pcap(&path)?.is_valid {
    pcap_manager.import_pcap(&path, description)?;
}
```

2. **Handle Partial Files:**
```rust
// Some files may be truncated but still usable
let validation = PcapManager::validate_pcap(&path)?;
if validation.packet_count > 0 {
    // File has some valid packets
    println!("Warning: File may be truncated ({} packets)", validation.packet_count);
    pcap_manager.import_pcap(&path, description)?;
}
```

3. **Check Disk Space:**
```rust
// PCAP files can be large - check before importing
let file_size = std::fs::metadata(&path)?.len();
if file_size > 1_000_000_000 {  // 1GB
    println!("Warning: Large file ({} bytes)", file_size);
}
```

## Limitations

### Current Limitations

1. **No Format Conversion** - Export maintains original format
   - Future: Add PCAP ↔ PCAPNG conversion

2. **Simple Packet Counting** - Only counts packets, doesn't parse content
   - Future: Extract additional metadata during import

3. **No Filtering on Import** - Imports all packets
   - Future: Add selective import with filters

4. **No Merge Support** - Cannot combine multiple PCAPs
   - Future: Add merge functionality

### Workarounds

**Format Conversion:**
Use `editcap` from Wireshark:
```bash
editcap -F pcap capture.pcapng capture.pcap  # PCAPNG → PCAP
editcap -F pcapng capture.pcap capture.pcapng  # PCAP → PCAPNG
```

**Packet Filtering:**
Use `tshark`:
```bash
tshark -r capture.pcap -Y "btle.advertising_address == aa:bb:cc:dd:ee:ff" -w filtered.pcap
```

**Merging Captures:**
Use `mergecap`:
```bash
mergecap -w merged.pcap capture1.pcap capture2.pcap capture3.pcap
```

## Performance

### Import Performance

**Typical Performance:**
- Small files (<10MB): <100ms
- Medium files (10-100MB): <1s
- Large files (>100MB): 1-10s

**Factors:**
- Packet count (more packets = slower)
- File size (larger = more I/O)
- Disk speed (SSD vs HDD)

### Optimization Tips

1. **Batch Imports** - Import multiple files in parallel
2. **Validation** - Skip validation if file is trusted
3. **Disk I/O** - Use SSD for capture storage

## Testing

### Unit Tests

The PCAP module includes 5 unit tests:
- `test_pcap_format_detection` - Format detection from extension
- `test_pcap_format_extension` - Extension retrieval from format
- `test_validate_missing_file` - Error handling for missing files
- `test_import_missing_file` - Import error handling
- `test_export_nonexistent_capture` - Export error handling

### Running Tests

```bash
cargo test --package ubertooth-platform --lib pcap::tests
```

## References

- [PCAP Format Specification](https://wiki.wireshark.org/Development/LibpcapFileFormat)
- [PCAPNG Format Specification](https://github.com/pcapng/pcapng)
- [Wireshark](https://www.wireshark.org/)
- [pcap-file Crate](https://docs.rs/pcap-file/)

---

**Last Updated:** 2026-03-19
**Phase:** 4.3 - PCAP Integration
**Status:** Complete
