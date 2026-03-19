# Issue #1: ui.rs Refactoring - Detailed Implementation Plan

## Challenge Encountered

Initial attempt to split ui.rs revealed significant complexity:
- 40 functions with complex interdependencies
- Multiple AppState variants (15+) with different field structures
- Cross-module function calls (capture → analysis, comparison)
- Shared helper functions (centered_rect, categorize_error)

**Lesson:** Cannot simply extract functions - need to understand:
1. AppState structure and all variants
2. Function call graph (who calls whom)
3. Shared utilities placement
4. Import requirements per module

## Recommended Approach: Incremental Refactoring

### Phase 1: Preparation (30 min)
1. **Map dependencies**
   ```bash
   # Create function call graph
   grep -n "render_" apps/cli/src/tui/ui.rs | awk '{print $1}' > functions.txt
   # For each function, grep for calls to other render_ functions
   ```

2. **Identify shared utilities**
   - `centered_rect()` - used by multiple modules
   - `categorize_error()` - error handling
   - Layout constants
   - Common imports

3. **Document AppState variants**
   ```bash
   grep -A 20 "pub enum AppState" apps/cli/src/tui/app.rs > appstate_structure.txt
   ```

### Phase 2: Extract Leaf Modules First (2 hours)

**Order of extraction (least → most dependent):**

#### 1. Utilities Module (utils.rs)
Extract standalone helpers:
- `centered_rect()`
- `categorize_error()`
- Any other pure functions

#### 2. Help Module (help.rs)  
- Single function: `render_help_overlay()`
- No dependencies on other render functions
- ~643 lines
- **Test**: Compile after extraction

#### 3. Session Module (session.rs)
- Functions: `render_session_*`
- Clear boundary, minimal dependencies
- ~180 lines
- **Test**: Compile + run

#### 4. Settings Module (settings.rs)
- Functions: `render_settings()`, `render_theme_selector()`, `render_confirmation()`, `render_export_menu()`
- May call `centered_rect()` from utils
- ~460 lines
- **Test**: Compile + run

#### 5. Live Capture Module (live_capture.rs)
- Functions: `render_live_*`
- Self-contained
- ~280 lines
- **Test**: Compile + run

### Phase 3: Extract Analysis/Capture Modules (1 hour)

These have interdependencies - need careful coordination:

#### 6. Analysis Module (analysis.rs)
- Functions: `render_analysis_*`, `render_decoded_packets()`
- May be called from capture module
- Export public functions that capture needs
- ~590 lines

#### 7. Packet View Module (packet_view.rs)
- Functions: `render_packet_*`
- Used by multiple other modules
- ~650 lines

#### 8. Comparison Module (comparison.rs)
- Functions: `render_comparison_*`, `render_filter_dialog()`
- Calls packet_view functions
- ~540 lines

#### 9. Capture Module (capture.rs)
- Functions: `render_capture_*`, `render_results()`
- Calls analysis, comparison, packet_view
- ~750 lines

### Phase 4: Extract Dashboard (30 min)

#### 10. Dashboard Module (dashboard.rs)
- Functions: `render_main_menu()`, `render_tool_*`, `render_error_message()`
- Core menu system
- ~600 lines

### Phase 5: Create mod.rs (30 min)

**Critical:** mod.rs must:
1. Re-export all public functions sub-modules need
2. Import from sub-modules in `render_content()`
3. Keep core functions: `render()`, `render_header()`, `render_footer()`, `render_notification()`, `render_dialog()`

Template:
```rust
// Sub-modules
pub mod utils;
pub mod help;
pub mod session;
pub mod settings;
pub mod live_capture;
pub mod analysis;
pub mod packet_view;
pub mod comparison;
pub mod capture;
pub mod dashboard;

// Re-export utilities
pub use utils::{centered_rect, categorize_error};

// Main render function
pub fn render(...) {
    // Dispatch to sub-modules based on AppState
    match state {
        AppState::MainMenu { ... } => dashboard::render_main_menu(...),
        AppState::Results { ... } => capture::render_results(...),
        // etc.
    }
}
```

### Phase 6: Fix Imports & Cross-references (1 hour)

For each module that calls functions in other modules:
1. Add `use super::other_module::function_name;`
2. Or call via module path: `other_module::function_name()`

Common imports needed:
```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell, Wrap},
    Frame,
};
use std::sync::Arc;
use crate::tui::{app::AppState, themes::Theme};
use super::utils::centered_rect;
```

### Phase 7: Testing & Verification (30 min)

After each module extraction:
```bash
# 1. Compile
cargo build --bin ubertooth-cli

# 2. Run
cargo run --bin ubertooth-cli

# 3. Test navigation
# - Open each screen
# - Verify rendering works
# - Check no panics
```

## Tools & Scripts

### Function Extraction Script
```bash
#!/bin/bash
# extract_function.sh <start_line> <end_line> <output_file>

START=$1
END=$2
OUTPUT=$3

sed -n "${START},${END}p" apps/cli/src/tui/ui.rs | \
  sed 's/^fn render_/pub fn render_/' > "$OUTPUT"
```

### Import Generator
```bash
# Based on function content, suggest imports
grep -o "use.*;" function.rs | sort | uniq
```

### Dependency Checker
```bash
# Find which functions a function calls
grep "render_[a-z_]*(" function.rs | \
  sed 's/.*\(render_[a-z_]*\).*/\1/' | \
  sort | uniq
```

## Estimated Timeline

| Phase | Time | Cumulative |
|-------|------|------------|
| 1. Preparation | 30 min | 30 min |
| 2. Leaf modules (5) | 2 hours | 2.5 hours |
| 3. Core modules (4) | 1 hour | 3.5 hours |
| 4. Dashboard | 30 min | 4 hours |
| 5. mod.rs | 30 min | 4.5 hours |
| 6. Fix imports | 1 hour | 5.5 hours |
| 7. Testing | 30 min | 6 hours |

**Total: ~6 hours** (vs. original estimate of 2-3 hours)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Circular dependencies | Extract in dependency order (leaf → root) |
| Breaking changes | Test after each module |
| Import errors | Use comprehensive import lists |
| AppState mismatch | Document all variants before starting |
| Lost context | Save at each milestone, commit frequently |

## Success Criteria

- ✅ No file > 800 lines
- ✅ All tests pass
- ✅ Application runs without errors
- ✅ All screens render correctly
- ✅ No functional changes
- ✅ Clean git history with incremental commits

## Recommendation

**Defer to fresh session:**
This refactoring is more complex than initially estimated. Recommend:

1. Close issue #1 with this detailed plan
2. Create sub-issues for each phase
3. Tackle in fresh context with full token budget
4. Use pair programming / incremental approach
5. Commit after each successful module extraction

**Why:**
- Current context: 124K/200K tokens (62% used)
- Remaining work needs ~40-50K tokens
- Complex cross-references need careful attention
- Better to do methodically than rush
