# Issue #2: Refactor sidecar.rs (3,469 lines → modules)

**Status:** Planning
**Created:** 2026-03-19
**File:** `crates/platform/src/sidecar.rs`
**Current Size:** 3,469 lines
**Target:** <800 lines per module

---

## Overview

The sidecar.rs file manages the Python ubertooth-tools wrapper. It contains:
- 4 impl blocks
- 55 public methods
- Multiple helper functions
- 6 struct definitions
- Test code

### Key Challenges
1. **Backend trait implementation** - UbertoothBackendProvider trait must remain intact
2. **Python subprocess management** - Command::new(), output parsing, error handling
3. **PCAP parsing** - Complex binary data processing with pcap-file crate
4. **Method interdependencies** - Parsing methods called by tool execution methods
5. **Test coverage** - Must preserve existing test_parse_pcap_ble_rf test

---

## Proposed Module Structure

```
crates/platform/src/sidecar/
├── mod.rs                  (200 lines) - Re-exports, main SidecarManager struct
├── types.rs                (100 lines) - All struct definitions
├── validation.rs           (150 lines) - check_ubertooth_installed, validation helpers
├── config.rs               (250 lines) - Configuration methods (6 methods)
├── device.rs               (200 lines) - Device management (3 methods)
├── capture.rs              (300 lines) - Capture operations (5 methods)
├── tools/
│   ├── mod.rs              (50 lines)  - Tool execution coordination
│   ├── scan.rs             (300 lines) - btle_scan, bt_scan
│   ├── spectrum.rs         (250 lines) - bt_specan
│   ├── analyze.rs          (300 lines) - bt_analyze, afh_analyze
│   ├── follow.rs           (250 lines) - bt_follow
│   ├── comparison.rs       (250 lines) - bt_compare
│   └── other.rs            (300 lines) - bt_discover, bt_decode, bt_fingerprint, bt_jam, bt_spoof
├── parsing/
│   ├── mod.rs              (50 lines)  - Parsing coordination
│   ├── pcap.rs             (600 lines) - parse_pcap, PCAP analysis
│   ├── ble.rs              (400 lines) - BLE-specific parsing
│   └── advertising.rs      (300 lines) - Advertisement data parsing
└── backend.rs              (200 lines) - UbertoothBackendProvider trait impl
```

**Total:** ~4,450 lines across 18 files (avg 247 lines/file)

---

## Implementation Phases

### Phase 1: Extract Types & Validation (Low Risk)
**Files:** `types.rs`, `validation.rs`
**Lines:** ~250 total
**Risk:** LOW - No method dependencies

**Steps:**
1. Create `crates/platform/src/sidecar/types.rs`
   - Move structs: PcapAnalysis, TimingAnalysis, SecurityObservation, SecurityAnalysis, BleDevice, SidecarManager
   - Add `pub(super)` visibility for module access
2. Create `crates/platform/src/sidecar/validation.rs`
   - Move `check_ubertooth_installed()` method
   - Extract validation helpers if any
3. Update `sidecar.rs` imports
4. **TEST:** `cargo check --package ubertooth-platform`

**Success Criteria:** Compilation passes with no warnings

---

### Phase 2: Extract Configuration (Medium Risk)
**Files:** `config.rs`
**Lines:** ~250
**Risk:** MEDIUM - Methods modify internal state

**Methods to Extract:**
- configure_channel()
- configure_modulation()
- configure_power()
- configure_squelch()
- configure_leds()
- session_context()

**Steps:**
1. Create `crates/platform/src/sidecar/config.rs`
2. Move configuration methods with proper `impl SidecarManager` block
3. Add internal helper functions if needed
4. Update `sidecar.rs` to use config module
5. **TEST:** `cargo check --package ubertooth-platform`

---

### Phase 3: Extract Device & Capture (Medium Risk)
**Files:** `device.rs`, `capture.rs`
**Lines:** ~500 total
**Risk:** MEDIUM - Subprocess execution patterns

**Device Methods:**
- device_connect()
- device_disconnect()
- device_status()

**Capture Methods:**
- capture_list()
- capture_get()
- capture_delete()
- capture_tag()
- capture_export()

**Steps:**
1. Create both files in single commit
2. Move methods with subprocess Command patterns
3. Keep error handling patterns consistent
4. **TEST:** `cargo check --package ubertooth-platform`

---

### Phase 4: Extract Tool Execution (High Risk)
**Files:** `tools/*.rs` (7 files)
**Lines:** ~1,700 total
**Risk:** HIGH - Complex subprocess management, multiple dependencies

**Breakdown:**
- **scan.rs**: btle_scan(), bt_scan() - Most commonly used
- **spectrum.rs**: bt_specan() - Spectrum analysis
- **analyze.rs**: bt_analyze(), afh_analyze() - Analysis tools
- **follow.rs**: bt_follow() - Connection following
- **comparison.rs**: bt_compare() - Comparison logic
- **other.rs**: bt_discover(), bt_decode(), bt_fingerprint(), bt_jam(), bt_spoof()

