# Add Interactive TUI Mode for Easy Tool Access and Strike48 Integration

## 🎯 Problem Statement

The Ubertooth connector currently has two modes:
1. **Headless agent** - Runs as Strike48 connector, great for automation
2. **CLI** - Powerful but requires knowing exact commands and parameters

**Gap:** No easy way for researchers/operators to:
- Quickly explore what tools are available
- Execute tools without memorizing syntax
- Configure Strike48 connection settings
- See results in a user-friendly way
- Use the tool interactively in the field

## 👥 Target Users

1. **Security Researchers** - Need quick access to scanning/analysis tools
2. **Field Operators** - Want simple interface for common tasks
3. **Strike48 Users** - Need easy way to configure and test connection before deploying

## 💡 Proposed Solution

Add an interactive Terminal User Interface (TUI) mode to `ubertooth-cli`:

```bash
ubertooth-cli --tui
```

**Key Features:**
- Menu-driven access to all 36 tools
- Dynamic parameter forms (generated from tool schemas)
- Strike48 connection settings page
- Live progress indicators
- Results display with basic formatting

## 🎨 User Experience

### Main Menu
```
┌─ Ubertooth CLI ────────────────────────────────────────────────────┐
│ Device: Connected ✓ | Backend: Rust | Strike48: Connected ✓       │
├────────────────────────────────────────────────────────────────────┤
│ Select Tool Category:                                               │
│                                                                     │
│   1. 🔌 Device Management (3 tools)                                │
│      Connect, status, disconnect                                   │
│                                                                     │
│   2. 🔍 Reconnaissance (7 tools)                                   │
│      BLE scan, spectrum analysis, follow connections               │
│                                                                     │
│   3. 📊 Analysis (5 tools)                                         │
│      Packet analysis, fingerprinting, comparison                   │
│                                                                     │
│   4. 📁 Capture Management (5 tools)                               │
│      List, view, export, tag captures                              │
│                                                                     │
│   5. ⚙️  Configuration (8 tools)                                    │
│      Channel, power, modulation, presets                           │
│                                                                     │
│   6. 🎯 Attack Operations (5 tools) ⚠️                              │
│      Injection, jamming, MITM (requires authorization)             │
│                                                                     │
│   7. 🔧 Advanced (2 tools)                                         │
│      Raw USB commands, session context                             │
│                                                                     │
│ [s] Settings  [h] Help  [q] Quit                                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Tool Execution (Example: BLE Scan)
```
┌─ BLE Scan ─────────────────────────────────────────────────────────┐
│                                                                     │
│  Duration (seconds):  [30____________________]  (1-300)            │
│                       ↑ Required                                   │
│                                                                     │
│  Channel:             [37____________________]  (37/38/39)         │
│                       ↑ BLE advertising channels                   │
│                                                                     │
│  Save PCAP:           [✓] Yes                                      │
│                                                                     │
│  [Start Scan] [Cancel] [Back]                                      │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────│
│  Status: Scanning...                                                │
│  Progress: [████████████░░░░░░░░] 20/30 seconds                    │
│  Packets:  312                                                      │
│  Devices:  18 unique                                                │
│                                                                     │
│  Recent Devices:                                                    │
│  • Apple Watch    F2:3A:B4:C5:D6:E7  -45 dBm  [ADV_IND]           │
│  • Unknown        A1:B2:C3:D4:E5:F6  -67 dBm  [ADV_NONCONN_IND]   │
│  • Fitbit Charge  C3:D4:E5:F6:A7:B8  -72 dBm  [SCAN_RSP]          │
│  ...                                                                │
│                                                                     │
│  [Cancel Scan (Ctrl+C)]                                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Settings Page
```
┌─ Settings ─────────────────────────────────────────────────────────┐
│                                                                     │
│  Prospector Studio / Strike48 Connection:                          │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Server URL:  [wss://jt-demo-01.strike48.engineering_______] │  │
│  │                                                              │  │
│  │ Tenant ID:   [non-prod____________________________________] │  │
│  │                                                              │  │
│  │ Auth Token:  [ott_QoU4XIHRjkuDRxMpSBLHCyaRmaBfWG1_30WV0vV3] │  │
│  │              (leave empty for unauthenticated mode)         │  │
│  │                                                              │  │
│  │ Status:      ✓ Connected                                    │  │
│  │ Instance:    matrix:non-prod:ubertooth:unknown-1772483...   │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  Backend Selection:                                                 │
│  ( ) Python (ubertooth-tools) - Stable, all features               │
│  (•) Rust (native USB) - 100-200x faster, 9 tools native          │
│                                                                     │
│  Device:                                                            │
│  Device Index: [0__]  (0 for first Ubertooth found)               │
│                                                                     │
│  [Test Connection] [Save] [Cancel]                                 │
│                                                                     │
│  Config file: ~/.ubertooth/config.toml                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Results Display
```
┌─ BLE Scan Results ─────────────────────────────────────────────────┐
│                                                                     │
│  Scan completed successfully!                                       │
│  Duration: 30.2 seconds                                             │
│  Packets:  461                                                      │
│  Devices:  23 unique                                                │
│  Capture:  cap-btle-37-20260303-120045                             │
│  PCAP:     ~/.ubertooth/captures/cap-btle-37-20260303-120045.pcap  │
│                                                                     │
│  ─────────────────────────────────────────────────────────────────│
│  Discovered Devices:                                                │
│                                                                     │
│  MAC Address         Name              RSSI   Type      Flags      │
│  ──────────────────────────────────────────────────────────────── │
│  F2:3A:B4:C5:D6:E7  Apple Watch       -45    Random    LE+BR/EDR  │
│  A1:B2:C3:D4:E5:F6  (Unknown)         -67    Public    LE Only    │
│  C3:D4:E5:F6:A7:B8  Fitbit Charge 5   -72    Random    LE Only    │
│  B4:C5:D6:E7:F8:A9  Samsung Galaxy    -58    Public    LE+BR/EDR  │
│  ...                                                                │
│                                                                     │
│  [View Raw JSON] [Export] [Analyze] [Back to Menu]                │
└─────────────────────────────────────────────────────────────────────┘
```

## 🎯 MVP Scope (1-2 Weeks)

### Week 1: Core Functionality (3-4 days)
**Goal:** Functional TUI that can execute all 36 tools

- [ ] Basic ratatui application structure
- [ ] Main menu with 7 tool categories
- [ ] Tool selection submenu (list tools in category)
- [ ] Generic parameter form builder
  - Read tool schema (from existing `Tool` trait)
  - Generate text inputs for each parameter
  - Basic validation (required fields, type checking)
- [ ] Tool execution
  - Call existing `Tool::execute()` methods
  - Handle async operations
  - Show spinner/progress for long operations
- [ ] Results display (JSON pretty-print for now)
- [ ] Settings page
  - Strike48 URL, tenant, auth token fields
  - Backend selection (Rust/Python)
  - Save to `~/.ubertooth/config.toml`
- [ ] Basic navigation (arrow keys, enter, esc, q)

**Deliverable:** Can execute any tool through TUI, configure Strike48 settings.

### Week 2: Polish & UX (2-3 days)
**Goal:** Production-ready, pleasant to use

- [ ] Better results formatting
  - Tables for device lists
  - Syntax highlighting for JSON
  - Capture file paths as clickable links (copy to clipboard)
- [ ] Progress indicators
  - Live packet counter during scans
  - Progress bar with time remaining
  - Cancellation support (Ctrl+C)
- [ ] Error handling
  - Graceful error modals
  - Device disconnect recovery
  - Strike48 connection errors
- [ ] Help system
  - Help screen ([h] key)
  - Tool descriptions in menu
  - Parameter hints in forms
- [ ] Keyboard shortcuts
  - Tab/Shift+Tab: Navigate fields
  - Ctrl+C: Cancel operation
  - Esc: Back/cancel
  - [s]: Jump to settings
  - [q]: Quit (with confirmation if operation running)
- [ ] State persistence
  - Remember last used parameters
  - Save Strike48 config
  - Recent tools list

**Deliverable:** Polished, production-ready TUI.

## 🛠️ Technical Implementation

### Architecture

```
apps/cli/
├── main.rs           (entry point, --tui flag handling)
├── tui/
│   ├── mod.rs        (TUI module exports)
│   ├── app.rs        (App state, event loop)
│   ├── ui.rs         (Rendering logic)
│   ├── views/
│   │   ├── menu.rs       (Main menu)
│   │   ├── tool_form.rs  (Generic tool parameter form)
│   │   ├── results.rs    (Results display)
│   │   └── settings.rs   (Settings page)
│   ├── events.rs     (Event handling)
│   └── widgets/
│       ├── table.rs      (Reusable table widget)
│       └── progress.rs   (Progress bar)
└── config.rs         (Load/save config)
```

### Dependencies

Add to `apps/cli/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...

