# Mouse Support - Test Plan

## Automated Build Verification

```bash
# Verify the code compiles
cargo build --bin ubertooth-cli

# Check for mouse-related code
grep -n "MouseEvent\|ScrollDown\|ScrollUp" apps/cli/src/tui/app.rs

# Verify mouse capture is enabled
grep -n "EnableMouseCapture\|DisableMouseCapture" apps/cli/src/tui/app.rs
```

## Manual Testing Checklist

### Prerequisites
- Terminal with mouse support (Windows Terminal, iTerm2, Alacritty, etc.)
- Ubertooth CLI built: `cargo build --bin ubertooth-cli`

### Test Procedure

Run: `cargo run --bin ubertooth-cli`

---

### Test 1: Main Menu - Mouse Clicks ✓

**Test Steps:**
1. Launch application
2. Use mouse to click on different menu categories (0-6)
3. Verify clicked item is selected and category opens

**Expected Results:**
- Click highlights item
- Click activates/opens category
- Visual feedback shows selection

**Test Items:**
- [ ] Device Management
- [ ] Configuration
- [ ] Reconnaissance & Scanning
- [ ] Capture Management
- [ ] Analysis & Decoding
- [ ] Attack Operations
- [ ] Settings

---

### Test 2: Main Menu - Scroll Wheel ✓

**Test Steps:**
1. In main menu
2. Scroll wheel down
3. Scroll wheel up

**Expected Results:**
- Scroll down moves selection down
- Scroll up moves selection up
- Selection wraps or stops at boundaries

---

### Test 3: Tool Category Menu - Mouse Clicks ✓

**Test Steps:**
1. Enter any tool category (click or press Enter on category)
2. Click different tools in the list
3. Verify tool is selected and executed

**Expected Results:**
- Click selects tool
- Click executes tool (form or immediate execution)
- Multiple tools can be selected and executed

**Test Categories:**
- [ ] Device Management tools
- [ ] Configuration tools
- [ ] Recon tools
- [ ] Capture tools

---

### Test 4: Tool Category - Scroll Wheel ✓

**Test Steps:**
1. In tool category with many tools
2. Use scroll wheel to navigate
3. Verify selection moves correctly

**Expected Results:**
- Scroll wheel navigates through tool list
- Selection updates visually
- Can reach all tools via scrolling

---

### Test 5: Settings Menu - Mouse Clicks ✓

**Test Steps:**
1. Press 's' or click Settings from main menu
2. Click on each settings option
3. Verify option activates

**Expected Results:**
- Click "Change Theme" opens theme selector
- Click "View Tool History" shows history
- Click "View Favorites" shows favorites
- All 6 settings options clickable

**Settings Options:**
- [ ] Change Theme
- [ ] View Tool History
- [ ] View Favorites
- [ ] Clear History
- [ ] Device Information
- [ ] About

---

### Test 6: Theme Selector - Mouse Clicks ✓

**Test Steps:**
1. Navigate to Settings → Change Theme
2. Click on each theme
3. Verify theme applies immediately

**Expected Results:**
- Click selects theme
- Theme applies instantly (colors change)
- Notification shows "Theme 'X' applied"
- Returns to settings menu

**Themes to Test:**
- [ ] Dark (default)
- [ ] Light
- [ ] Cyberpunk
- [ ] Solarized Dark
- [ ] Matrix

---

### Test 7: Theme Selector - Scroll Wheel ✓

**Test Steps:**
1. In theme selector
2. Use scroll wheel to navigate themes
3. Press Enter to apply selected theme

**Expected Results:**
- Scroll wheel moves through theme list
- Current theme marked with ★
- Can select and apply via scroll + Enter

---

### Test 8: Theme Selector - Quick Select ✓

**Test Steps:**
1. In theme selector
2. Press number keys 1-5
3. Verify theme selection jumps to that theme

**Expected Results:**
- Number keys select corresponding theme
- Works with both keyboard and mouse navigation

---

### Test 9: Packet Results - Mouse Clicks ✓

**Test Steps:**
1. Execute bt_decode or capture_list
2. Click on different packets in results
3. Verify packet is selected

**Expected Results:**
- Click selects packet
- Packet details expand/show
- Selected packet highlighted

**Note:** Requires actual capture data or mock data

---

### Test 10: Packet Results - Scroll Wheel ✓

**Test Steps:**
1. In packet results view
2. Use scroll wheel to navigate packet list
3. Verify smooth navigation

**Expected Results:**
- Scroll wheel moves through packets
- Large lists scroll smoothly
- Selection updates as you scroll

---

### Test 11: Help Overlay - Scroll Wheel ✓

**Test Steps:**
1. Press '?' to open help overlay
2. Use scroll wheel to scroll content
3. Verify help text scrolls