**Steps:**
1. Create `tools/mod.rs` first with re-exports
2. Extract ONE file at a time, test after each
3. Order: scan → spectrum → analyze → follow → comparison → other
4. **TEST after each:** `cargo check --package ubertooth-platform`

**Critical:** Each tool method calls parsing methods - ensure imports are correct

---

### Phase 5: Extract Parsing (Highest Risk)
**Files:** `parsing/*.rs` (4 files)
**Lines:** ~1,350 total
**Risk:** HIGHEST - Complex binary parsing, method interdependencies

**Breakdown:**
- **pcap.rs**: parse_pcap() - Main PCAP parsing (~600 lines)
- **ble.rs**: extract_ble_device(), extract_ble_device_from_rf() (~400 lines)
- **advertising.rs**: extract_device_name(), extract_ble_advertising_data(), parse_ble_advertising() (~300 lines)

**Dependencies Map:**
```
parse_pcap() → extract_ble_device() → extract_device_name()
              ↓                       ↓
           extract_ble_device_from_rf() → extract_ble_advertising_data() → parse_ble_advertising()
```

**Steps:**
1. Create `parsing/mod.rs` with module structure
2. Extract leaf functions FIRST (parse_ble_advertising, extract_device_name)
3. Extract mid-level (extract_ble_advertising_data, extract_ble_device*)
4. Extract root (parse_pcap) LAST
5. **TEST after each:** `cargo check --package ubertooth-platform`
6. **CRITICAL:** Run test_parse_pcap_ble_rf after completion

---

### Phase 6: Extract Backend Trait (Medium Risk)
**Files:** `backend.rs`
**Lines:** ~200
**Risk:** MEDIUM - Public API boundary

**Content:**
- UbertoothBackendProvider trait implementation
- All trait methods that delegate to other modules

**Steps:**
1. Create `backend.rs` with trait impl
2. Import all necessary modules
3. Verify trait methods delegate correctly
4. **TEST:** `cargo check --package ubertooth-platform`
5. **TEST:** Run examples that use backend trait

---

### Phase 7: Final Cleanup & Verification
**Files:** `mod.rs`
**Lines:** ~200
**Risk:** LOW - Coordination only

**Steps:**
1. Create `mod.rs` with all module declarations and re-exports
2. Update `crates/platform/src/lib.rs` to import from new structure
3. Remove old `sidecar.rs` file
4. **FULL TEST SUITE:**
   ```bash
   cargo test --package ubertooth-platform
   cargo test --package ubertooth-cli
   cargo clippy --package ubertooth-platform
   cargo fmt --check
   ```
5. Verify examples still work:
   ```bash
   cargo run --example test_backend_integration
   ```

---

## Testing Strategy

### After Each Phase
1. `cargo check --package ubertooth-platform` - Must pass
2. `cargo clippy --package ubertooth-platform` - No new warnings
3. Visual inspection of module structure

### After Phase 5 (Parsing)
1. Run `cargo test --package ubertooth-platform test_parse_pcap_ble_rf`
2. Verify test still passes with same results

### Final Verification
1. Full test suite
2. All examples compile and run
3. No clippy warnings
4. Formatting checks pass
5. Integration tests pass

---

## Rollback Plan

If any phase fails:
1. **DO NOT** continue to next phase
2. Review compilation errors
3. Fix issues incrementally
4. If unfixable quickly, restore from backup:
   ```bash
   git restore crates/platform/src/sidecar.rs crates/platform/src/sidecar/
   ```
5. Document what went wrong
6. Revise plan before next attempt

---

## Risk Mitigation

1. **Small commits** - One phase per commit
2. **Test after each phase** - Catch issues early
3. **Preserve tests** - Don't modify test logic
4. **Keep visibility consistent** - Use pub(super) within module, pub for external API
5. **Document dependencies** - Comment where methods call each other

---

## Success Criteria

- ✅ All files <800 lines
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Examples compile and run
- ✅ Code is more maintainable
- ✅ No functionality lost

---

## Estimated Time

**Conservative estimate:** 5-6 hours for careful incremental extraction

**Breakdown:**
- Phase 1: 30 min
- Phase 2: 45 min
- Phase 3: 1 hour
- Phase 4: 1.5 hours
- Phase 5: 2 hours (most complex)
- Phase 6: 45 min
- Phase 7: 1 hour (testing)

---

## Notes from Issue #1 (ui.rs refactoring)

**Key Lessons:**
1. **Underestimated complexity** - Simple function extraction doesn't work for stateful modules
2. **Need dependency mapping** - Must understand what calls what
3. **Test incrementally** - Catch issues after each small change
4. **Document state** - Know what variants/fields are accessed where
5. **Extract leaf modules first** - Start with least dependencies

**Applied to Issue #2:**
- Start with types (no dependencies)
- Then validation (minimal dependencies)
- Leave parsing for later (most dependencies)
- Test after EVERY phase, not just at end

---

*Created: 2026-03-19*
*Last Updated: 2026-03-19*