# TUI (always included)
ratatui = "0.26"
crossterm = "0.27"
tui-textarea = "0.4"  # For text input widgets

# Already have tokio, so async is covered
```

### Key Design Decisions

1. **Form Generation:** Use existing `Tool::input_schema()` to auto-generate forms
   - Parse JSON schema → create text inputs
   - Validate before execution
   - Show parameter descriptions as hints

2. **Async Execution:** Use tokio channels for tool execution
   ```rust
   // Pseudo-code
   let (tx, rx) = mpsc::channel();
   tokio::spawn(async move {
       let result = tool.execute(params).await;
       tx.send(result).await;
   });
   // UI polls rx for updates
   ```

3. **State Management:** Simple enum-based state machine
   ```rust
   enum AppState {
       MainMenu,
       ToolCategory(Category),
       ToolForm { tool: Arc<dyn Tool>, params: HashMap },
       Executing { tool_name: String, progress: f32 },
       Results { output: Value },
       Settings,
   }
   ```

4. **Config Storage:** TOML file at `~/.ubertooth/config.toml`
   ```toml
   [strike48]
   url = "wss://jt-demo-01.strike48.engineering"
   tenant_id = "non-prod"
   auth_token = "ott_..."

   [backend]
   type = "rust"  # or "python"

   [device]
   index = 0
   ```

### Integration Points

**Reuse existing infrastructure:**
- ✅ `ubertooth-tools::create_tool_registry()` - Get all 36 tools
- ✅ `Tool::input_schema()` - Generate forms automatically
- ✅ `Tool::execute()` - Run tools
- ✅ `CaptureStore` - List/manage captures
- ✅ Backend abstraction - Works with Rust or Python backend

**No changes needed to existing code!** TUI is purely additive.

## 🚫 Non-Goals (Not in MVP)

These are great ideas but **NOT** in the 1-2 week MVP:

- ❌ Real-time spectrum waterfall visualization
- ❌ Advanced packet inspector (use Wireshark for that)
- ❌ Capture editing/manipulation
- ❌ Multi-device support
- ❌ Scripting/automation within TUI
- ❌ Replicate Strike48 UI locally
- ❌ Show tool execution history
- ❌ Fancy animations/transitions

**Philosophy:** Ship a functional MVP fast, add polish later.

## ✅ Success Criteria

**MVP is successful if:**
1. ✅ Can execute all 36 tools through TUI
2. ✅ Parameter forms are generated automatically from schemas
3. ✅ Can configure Strike48 connection and see status
4. ✅ Results are readable (doesn't have to be beautiful)
5. ✅ No crashes on common operations
6. ✅ Works on Linux (primary target)
7. ✅ Can be used by someone who's never used the CLI before

## 📋 Implementation Checklist

### Week 1: Core (Must-Have)
- [ ] Set up ratatui app structure in `apps/cli/src/tui/`
- [ ] Main menu rendering (7 categories)
- [ ] Tool selection submenu
- [ ] Generic form builder using tool schemas
- [ ] Execute tool and show JSON results
- [ ] Settings page (Strike48 config)
- [ ] Config persistence (`~/.ubertooth/config.toml`)
- [ ] Basic keyboard navigation
- [ ] `ubertooth-cli --tui` flag handling

### Week 2: Polish (Should-Have)
- [ ] Progress indicators for long operations
- [ ] Cancel support (Ctrl+C during scan)
- [ ] Better results formatting (tables for devices)
- [ ] Error handling (modals, graceful recovery)
- [ ] Help screen
- [ ] Input validation with error messages
- [ ] State persistence (remember last params)
- [ ] Keyboard shortcuts guide
- [ ] Test on actual hardware

### Documentation
- [ ] Update README with TUI section
- [ ] Add TUI screenshots/demo GIF
- [ ] Document keyboard shortcuts
- [ ] Update `AGENT_PROMPT.md` to mention TUI

## 🧪 Testing Strategy

**Manual Testing (primary):**
- Test all 36 tools through TUI
- Test with Ubertooth hardware connected/disconnected
- Test Strike48 connection (authenticated and unauthenticated)
- Test on different terminal sizes
- Test error scenarios (USB unplugged, timeout, etc.)

**Automated Testing:**
- Unit tests for form builder
- Unit tests for config loading/saving
- Smoke test: Launch TUI, verify main menu renders

**No snapshot testing for UI** - too fragile, not worth it for MVP.

## 📦 Deliverables

**End of Week 1:**
- Working TUI that can execute all tools
- Can configure Strike48 settings
- Functional but ugly

**End of Week 2:**
- Polished, production-ready TUI
- Documentation updated
- README includes demo GIF
- Tagged release: `v0.2.0` or similar

## 🎬 Demo Script (For Testing)

1. Launch TUI: `ubertooth-cli --tui`
2. Navigate to Settings, configure Strike48
3. Test Connection → Should show "Connected ✓"
4. Navigate to Reconnaissance → BLE Scan
5. Set duration=10, channel=37, save_pcap=true
6. Execute scan
7. See live progress (packets counting up)
8. View results (table of devices)
9. Navigate to Capture Management → List Captures
10. See the capture we just created
11. Exit TUI

**If this works smoothly, MVP is successful!**

## 🚀 Future Enhancements (Phase 2+)

Once MVP is shipped, consider:
- Real-time spectrum waterfall (using `tui-rs-tree-widget` or custom)
- Capture diff viewer (side-by-side comparison)
- Tool favorites/recent list
- Batch operations (run multiple scans)
- Export results to CSV/Markdown
- Integrated help (show `AGENT_PROMPT.md` in TUI)
- Theme support (dark/light/custom)
- Remote operation (TUI connects to remote agent)

## 💭 Open Questions

1. **Terminal requirements:** Minimum terminal size? (suggest 80x24)
2. **Windows support:** Test on Windows Terminal? (nice-to-have, not blocker)
3. **Color scheme:** Match Strike48 branding or use defaults?
4. **Logging:** Show logs in TUI or just write to file?

## 🎯 Assignment

**Estimated effort:** 1-2 weeks (focused work)
**Priority:** HIGH (NOW)
**Complexity:** Medium (reusing existing infrastructure)

**Skills needed:**
- Rust (intermediate)
- TUI development (can learn ratatui quickly)
- Async/tokio (basic understanding)

**Who can work on this:**
- One developer can knock this out in 1-2 weeks
- Could pair program for faster delivery
- Recommend starting with Week 1 goals, ship early, iterate

---

## 📝 Notes

- Keep it simple! Functional > Beautiful for MVP
- Reuse everything - don't reinvent wheels
- Ship early, get feedback, iterate
- If something is hard, skip it for v2

Let's build this! 🚀