**Expected Results:**
- Scroll wheel scrolls help text up/down
- Scroll indicator shows position (e.g., "5/50")
- Can reach all help content via scrolling

**Note:** Click should not affect help (it's read-only)

---

### Test 12: Keyboard Navigation Still Works ✓

**Test Steps:**
1. Navigate using only keyboard (no mouse)
2. Verify all functionality accessible
3. Test all keyboard shortcuts

**Expected Results:**
- All features work without mouse
- Keyboard shortcuts function normally
- Help overlay (?) documents all shortcuts

**Key Shortcuts to Test:**
- [ ] ↑/↓ - Navigate
- [ ] Enter - Select
- [ ] Esc - Go back
- [ ] q - Quit
- [ ] s - Settings
- [ ] ? - Help
- [ ] 1-9 - Quick select

---

### Test 13: Mouse + Keyboard Combination ✓

**Test Steps:**
1. Use mouse to select item
2. Use keyboard to activate (Enter)
3. Mix mouse and keyboard navigation

**Expected Results:**
- Mouse and keyboard work together seamlessly
- Selection state consistent across input methods
- No conflicts or unexpected behavior

---

### Test 14: Terminal Compatibility ✓

**Test in Multiple Terminals:**
- [ ] Windows Terminal
- [ ] iTerm2 (macOS)
- [ ] Alacritty
- [ ] Kitty
- [ ] GNOME Terminal
- [ ] tmux (with mouse mode: `set -g mouse on`)
- [ ] SSH session

**Expected Results:**
- Mouse works in all modern terminals
- Graceful degradation in terminals without mouse
- Keyboard always works as fallback

---

### Test 15: Edge Cases ✓

**Test Steps:**
1. Click outside menu boundaries
2. Rapid clicking
3. Scroll at list boundaries
4. Click during tool execution

**Expected Results:**
- Clicks outside menus ignored safely
- Rapid clicks don't cause issues
- Scroll stops at boundaries (no crashes)
- Clicks during execution are handled gracefully

---

## Automated Tests (Future)

Would benefit from integration tests:

```rust
#[cfg(test)]
mod mouse_tests {
    use super::*;

    #[test]
    fn test_mouse_click_main_menu() {
        // Create app in main menu state
        // Simulate mouse click at row 5
        // Verify selection changed
    }

    #[test]
    fn test_scroll_wheel_navigation() {
        // Create app with list state
        // Simulate scroll events
        // Verify selection updates
    }

    #[test]
    fn test_click_outside_bounds() {
        // Simulate click at invalid position
        // Verify no crash, graceful handling
    }
}
```

---

## Known Issues to Watch For

1. **Click Position Off by One**: If clicks select wrong items, adjust the row calculation formula
2. **Scroll Too Sensitive**: If scroll wheel moves too fast, add scroll acceleration
3. **No Visual Feedback**: Items change selection but may not show hover state
4. **Terminal-Specific**: Some terminals may not report mouse events

---

## Success Criteria

✅ All menu items clickable
✅ Scroll wheel works in all lists
✅ Keyboard shortcuts still work
✅ No crashes with mouse input
✅ Works in major terminals
✅ Graceful degradation without mouse

---

## Test Results Template

```
Tested on: [Date]
Terminal: [Terminal Name/Version]
OS: [Operating System]

Test Results:
- Main Menu Clicks: ✅/❌
- Main Menu Scroll: ✅/❌
- Tool Category Clicks: ✅/❌
- Tool Category Scroll: ✅/❌
- Settings Clicks: ✅/❌
- Theme Selector: ✅/❌
- Packet Results: ✅/❌
- Help Scroll: ✅/❌
- Keyboard Still Works: ✅/❌
- Edge Cases: ✅/❌

Overall: ✅ PASS / ❌ FAIL

Notes:
[Any issues or observations]
```

---

## Quick Smoke Test (5 minutes)

For rapid verification:

1. ✅ Launch app: `cargo run --bin ubertooth-cli`
2. ✅ Click 3 different menu items
3. ✅ Scroll wheel up/down in menu
4. ✅ Press 's', click "Change Theme"
5. ✅ Click a theme, verify it applies
6. ✅ Press '?', scroll help with wheel
7. ✅ Press 'q' to quit
8. ✅ Verify no crashes or errors

If all pass → Mouse support working! ✅

---

## Reporting Issues

If you find issues, report with:
- Terminal name and version
- OS and version
- Exact steps to reproduce
- Expected vs actual behavior
- Screenshots if possible

---

## Next Steps After Testing

Once manual testing confirms mouse support works:
1. Document any issues found
2. Fix critical bugs
3. Add automated integration tests
4. Update MOUSE_SUPPORT.md with test results
5. Mark Phase 5.4 as verified ✅
