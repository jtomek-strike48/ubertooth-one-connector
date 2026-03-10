//! TUI application state and main loop

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use serde_json::Value;
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tui_textarea::TextArea;
use ubertooth_core::{PentestTool, ToolRegistry};
use ubertooth_platform::SidecarManager;
use ubertooth_tools::create_tool_registry;

use super::events::EventHandler;
use super::ui;
use super::views::{Category, ToolForm};

/// Result of tool execution
#[derive(Debug)]
pub enum ExecutionResult {
    Success(Value),
    Error(String),
}

/// Application state machine
pub enum AppState {
    /// Main menu - select tool category
    MainMenu { selected_index: usize },

    /// Tool category submenu - select specific tool
    ToolCategory {
        category: Category,
        selected_index: usize,
    },

    /// Tool parameter form
    ToolForm {
        form: Box<ToolForm>,
        error: Option<String>,
        hotkey_mode: bool,
    },

    /// Executing tool
    Executing {
        tool_name: String,
        // Receiver for execution results
        result_rx: Option<mpsc::Receiver<ExecutionResult>>,
        // Show as notification instead of full results page
        show_as_notification: bool,
    },

    /// Display results
    Results {
        tool_name: String,
        output: serde_json::Value,
        success: bool,
        selected_capture: Option<usize>,
        tool: Option<Arc<dyn PentestTool>>,
        // Packet list state for bt_decode
        packet_list_state: Option<PacketListState>,
        // Analysis view state for bt_analyze
        analysis_view_state: Option<AnalysisViewState>,
    },

    /// Settings page
    Settings { selected_index: usize },

    /// Confirmation dialog
    Confirmation {
        message: String,
        on_confirm: ConfirmAction,
    },

    /// Export menu for packet data
    ExportMenu {
        selected_index: usize,
        // Context from Results state
        packets: Vec<serde_json::Value>,
        packet_list_state: PacketListState,
        // Store full results state to return to
        previous_tool_name: String,
        previous_output: serde_json::Value,
        previous_success: bool,
    },

    /// Filter dialog for packet data
    FilterDialog {
        selected_section: usize,  // 0=packet types, 1=MAC, 2=RSSI, 3=actions
        selected_packet_type: usize,
        packet_type_selections: std::collections::HashSet<String>,
        mac_filter: String,
        rssi_min: String,
        rssi_max: String,
        // Store full results state to return to
        previous_tool_name: String,
        previous_output: serde_json::Value,
        previous_success: bool,
        previous_packet_list_state: PacketListState,
    },
}

/// Action to take on confirmation
#[derive(Debug)]
pub enum ConfirmAction {
    DeleteCapture(String),
}

/// Export options for packet data
#[derive(Debug, Clone, Copy)]
pub enum ExportOption {
    BookmarkedPackets,
    FilteredPackets,
    Statistics,
    ComparisonReport,
    TimelineData,
    AllPackets,
}

impl ExportOption {
    pub fn all() -> Vec<Self> {
        vec![
            Self::BookmarkedPackets,
            Self::FilteredPackets,
            Self::Statistics,
            Self::ComparisonReport,
            Self::TimelineData,
            Self::AllPackets,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::BookmarkedPackets => "Export Bookmarked Packets (JSON)",
            Self::FilteredPackets => "Export Filtered Packets (JSON)",
            Self::Statistics => "Export Statistics (JSON)",
            Self::ComparisonReport => "Export Comparison Report (Markdown)",
            Self::TimelineData => "Export Timeline Data (CSV)",
            Self::AllPackets => "Export All Packets (JSON)",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::BookmarkedPackets => "Export only packets marked with bookmarks (★)",
            Self::FilteredPackets => "Export packets matching current filters",
            Self::Statistics => "Export packet statistics and distribution data",
            Self::ComparisonReport => "Export side-by-side comparison of marked packets",
            Self::TimelineData => "Export timeline activity data as CSV",
            Self::AllPackets => "Export all packets in the current capture",
        }
    }
}

/// Text input dialog context
#[derive(Debug)]
pub enum DialogContext {
    /// Adding/editing annotation for packet index
    Annotation { packet_index: usize },
    /// Search functionality (future use)
    Search,
    /// Filter input (future use)
    Filter { field: String },
}

/// Text input dialog state
pub struct TextInputDialog {
    /// The textarea widget
    pub textarea: TextArea<'static>,
    /// What action to take when dialog is submitted
    pub context: DialogContext,
    /// Dialog title
    pub title: String,
}

/// Device connection status
#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub connected: bool,
    pub firmware: Option<String>,
}

/// Temporary notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub success: bool,
}

/// Analysis view state for bt_analyze
#[derive(Debug, Clone)]
pub struct AnalysisViewState {
    /// Currently selected item index (device/observation)
    pub selected_index: usize,
    /// Set of expanded item indices
    pub expanded_items: std::collections::HashSet<usize>,
    /// View mode (overview, devices, security, timing)
    pub view_mode: AnalysisViewMode,
    /// Scroll offset
    pub scroll_offset: usize,
}

/// View mode for analysis results
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisViewMode {
    Overview,
    Devices,
    Security,
    Timing,
}

impl AnalysisViewState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            expanded_items: std::collections::HashSet::new(),
            view_mode: AnalysisViewMode::Overview,
            scroll_offset: 0,
        }
    }

    pub fn toggle_expanded(&mut self, index: usize) {
        if self.expanded_items.contains(&index) {
            self.expanded_items.remove(&index);
        } else {
            self.expanded_items.insert(index);
        }
    }

    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded_items.contains(&index)
    }
}

/// Packet list state for bt_decode view
#[derive(Debug, Clone)]
pub struct PacketListState {
    /// Currently selected packet index
    pub selected_index: usize,
    /// Set of expanded packet indices
    pub expanded_packets: std::collections::HashSet<usize>,
    /// Scroll offset (for large lists)
    pub scroll_offset: usize,
    /// Active filters
    pub filters: PacketFilters,
    /// View mode (list, timeline, stats)
    pub view_mode: PacketViewMode,
    /// Bookmarked packet indices
    pub bookmarks: std::collections::HashSet<usize>,
    /// Packets marked for comparison
    pub comparison_marks: Vec<usize>,
    /// Follow stream MAC address
    pub follow_mac: Option<String>,
    /// Packet annotations (index -> note text)
    pub annotations: std::collections::HashMap<usize, String>,
}

/// View mode for packet list
#[derive(Debug, Clone, PartialEq)]
pub enum PacketViewMode {
    List,
    Statistics,
    Timeline,
    Comparison,
}

/// Filters for packet list
#[derive(Debug, Clone, Default)]
pub struct PacketFilters {
    pub packet_types: Vec<String>,  // Multi-select packet types
    pub mac_address: Option<String>,
    pub rssi_min: Option<i32>,
    pub rssi_max: Option<i32>,
    pub time_range: Option<(f64, f64)>,
}

impl PacketFilters {
    pub fn is_active(&self) -> bool {
        !self.packet_types.is_empty()
            || self.mac_address.is_some()
            || self.rssi_min.is_some()
            || self.rssi_max.is_some()
            || self.time_range.is_some()
    }

    pub fn clear(&mut self) {
        self.packet_types.clear();
        self.mac_address = None;
        self.rssi_min = None;
        self.rssi_max = None;
        self.time_range = None;
    }

    pub fn matches(&self, packet: &serde_json::Value) -> bool {
        // Check packet type filter
        if !self.packet_types.is_empty() {
            if let Some(pkt_type) = packet.get("packet_type").and_then(|t| t.as_str()) {
                if !self.packet_types.iter().any(|t| t == pkt_type) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check MAC address filter
        if let Some(ref filter_mac) = self.mac_address {
            if let Some(packet_mac) = packet.get("mac_address").and_then(|m| m.as_str()) {
                if !packet_mac.contains(filter_mac) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check RSSI range filter
        if self.rssi_min.is_some() || self.rssi_max.is_some() {
            if let Some(rssi_str) = packet.get("rssi").and_then(|r| r.as_str()) {
                if let Ok(rssi) = rssi_str.parse::<i32>() {
                    if let Some(min) = self.rssi_min {
                        if rssi < min {
                            return false;
                        }
                    }
                    if let Some(max) = self.rssi_max {
                        if rssi > max {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl PacketListState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            expanded_packets: std::collections::HashSet::new(),
            scroll_offset: 0,
            filters: PacketFilters::default(),
            view_mode: PacketViewMode::List,
            bookmarks: std::collections::HashSet::new(),
            comparison_marks: Vec::new(),
            follow_mac: None,
            annotations: std::collections::HashMap::new(),
        }
    }

    pub fn toggle_expanded(&mut self, index: usize) {
        if self.expanded_packets.contains(&index) {
            self.expanded_packets.remove(&index);
        } else {
            self.expanded_packets.insert(index);
        }
    }

    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded_packets.contains(&index)
    }

    pub fn toggle_bookmark(&mut self, index: usize) {
        if self.bookmarks.contains(&index) {
            self.bookmarks.remove(&index);
        } else {
            self.bookmarks.insert(index);
        }
    }

    pub fn is_bookmarked(&self, index: usize) -> bool {
        self.bookmarks.contains(&index)
    }

    pub fn toggle_comparison_mark(&mut self, index: usize) {
        if let Some(pos) = self.comparison_marks.iter().position(|&x| x == index) {
            self.comparison_marks.remove(pos);
        } else {
            if self.comparison_marks.len() < 2 {
                self.comparison_marks.push(index);
            }
        }
    }

    pub fn is_marked_for_comparison(&self, index: usize) -> bool {
        self.comparison_marks.contains(&index)
    }

    pub fn add_annotation(&mut self, index: usize, note: String) {
        if !note.is_empty() {
            self.annotations.insert(index, note);
        }
    }

    pub fn remove_annotation(&mut self, index: usize) {
        self.annotations.remove(&index);
    }

    pub fn get_annotation(&self, index: usize) -> Option<&String> {
        self.annotations.get(&index)
    }

    pub fn has_annotation(&self, index: usize) -> bool {
        self.annotations.contains_key(&index)
    }
}

/// Main TUI application
pub struct App {
    /// Current application state
    state: AppState,

    /// Tool registry
    registry: Arc<ToolRegistry>,

    /// Device connection status
    device_status: DeviceStatus,

    /// Temporary notification (cleared on next input)
    notification: Option<Notification>,

    /// Frame counter for animations
    frame_count: u64,

    /// Tool execution history (last 10)
    tool_history: Vec<String>,

    /// Favorited/bookmarked tools
    favorites: Vec<String>,

    /// Recently seen MAC addresses (for filtering)
    recent_macs: Vec<String>,

    /// Active text input dialog (overlays on current state)
    dialog: Option<TextInputDialog>,

    /// Should quit?
    should_quit: bool,
}

impl App {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        // Create backend (default to Python for now, will add config later)
        let backend = SidecarManager::new();
        let registry = Arc::new(create_tool_registry(backend));

        Ok(Self {
            state: AppState::MainMenu { selected_index: 0 },
            registry,
            device_status: DeviceStatus {
                connected: false,
                firmware: None,
            },
            notification: None,
            frame_count: 0,
            tool_history: Vec::new(),
            favorites: Vec::new(),
            recent_macs: Vec::new(),
            dialog: None,
            should_quit: false,
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Install panic hook to restore terminal on panic
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Restore terminal
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            // Call original hook
            original_hook(panic_info);
        }));

        // Create event handler
        let mut events = EventHandler::new(250); // 250ms tick rate

        // Main loop with error recovery
        let result = (|| -> Result<()> {
            loop {
                // Increment frame counter for animations
                self.frame_count = self.frame_count.wrapping_add(1);

                // Render UI (catch and log any render errors)
                if let Err(e) = terminal.draw(|f| ui::render(f, &self.state, &self.registry, &self.device_status, &self.notification, self.frame_count, &self.dialog)) {
                    tracing::error!("Render error: {}", e);
                    // Continue anyway - might be transient
                }

                // Check for tool execution results
                if let AppState::Executing { tool_name, result_rx, show_as_notification } = &mut self.state {
                    if let Some(rx) = result_rx {
                        match rx.try_recv() {
                            Ok(result) => {
                                // Move out of executing state
                                let tool_name = tool_name.clone();
                                let show_notification = *show_as_notification;

                                match result {
                                    ExecutionResult::Success(output) => {
                                        // Update device status if this was a device_connect command
                                        if tool_name == "device_connect" {
                                            if let Some(firmware) = output.get("firmware_version").and_then(|v| v.as_str()) {
                                                self.device_status.connected = true;
                                                self.device_status.firmware = Some(firmware.to_string());
                                            }
                                        } else if tool_name == "device_disconnect" {
                                            self.device_status.connected = false;
                                            self.device_status.firmware = None;
                                        }

                                        // Extract MAC addresses from analysis/scan results
                                        if tool_name.starts_with("bt_analyze") ||
                                           tool_name.starts_with("btle_scan") ||
                                           tool_name == "capture_list" {
                                            self.extract_macs_from_output(&output);
                                        }

                                        if show_notification {
                                            // Show as notification and return to main menu
                                            let message = if tool_name == "device_connect" {
                                                "Successfully connected to Ubertooth".to_string()
                                            } else if tool_name == "device_disconnect" {
                                                "Successfully disconnected from Ubertooth".to_string()
                                            } else if tool_name == "capture_delete" {
                                                "Capture deleted successfully".to_string()
                                            } else {
                                                format!("{} completed successfully", tool_name)
                                            };
                                            self.notification = Some(Notification {
                                                message,
                                                success: true,
                                            });
                                            self.state = AppState::MainMenu { selected_index: 0 };
                                        } else {
                                            // Check if this is capture_list with results
                                            let selected_capture = if tool_name == "capture_list" {
                                                output.get("captures")
                                                    .and_then(|c| c.as_array())
                                                    .filter(|arr| !arr.is_empty())
                                                    .map(|_| 0) // Select first capture
                                            } else {
                                                None
                                            };

                                            // Initialize packet list state for bt_decode
                                            let packet_list_state = if tool_name == "bt_decode" {
                                                Some(PacketListState::new())
                                            } else {
                                                None
                                            };

                                            // Initialize analysis view state for bt_analyze
                                            let analysis_view_state = if tool_name == "bt_analyze" {
                                                Some(AnalysisViewState::new())
                                            } else {
                                                None
                                            };

                                            self.state = AppState::Results {
                                                tool_name,
                                                output,
                                                success: true,
                                                selected_capture,
                                                tool: None, // TODO: Store tool for re-parameterization
                                                packet_list_state,
                                                analysis_view_state,
                                            };
                                        }
                                    }
                                    ExecutionResult::Error(error) => {
                                        if show_notification {
                                            // Show as notification and return to main menu
                                            let message = if tool_name == "device_connect" {
                                                format!("Failed to connect: {}", error)
                                            } else if tool_name == "device_disconnect" {
                                                format!("Failed to disconnect: {}", error)
                                            } else {
                                                format!("{} failed: {}", tool_name, error)
                                            };
                                            self.notification = Some(Notification {
                                                message,
                                                success: false,
                                            });
                                            self.state = AppState::MainMenu { selected_index: 0 };
                                        } else {
                                            self.state = AppState::Results {
                                                tool_name,
                                                output: serde_json::json!({ "error": error }),
                                                success: false,
                                                selected_capture: None,
                                                tool: None,
                                                packet_list_state: None,
                    analysis_view_state: None,
                                            };
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Channel still empty or closed, continue
                            }
                        }
                    }
                }

                // Handle events
                if let Some(event) = events.next()? {
                    if let Err(e) = self.handle_event(event) {
                        tracing::error!("Event handling error: {}", e);
                        // Show error to user
                        self.state = AppState::Results {
                            tool_name: "Error".to_string(),
                            output: serde_json::json!({ "error": format!("{}", e) }),
                            success: false,
                            selected_capture: None,
                            tool: None,
                            packet_list_state: None,
                    analysis_view_state: None,
                        };
                    }
                }

                if self.should_quit {
                    break;
                }
            }
            Ok(())
        })();

        // Restore terminal (always runs, even on error)
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    /// Handle an input event
    fn handle_event(&mut self, event: crossterm::event::Event) -> Result<()> {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        // Clear notification on any key press
        if matches!(event, Event::Key(_)) {
            self.notification = None;
        }

        // Handle text input dialog (highest priority)
        if self.dialog.is_some() {
            if let Event::Key(KeyEvent { code, .. }) = event {
                match code {
                    KeyCode::Esc => {
                        // Cancel dialog
                        self.dialog = None;
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        // Submit dialog
                        if let Some(dialog) = self.dialog.take() {
                            self.handle_dialog_submit(dialog)?;
                        }
                        return Ok(());
                    }
                    _ => {
                        // Pass input to textarea
                        if let Some(dialog) = &mut self.dialog {
                            dialog.textarea.input(event);
                        }
                        return Ok(());
                    }
                }
            }
            return Ok(());
        }

        // Handle confirmation dialog
        if let AppState::Confirmation { on_confirm, .. } = &self.state {
            if let Event::Key(KeyEvent { code, .. }) = event {
                match code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Execute the confirmation action
                        let action = match on_confirm {
                            ConfirmAction::DeleteCapture(id) => ConfirmAction::DeleteCapture(id.clone()),
                        };

                        // Return to main menu first
                        self.state = AppState::MainMenu { selected_index: 0 };

                        // Execute the action
                        match action {
                            ConfirmAction::DeleteCapture(id) => {
                                self.execute_delete_capture(id)?;
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Cancel - return to main menu
                        self.state = AppState::MainMenu { selected_index: 0 };
                        return Ok(());
                    }
                    _ => {
                        return Ok(());
                    }
                }
            }
            return Ok(());
        }

        // Handle form input specially
        if let AppState::ToolForm { form, error, hotkey_mode } = &mut self.state {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
                // Hotkey mode - direct parameter selection
                if *hotkey_mode {
                    match code {
                        KeyCode::Esc => {
                            self.go_back();
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            // Execute with current parameters
                            self.execute_tool()?;
                            return Ok(());
                        }
                        KeyCode::Char(ch) => {
                            // Handle hotkey parameter changes
                            let ch_upper = ch.to_uppercase().next().unwrap();

                            // Map hotkeys to field names
                            let field_name: Option<String> = match ch_upper {
                                'D' => Some("duration_sec".to_string()),
                                'C' => Some("channel".to_string()),
                                'S' => Some("save_pcap".to_string()),
                                'A' => Some("analysis_type".to_string()),
                                _ => {
                                    // Try to find field starting with this letter
                                    form.fields()
                                        .iter()
                                        .find(|f| f.name.chars().next().unwrap_or('?').to_uppercase().next().unwrap() == ch_upper)
                                        .map(|f| f.name.clone())
                                }
                            };

                            if let Some(field) = field_name {
                                // Cycle through values for this field
                                form.cycle_field(&field);
                                *error = None;
                            }
                            return Ok(());
                        }
                        KeyCode::Char(digit) if digit.is_ascii_digit() && digit >= '1' && digit <= '9' => {
                            // Number keys for duration_sec quick select
                            let durations = vec!["5", "10", "30", "60", "120"];
                            let idx = digit.to_digit(10).unwrap() as usize - 1;
                            if idx < durations.len() {
                                form.set_field_value("duration_sec", durations[idx].to_string());
                                *error = None;
                            }
                            return Ok(());
                        }
                        _ => {
                            return Ok(());
                        }
                    }
                }

                // Traditional form mode
                let is_dropdown = matches!(
                    form.input_modes().get(form.focused_index()),
                    Some(super::views::FieldInputMode::Dropdown { .. })
                );

                match code {
                    KeyCode::Esc => {
                        self.go_back();
                        return Ok(());
                    }
                    KeyCode::Tab => {
                        if modifiers.contains(KeyModifiers::SHIFT) {
                            form.focus_prev();
                        } else {
                            form.focus_next();
                        }
                        return Ok(());
                    }
                    KeyCode::Up if is_dropdown => {
                        form.dropdown_prev();
                        return Ok(());
                    }
                    KeyCode::Down if is_dropdown => {
                        form.dropdown_next();
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        self.execute_tool()?;
                        return Ok(());
                    }
                    _ => {
                        if !is_dropdown {
                            if let Some(input) = form.focused_input_mut() {
                                input.input(event);
                            }
                        }
                        *error = None;
                        return Ok(());
                    }
                }
            }
            return Ok(());
        }

        // Handle capture_list navigation
        #[derive(Debug)]
        enum CaptureAction {
            Analyze(String),
            Delete(String),
            View(String),
            Export(String),
            Tag(String),
        }

        let capture_action = if let AppState::Results { tool_name, output, success, selected_capture, .. } = &mut self.state {
            if *tool_name == "capture_list" && *success {
                if let Event::Key(KeyEvent { code, .. }) = event {
                    match code {
                        KeyCode::Up => {
                            // Move selection up
                            if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                if !captures.is_empty() {
                                    if let Some(idx) = selected_capture {
                                        if *idx > 0 {
                                            *idx -= 1;
                                        }
                                    } else {
                                        *selected_capture = Some(0);
                                    }
                                }
                            }
                            return Ok(());
                        }
                        KeyCode::Down => {
                            // Move selection down
                            if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                if !captures.is_empty() {
                                    if let Some(idx) = selected_capture {
                                        if *idx < captures.len() - 1 {
                                            *idx += 1;
                                        }
                                    } else {
                                        *selected_capture = Some(0);
                                    }
                                }
                            }
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            // Analyze
                            if let Some(idx) = selected_capture {
                                if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                    if let Some(capture) = captures.get(*idx) {
                                        if let Some(capture_id) = capture.get("capture_id").and_then(|v| v.as_str()) {
                                            Some(CaptureAction::Analyze(capture_id.to_string()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            // Delete
                            if let Some(idx) = selected_capture {
                                if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                    if let Some(capture) = captures.get(*idx) {
                                        if let Some(capture_id) = capture.get("capture_id").and_then(|v| v.as_str()) {
                                            Some(CaptureAction::Delete(capture_id.to_string()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        KeyCode::Char('v') | KeyCode::Char('V') => {
                            // View details
                            if let Some(idx) = selected_capture {
                                if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                    if let Some(capture) = captures.get(*idx) {
                                        if let Some(capture_id) = capture.get("capture_id").and_then(|v| v.as_str()) {
                                            Some(CaptureAction::View(capture_id.to_string()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            // Export
                            if let Some(idx) = selected_capture {
                                if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                    if let Some(capture) = captures.get(*idx) {
                                        if let Some(capture_id) = capture.get("capture_id").and_then(|v| v.as_str()) {
                                            Some(CaptureAction::Export(capture_id.to_string()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            // Tag
                            if let Some(idx) = selected_capture {
                                if let Some(captures) = output.get("captures").and_then(|c| c.as_array()) {
                                    if let Some(capture) = captures.get(*idx) {
                                        if let Some(capture_id) = capture.get("capture_id").and_then(|v| v.as_str()) {
                                            Some(CaptureAction::Tag(capture_id.to_string()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        KeyCode::Esc => {
                            self.go_back();
                            return Ok(());
                        }
                        _ => None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Execute capture action
        if let Some(action) = capture_action {
            match action {
                CaptureAction::Analyze(id) => self.launch_analysis(id)?,
                CaptureAction::Delete(id) => self.launch_capture_delete(id)?,
                CaptureAction::View(id) => self.launch_capture_get(id)?,
                CaptureAction::Export(id) => self.launch_capture_export(id)?,
                CaptureAction::Tag(id) => self.launch_capture_tag(id)?,
            }
            return Ok(());
        }

        // Handle bt_decode packet list navigation
        if let AppState::Results { tool_name, output, success, packet_list_state, .. } = &mut self.state {
            if *tool_name == "bt_decode" {
                if let Some(pls) = packet_list_state {
                    if let Event::Key(KeyEvent { code, .. }) = event {
                        if let Some(packets) = output.get("decoded_packets").and_then(|p| p.as_array()) {
                            let packet_count = packets.len();

                            match code {
                                KeyCode::Up => {
                                    if pls.selected_index > 0 {
                                        pls.selected_index -= 1;
                                        // Adjust scroll if needed
                                        if pls.selected_index < pls.scroll_offset {
                                            pls.scroll_offset = pls.selected_index;
                                        }
                                    }
                                    return Ok(());
                                }
                                KeyCode::Down => {
                                    if pls.selected_index < packet_count.saturating_sub(1) {
                                        pls.selected_index += 1;
                                        // Adjust scroll if needed (assume 20 visible lines)
                                        let visible_lines = 20;
                                        if pls.selected_index >= pls.scroll_offset + visible_lines {
                                            pls.scroll_offset = pls.selected_index - visible_lines + 1;
                                        }
                                    }
                                    return Ok(());
                                }
                                KeyCode::PageUp => {
                                    pls.selected_index = pls.selected_index.saturating_sub(10);
                                    pls.scroll_offset = pls.scroll_offset.saturating_sub(10);
                                    return Ok(());
                                }
                                KeyCode::PageDown => {
                                    pls.selected_index = (pls.selected_index + 10).min(packet_count.saturating_sub(1));
                                    pls.scroll_offset = (pls.scroll_offset + 10).min(packet_count.saturating_sub(20));
                                    return Ok(());
                                }
                                KeyCode::Home => {
                                    pls.selected_index = 0;
                                    pls.scroll_offset = 0;
                                    return Ok(());
                                }
                                KeyCode::End => {
                                    pls.selected_index = packet_count.saturating_sub(1);
                                    pls.scroll_offset = packet_count.saturating_sub(20);
                                    return Ok(());
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    // Toggle expand/collapse
                                    pls.toggle_expanded(pls.selected_index);
                                    return Ok(());
                                }
                                KeyCode::Char('s') | KeyCode::Char('S') => {
                                    // Toggle statistics view
                                    pls.view_mode = match pls.view_mode {
                                        PacketViewMode::List => PacketViewMode::Statistics,
                                        PacketViewMode::Statistics => PacketViewMode::List,
                                        PacketViewMode::Timeline => PacketViewMode::Statistics,
                                        PacketViewMode::Comparison => PacketViewMode::Statistics,
                                    };
                                    return Ok(());
                                }
                                KeyCode::Char('t') | KeyCode::Char('T') => {
                                    // Toggle timeline view
                                    pls.view_mode = match pls.view_mode {
                                        PacketViewMode::List => PacketViewMode::Timeline,
                                        PacketViewMode::Timeline => PacketViewMode::List,
                                        PacketViewMode::Statistics => PacketViewMode::Timeline,
                                        PacketViewMode::Comparison => PacketViewMode::Timeline,
                                    };
                                    return Ok(());
                                }
                                KeyCode::Char('b') | KeyCode::Char('B') => {
                                    // Toggle bookmark
                                    pls.toggle_bookmark(pls.selected_index);
                                    return Ok(());
                                }
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    // Mark for comparison
                                    pls.toggle_comparison_mark(pls.selected_index);
                                    return Ok(());
                                }
                                KeyCode::Char('f') | KeyCode::Char('F') => {
                                    // Follow stream - toggle following the MAC of selected packet
                                    if let Some(packet) = packets.get(pls.selected_index) {
                                        let mac = packet.get("mac_address")
                                            .and_then(|m| m.as_str())
                                            .map(|s| s.to_string());

                                        if pls.follow_mac.is_some() {
                                            // Clear follow
                                            pls.follow_mac = None;
                                        } else {
                                            // Start following
                                            pls.follow_mac = mac;
                                        }
                                    }
                                    return Ok(());
                                }
                                KeyCode::Char('l') | KeyCode::Char('L') => {
                                    // Return to list view
                                    pls.view_mode = PacketViewMode::List;
                                    return Ok(());
                                }
                                KeyCode::Char('c') | KeyCode::Char('C') => {
                                    // Open comparison view if 2 packets are marked
                                    if pls.comparison_marks.len() == 2 {
                                        pls.view_mode = PacketViewMode::Comparison;
                                    }
                                    return Ok(());
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') => {
                                    // Open text input dialog for annotation
                                    let packet_index = pls.selected_index;
                                    let existing_note = pls.get_annotation(packet_index).cloned().unwrap_or_default();

                                    let mut textarea = TextArea::default();
                                    if !existing_note.is_empty() {
                                        // Pre-fill with existing annotation
                                        textarea = TextArea::new(vec![existing_note]);
                                    }

                                    self.dialog = Some(TextInputDialog {
                                        textarea,
                                        context: DialogContext::Annotation { packet_index },
                                        title: format!("Annotation for Packet #{}", packet_index),
                                    });
                                    return Ok(());
                                }
                                KeyCode::Delete | KeyCode::Backspace => {
                                    // Remove annotation from selected packet (when in list view and has annotation)
                                    if pls.view_mode == PacketViewMode::List && pls.has_annotation(pls.selected_index) {
                                        pls.remove_annotation(pls.selected_index);
                                    }
                                    return Ok(());
                                }
                                KeyCode::Char('e') | KeyCode::Char('E') => {
                                    // Open export menu - store current state to return to
                                    let tool_name_clone = tool_name.clone();
                                    let output_clone = output.clone();
                                    let success_clone = *success;

                                    self.state = AppState::ExportMenu {
                                        selected_index: 0,
                                        packets: packets.clone(),
                                        packet_list_state: pls.clone(),
                                        previous_tool_name: tool_name_clone,
                                        previous_output: output_clone,
                                        previous_success: success_clone,
                                    };
                                    return Ok(());
                                }
                                KeyCode::Char('/') => {
                                    // Open filter dialog
                                    let tool_name_clone = tool_name.clone();
                                    let output_clone = output.clone();
                                    let success_clone = *success;
                                    let pls_clone = pls.clone();

                                    // Prepopulate with existing filters
                                    let packet_type_selections = pls.filters.packet_types.iter().cloned().collect();
                                    let mac_filter = pls.filters.mac_address.clone().unwrap_or_default();
                                    let rssi_min = pls.filters.rssi_min.map(|v| v.to_string()).unwrap_or_default();
                                    let rssi_max = pls.filters.rssi_max.map(|v| v.to_string()).unwrap_or_default();

                                    self.state = AppState::FilterDialog {
                                        selected_section: 0,
                                        selected_packet_type: 0,
                                        packet_type_selections,
                                        mac_filter,
                                        rssi_min,
                                        rssi_max,
                                        previous_tool_name: tool_name_clone,
                                        previous_output: output_clone,
                                        previous_success: success_clone,
                                        previous_packet_list_state: pls_clone,
                                    };
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Handle bt_analyze analysis view navigation
        if let AppState::Results { tool_name, output, analysis_view_state, .. } = &mut self.state {
            if *tool_name == "bt_analyze" {
                if let Some(avs) = analysis_view_state {
                    if let Event::Key(KeyEvent { code, .. }) = event {
                        match code {
                            KeyCode::Char('o') | KeyCode::Char('O') => {
                                // Switch to overview mode
                                avs.view_mode = AnalysisViewMode::Overview;
                                return Ok(());
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                // Switch to devices mode
                                avs.view_mode = AnalysisViewMode::Devices;
                                return Ok(());
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // Switch to security mode
                                avs.view_mode = AnalysisViewMode::Security;
                                return Ok(());
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                // Switch to timing mode
                                avs.view_mode = AnalysisViewMode::Timing;
                                return Ok(());
                            }
                            KeyCode::Up => {
                                if avs.selected_index > 0 {
                                    avs.selected_index -= 1;
                                    if avs.selected_index < avs.scroll_offset {
                                        avs.scroll_offset = avs.selected_index;
                                    }
                                }
                                return Ok(());
                            }
                            KeyCode::Down => {
                                // Determine max index based on view mode
                                let max_index = match avs.view_mode {
                                    AnalysisViewMode::Devices => {
                                        output.get("analysis")
                                            .and_then(|a| a.get("devices"))
                                            .and_then(|d| d.as_array())
                                            .map(|arr| arr.len().saturating_sub(1))
                                            .unwrap_or(0)
                                    }
                                    AnalysisViewMode::Security => {
                                        output.get("analysis")
                                            .and_then(|a| a.get("security_observations"))
                                            .and_then(|s| s.as_array())
                                            .map(|arr| arr.len().saturating_sub(1))
                                            .unwrap_or(0)
                                    }
                                    _ => 0,
                                };

                                if avs.selected_index < max_index {
                                    avs.selected_index += 1;
                                    let visible_lines = 20;
                                    if avs.selected_index >= avs.scroll_offset + visible_lines {
                                        avs.scroll_offset = avs.selected_index - visible_lines + 1;
                                    }
                                }
                                return Ok(());
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                // Toggle expansion for devices/security items
                                if matches!(avs.view_mode, AnalysisViewMode::Devices | AnalysisViewMode::Security) {
                                    avs.toggle_expanded(avs.selected_index);
                                }
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Handle export menu
        if let AppState::ExportMenu { selected_index, packets, packet_list_state, previous_tool_name, previous_output, previous_success } = &mut self.state {
            if let Event::Key(KeyEvent { code, .. }) = event {
                let options = ExportOption::all();
                match code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Down => {
                        if *selected_index < options.len().saturating_sub(1) {
                            *selected_index += 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        // Execute export
                        let option = options[*selected_index];
                        let packets_clone = packets.clone();
                        let state_clone = packet_list_state.clone();

                        // Store previous state to restore
                        let prev_tool = previous_tool_name.clone();
                        let prev_output = previous_output.clone();
                        let prev_success = *previous_success;
                        let prev_state = packet_list_state.clone();

                        // Return to results view
                        self.state = AppState::Results {
                            tool_name: prev_tool,
                            output: prev_output,
                            success: prev_success,
                            selected_capture: None,
                            tool: None,
                            packet_list_state: Some(prev_state),
                    analysis_view_state: None,
                        };

                        // Perform export
                        match self.export_packets(option, packets_clone, state_clone) {
                            Ok(path) => {
                                self.notification = Some(Notification {
                                    message: format!("Exported to: {}", path),
                                    success: true,
                                });
                            }
                            Err(e) => {
                                self.notification = Some(Notification {
                                    message: format!("Export failed: {}", e),
                                    success: false,
                                });
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        // Restore previous results state
                        let prev_tool = previous_tool_name.clone();
                        let prev_output = previous_output.clone();
                        let prev_success = *previous_success;
                        let prev_state = packet_list_state.clone();

                        self.state = AppState::Results {
                            tool_name: prev_tool,
                            output: prev_output,
                            success: prev_success,
                            selected_capture: None,
                            tool: None,
                            packet_list_state: Some(prev_state),
                    analysis_view_state: None,
                        };
                        return Ok(());
                    }
                    _ => {}
                }
            }
            return Ok(());
        }

        // Handle filter dialog
        if let AppState::FilterDialog {
            selected_section,
            selected_packet_type,
            packet_type_selections,
            mac_filter,
            rssi_min,
            rssi_max,
            previous_tool_name,
            previous_output,
            previous_success,
            previous_packet_list_state,
        } = &mut self.state {
            if let Event::Key(KeyEvent { code, .. }) = event {
                let packet_types = vec!["ADV_IND", "SCAN_REQ", "SCAN_RSP", "CONNECT_REQ", "DATA"];

                match code {
                    KeyCode::Up => {
                        if *selected_section > 0 {
                            *selected_section -= 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Down => {
                        if *selected_section < 3 {  // 0=types, 1=MAC, 2=RSSI, 3=actions
                            *selected_section += 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Left => {
                        if *selected_section == 0 && *selected_packet_type > 0 {
                            *selected_packet_type -= 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Right => {
                        if *selected_section == 0 && *selected_packet_type < packet_types.len().saturating_sub(1) {
                            *selected_packet_type += 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Char(' ') => {
                        // Toggle packet type selection
                        if *selected_section == 0 {
                            let pkt_type = packet_types[*selected_packet_type].to_string();
                            if packet_type_selections.contains(&pkt_type) {
                                packet_type_selections.remove(&pkt_type);
                            } else {
                                packet_type_selections.insert(pkt_type);
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        // Input for text fields
                        match *selected_section {
                            1 => {  // MAC filter
                                mac_filter.push(c);
                            }
                            2 => {  // RSSI (assume we're on min for now, can toggle later)
                                if c.is_ascii_digit() || c == '-' {
                                    if rssi_min.len() < 4 {  // -120 to -30 typically
                                        rssi_min.push(c);
                                    }
                                }
                            }
                            _ => {}
                        }
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        // Remove character from text fields
                        match *selected_section {
                            1 => {
                                mac_filter.pop();
                            }
                            2 => {
                                rssi_min.pop();
                            }
                            _ => {}
                        }
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        // Apply filters
                        if *selected_section == 3 {  // "Apply" action
                            let mut new_pls = previous_packet_list_state.clone();
                            new_pls.filters.packet_types = packet_type_selections.iter().cloned().collect();
                            new_pls.filters.mac_address = if mac_filter.is_empty() {
                                None
                            } else {
                                Some(mac_filter.clone())
                            };
                            new_pls.filters.rssi_min = rssi_min.parse().ok();
                            new_pls.filters.rssi_max = rssi_max.parse().ok();

                            self.state = AppState::Results {
                                tool_name: previous_tool_name.clone(),
                                output: previous_output.clone(),
                                success: *previous_success,
                                selected_capture: None,
                                tool: None,
                                packet_list_state: Some(new_pls),
                    analysis_view_state: None,
                            };

                            self.notification = Some(Notification {
                                message: "Filters applied".to_string(),
                                success: true,
                            });
                        }
                        return Ok(());
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        // Clear all filters (only when focused on actions section)
                        if *selected_section == 3 {
                            packet_type_selections.clear();
                            mac_filter.clear();
                            rssi_min.clear();
                            rssi_max.clear();

                            let mut new_pls = previous_packet_list_state.clone();
                            new_pls.filters.clear();

                            self.state = AppState::Results {
                                tool_name: previous_tool_name.clone(),
                                output: previous_output.clone(),
                                success: *previous_success,
                                selected_capture: None,
                                tool: None,
                                packet_list_state: Some(new_pls),
                    analysis_view_state: None,
                            };

                            self.notification = Some(Notification {
                                message: "Filters cleared".to_string(),
                                success: true,
                            });
                        }
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        // Cancel - restore previous state
                        self.state = AppState::Results {
                            tool_name: previous_tool_name.clone(),
                            output: previous_output.clone(),
                            success: *previous_success,
                            selected_capture: None,
                            tool: None,
                            packet_list_state: Some(previous_packet_list_state.clone()),
                    analysis_view_state: None,
                        };
                        return Ok(());
                    }
                    _ => {}
                }
            }
            return Ok(());
        }

        // Normal navigation
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('s') => {
                    // Open settings
                    self.state = AppState::Settings { selected_index: 0 };
                }
                KeyCode::Up => {
                    self.move_selection(-1);
                }
                KeyCode::Down => {
                    self.move_selection(1);
                }
                KeyCode::Right => {
                    // Right arrow: jump to Device Status when on connect/disconnect toggle
                    if let AppState::ToolCategory { category, selected_index } = &mut self.state {
                        if matches!(category, Category::DeviceManagement) && *selected_index == 0 {
                            // Jump to Device Status (index 1 after filtering)
                            *selected_index = 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    self.handle_selection()?;
                }
                KeyCode::Esc => {
                    self.go_back();
                }
                KeyCode::Char(ch) if ch.is_ascii_digit() => {
                    // Number key shortcuts for quick selection
                    self.handle_number_shortcut(ch)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Move selection up or down
    fn move_selection(&mut self, delta: i32) {
        match &mut self.state {
            AppState::MainMenu { selected_index } => {
                let new_index = (*selected_index as i32 + delta).max(0).min(6) as usize;
                *selected_index = new_index;
            }
            AppState::ToolCategory { selected_index, category } => {
                // Use filtered tool count for DeviceManagement
                let device_connected = if matches!(category, Category::DeviceManagement) {
                    Some(self.device_status.connected)
                } else {
                    None
                };
                let tool_count = category.tool_count_filtered(&self.registry, device_connected);
                let new_index = (*selected_index as i32 + delta).max(0).min(tool_count as i32 - 1) as usize;
                *selected_index = new_index;
            }
            AppState::Settings { selected_index } => {
                // 6 settings options
                let new_index = (*selected_index as i32 + delta).max(0).min(5) as usize;
                *selected_index = new_index;
            }
            _ => {}
        }
    }

    /// Handle number key shortcuts for quick selection
    fn handle_number_shortcut(&mut self, ch: char) -> Result<()> {
        let digit = ch.to_digit(10).unwrap() as usize;

        match &mut self.state {
            AppState::MainMenu { selected_index } => {
                // Main menu: 0 = connection toggle, 1-6 = categories
                if digit <= 6 {
                    *selected_index = digit;
                    // Auto-select on number press
                    self.handle_selection()?;
                }
            }
            AppState::ToolCategory { selected_index, category } => {
                // Tool category: 1-9 = tools (1-indexed in UI)
                if digit >= 1 && digit <= 9 {
                    let device_connected = if matches!(category, Category::DeviceManagement) {
                        Some(self.device_status.connected)
                    } else {
                        None
                    };
                    let tool_count = category.tool_count_filtered(&self.registry, device_connected);
                    let target_index = digit - 1; // Convert to 0-indexed

                    if target_index < tool_count {
                        *selected_index = target_index;
                        // Auto-select on number press
                        self.handle_selection()?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle Settings selection
    fn handle_settings_selection(&mut self, selected_index: usize) -> Result<()> {
        match selected_index {
            0 => {
                // View Tool History
                let history_data = serde_json::json!({
                    "history": self.tool_history.clone(),
                    "count": self.tool_history.len()
                });
                self.state = AppState::Results {
                    tool_name: "Tool History".to_string(),
                    output: history_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            1 => {
                // View Favorites
                let favorites_data = serde_json::json!({
                    "favorites": self.favorites.clone(),
                    "count": self.favorites.len()
                });
                self.state = AppState::Results {
                    tool_name: "Favorited Tools".to_string(),
                    output: favorites_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            2 => {
                // View Recent MAC Addresses
                let mac_data = serde_json::json!({
                    "recent_macs": self.recent_macs.clone(),
                    "count": self.recent_macs.len(),
                    "description": "Recent MAC addresses from scans and analysis"
                });
                self.state = AppState::Results {
                    tool_name: "Recent MAC Addresses".to_string(),
                    output: mac_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            3 => {
                // Backend Info
                let backend_data = serde_json::json!({
                    "backend": "Rust (native USB) with Python fallback",
                    "device_detection": "Auto-detect first Ubertooth",
                    "usb_library": "libusb via FFI"
                });
                self.state = AppState::Results {
                    tool_name: "Backend Configuration".to_string(),
                    output: backend_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            4 => {
                // Strike48 Connection
                let strike48_data = serde_json::json!({
                    "server_url": "wss://jt-demo-01.strike48.engineering",
                    "tenant_id": "non-prod",
                    "status": "Not configured"
                });
                self.state = AppState::Results {
                    tool_name: "Strike48 Connection".to_string(),
                    output: strike48_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            5 => {
                // About
                let about_data = serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "build": "release",
                    "tools_count": self.registry.tools().len()
                });
                self.state = AppState::Results {
                    tool_name: "About Ubertooth TUI".to_string(),
                    output: about_data,
                    success: true,
                    selected_capture: None,
                    tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Enter key on current selection
    fn handle_selection(&mut self) -> Result<()> {
        match &self.state {
            AppState::Settings { selected_index } => {
                return self.handle_settings_selection(*selected_index);
            }
            AppState::MainMenu { selected_index } => {
                // Index 0 is the device connection toggle
                if *selected_index == 0 {
                    // Execute device_connect or device_disconnect based on state
                    let tool_name = if self.device_status.connected {
                        "device_disconnect"
                    } else {
                        "device_connect"
                    };

                    if let Some(tool) = self.registry.get(tool_name) {
                        if let Ok(form) = ToolForm::new(tool.clone()) {
                            let params = form.build_params();
                            let (tx, rx) = mpsc::channel(1);

                            // Spawn async task to execute tool
                            tokio::spawn(async move {
                                let result = match tool.execute(params).await {
                                    Ok(output) => ExecutionResult::Success(output),
                                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                                };
                                let _ = tx.send(result).await;
                            });

                            // Transition to executing state (show as notification)
                            self.state = AppState::Executing {
                                tool_name: tool_name.to_string(),
                                result_rx: Some(rx),
                                show_as_notification: true,
                            };
                        }
                    }
                    return Ok(());
                }

                // Remaining indices are categories (adjust by -1)
                let category = Category::from_index(*selected_index - 1);

                // Auto-execute capture_list when entering Captures category
                if matches!(category, Category::CaptureManagement) {
                    // Find capture_list tool
                    if let Some(tool) = self.registry.get("capture_list") {
                        if let Ok(form) = ToolForm::new(tool.clone()) {
                            let tool_name = "capture_list".to_string();
                            let params = form.build_params();

                            // Create channel for results
                            let (tx, rx) = mpsc::channel(1);

                            // Spawn async task to execute tool
                            tokio::spawn(async move {
                                let result = match tool.execute(params).await {
                                    Ok(output) => ExecutionResult::Success(output),
                                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                                };
                                let _ = tx.send(result).await;
                            });

                            // Transition directly to executing state
                            self.state = AppState::Executing {
                                tool_name,
                                result_rx: Some(rx),
                                show_as_notification: false,
                            };
                            return Ok(());
                        }
                    }
                }

                self.state = AppState::ToolCategory {
                    category,
                    selected_index: 0,
                };
            }
            AppState::ToolCategory { category, selected_index } => {
                // Get selected tool (use filtered list for DeviceManagement)
                let device_connected = if matches!(category, Category::DeviceManagement) {
                    Some(self.device_status.connected)
                } else {
                    None
                };
                let tools = category.get_tools_filtered(&self.registry, device_connected);
                if let Some(tool) = tools.get(*selected_index) {
                    // Create form for this tool
                    match ToolForm::new(tool.clone()) {
                        Ok(form) => {
                            // Auto-execute if no params or all params are optional
                            if form.fields().is_empty() || form.all_fields_optional() {
                                let tool_name = form.tool_name().to_string();
                                let tool_clone = form.get_tool();

                                // Use default params
                                let params = form.build_params();

                                // Create channel for results
                                let (tx, rx) = mpsc::channel(1);

                                // Spawn async task to execute tool
                                tokio::spawn(async move {
                                    let result = match tool_clone.execute(params).await {
                                        Ok(output) => ExecutionResult::Success(output),
                                        Err(e) => ExecutionResult::Error(format!("{}", e)),
                                    };
                                    let _ = tx.send(result).await;
                                });

                                // Transition directly to executing state
                                self.state = AppState::Executing {
                                    tool_name,
                                    result_rx: Some(rx),
                                    show_as_notification: false,
                                };
                            } else {
                                // Show form for tools with required parameters
                                self.state = AppState::ToolForm {
                                    form: Box::new(form),
                                    error: None,
                                    hotkey_mode: false,  // Allow typing in form fields by default
                                };
                            }
                        }
                        Err(e) => {
                            // Show error in form state
                            self.state = AppState::ToolForm {
                                form: Box::new(ToolForm::new(tool.clone()).unwrap()),
                                error: Some(format!("Failed to create form: {}", e)),
                                hotkey_mode: false,  // Allow typing in form fields by default
                            };
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Go back to previous screen
    fn go_back(&mut self) {
        match &self.state {
            AppState::MainMenu { .. } => {
                // Already at main menu, quit
                self.should_quit = true;
            }
            AppState::ToolCategory { .. } | AppState::Settings { .. } => {
                // Go back to main menu
                self.state = AppState::MainMenu { selected_index: 0 };
            }
            AppState::ToolForm { .. } | AppState::Results { .. } => {
                // Go back to previous category (for now, just go to main menu)
                self.state = AppState::MainMenu { selected_index: 0 };
            }
            _ => {}
        }
    }

    /// Execute the tool with current form parameters
    fn execute_tool(&mut self) -> Result<()> {
        let (tool_name, rx) = if let AppState::ToolForm { form, error, .. } = &mut self.state {
            // Validate form
            if let Err(e) = form.validate() {
                *error = Some(e);
                return Ok(());
            }

            // Create channel for results
            let (tx, rx) = mpsc::channel(1);

            // Get tool name before moving form
            let tool_name = form.tool_name().to_string();

            // Get parameters and tool for execution
            let params = form.build_params();
            let tool = form.get_tool();

            // Spawn async task to execute tool
            let params_for_spawn = params.clone();
            let tool_for_spawn = tool.clone();
            tokio::spawn(async move {
                let result = match tool_for_spawn.execute(params_for_spawn).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            (tool_name, rx)
        } else {
            return Ok(());
        };

        // Add to history (after releasing the borrow)
        self.add_to_history(tool_name.clone());

        // Transition to executing state
        self.state = AppState::Executing {
            tool_name,
            result_rx: Some(rx),
            show_as_notification: false,
        };

        Ok(())
    }

    /// Launch analysis tool for a specific capture
    fn launch_analysis(&mut self, capture_id: String) -> Result<()> {
        // Find bt_analyze tool
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "bt_analyze")
            .cloned();

        if let Some(tool) = tool {
            // Create channel for results
            let (tx, rx) = mpsc::channel(1);

            // Build parameters with the capture_id
            let params = serde_json::json!({
                "capture_id": capture_id,
                "analysis_type": "auto"
            });

            // Spawn async task to execute analysis
            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            // Transition to executing state
            self.state = AppState::Executing {
                tool_name: "bt_analyze".to_string(),
                result_rx: Some(rx),
                show_as_notification: false,
            };
        } else {
            // bt_analyze not found
            self.state = AppState::Results {
                tool_name: "Error".to_string(),
                output: serde_json::json!({ "error": "bt_analyze tool not found" }),
                success: false,
                selected_capture: None,
                tool: None,
                    packet_list_state: None,
                    analysis_view_state: None,
            };
        }

        Ok(())
    }

    /// Launch capture_get to view details
    fn launch_capture_get(&mut self, capture_id: String) -> Result<()> {
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "capture_get")
            .cloned();

        if let Some(tool) = tool {
            let (tx, rx) = mpsc::channel(1);
            let params = serde_json::json!({ "capture_id": capture_id });

            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            self.state = AppState::Executing {
                tool_name: "capture_get".to_string(),
                result_rx: Some(rx),
                show_as_notification: false,
            };
        }

        Ok(())
    }

    /// Launch capture_delete with confirmation
    fn launch_capture_delete(&mut self, capture_id: String) -> Result<()> {
        // Show confirmation dialog
        self.state = AppState::Confirmation {
            message: format!("Delete capture {}? This cannot be undone.", capture_id),
            on_confirm: ConfirmAction::DeleteCapture(capture_id),
        };

        Ok(())
    }

    /// Execute confirmed delete action
    /// Handle dialog submission
    fn handle_dialog_submit(&mut self, dialog: TextInputDialog) -> Result<()> {
        let lines = dialog.textarea.lines();
        let text = lines.join("\n").trim().to_string();

        match dialog.context {
            DialogContext::Annotation { packet_index } => {
                // Update annotation in packet list state
                if let AppState::Results { packet_list_state, .. } = &mut self.state {
                    if let Some(pls) = packet_list_state {
                        if text.is_empty() {
                            // Remove annotation if text is empty
                            pls.remove_annotation(packet_index);
                        } else {
                            // Add or update annotation
                            pls.add_annotation(packet_index, text);
                        }
                    }
                }
            }
            DialogContext::Search => {
                // TODO: Implement search functionality in Task #49
                self.notification = Some(Notification {
                    message: "Search not yet implemented".to_string(),
                    success: false,
                });
            }
            DialogContext::Filter { field: _ } => {
                // TODO: Implement filter in Task #47
                self.notification = Some(Notification {
                    message: "Filter not yet implemented".to_string(),
                    success: false,
                });
            }
        }

        Ok(())
    }

    fn execute_delete_capture(&mut self, capture_id: String) -> Result<()> {
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "capture_delete")
            .cloned();

        if let Some(tool) = tool {
            let (tx, rx) = mpsc::channel(1);
            let params = serde_json::json!({ "capture_id": capture_id.clone() });

            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            // Use notification for delete results
            self.state = AppState::Executing {
                tool_name: "capture_delete".to_string(),
                result_rx: Some(rx),
                show_as_notification: true,
            };
        }

        Ok(())
    }

    /// Launch capture_export
    fn launch_capture_export(&mut self, capture_id: String) -> Result<()> {
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "capture_export")
            .cloned();

        if let Some(tool) = tool {
            let (tx, rx) = mpsc::channel(1);
            // Use defaults: json format, default path
            let params = serde_json::json!({
                "capture_id": capture_id,
                "format": "json"
            });

            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            self.state = AppState::Executing {
                tool_name: "capture_export".to_string(),
                result_rx: Some(rx),
                show_as_notification: false,
            };
        }

        Ok(())
    }

    /// Export packets based on selected option
    fn export_packets(&self, option: ExportOption, packets: Vec<serde_json::Value>, state: PacketListState) -> Result<String> {
        use std::fs;
        use std::path::PathBuf;

        // Create export directory
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let export_dir = PathBuf::from(home).join(".ubertooth").join("exports");
        fs::create_dir_all(&export_dir)?;

        // Generate filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        match option {
            ExportOption::BookmarkedPackets => {
                let bookmarked: Vec<_> = packets.iter()
                    .enumerate()
                    .filter(|(idx, _)| state.is_bookmarked(*idx))
                    .map(|(_, pkt)| pkt.clone())
                    .collect();

                let filename = format!("bookmarked_packets_{}.json", timestamp);
                let path = export_dir.join(&filename);
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "exported_at": chrono::Utc::now().to_rfc3339(),
                    "export_type": "bookmarked_packets",
                    "packet_count": bookmarked.len(),
                    "packets": bookmarked,
                }))?;
                fs::write(&path, json)?;
                Ok(path.to_string_lossy().to_string())
            }

            ExportOption::FilteredPackets => {
                let filtered: Vec<_> = packets.iter()
                    .filter(|pkt| {
                        // Apply filters from state
                        if let Some(ref follow_mac) = state.follow_mac {
                            if let Some(mac) = pkt.get("mac_address").and_then(|m| m.as_str()) {
                                return mac == follow_mac;
                            }
                            return false;
                        }
                        true
                    })
                    .cloned()
                    .collect();

                let filename = format!("filtered_packets_{}.json", timestamp);
                let path = export_dir.join(&filename);
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "exported_at": chrono::Utc::now().to_rfc3339(),
                    "export_type": "filtered_packets",
                    "filter": state.follow_mac.clone().unwrap_or_else(|| "none".to_string()),
                    "packet_count": filtered.len(),
                    "packets": filtered,
                }))?;
                fs::write(&path, json)?;
                Ok(path.to_string_lossy().to_string())
            }

            ExportOption::Statistics => {
                // Calculate statistics
                let mut packet_types = std::collections::HashMap::new();
                let mut channels = std::collections::HashMap::new();
                let mut rssi_values = Vec::new();
                let mut mac_addresses = std::collections::HashSet::new();

                for packet in &packets {
                    if let Some(pkt_type) = packet.get("packet_type").and_then(|t| t.as_str()) {
                        *packet_types.entry(pkt_type.to_string()).or_insert(0) += 1;
                    }
                    if let Some(channel) = packet.get("channel").and_then(|c| c.as_str()) {
                        *channels.entry(channel.to_string()).or_insert(0) += 1;
                    }
                    if let Some(rssi) = packet.get("rssi").and_then(|r| r.as_str()).and_then(|s| s.parse::<i32>().ok()) {
                        rssi_values.push(rssi);
                    }
                    if let Some(mac) = packet.get("mac_address").and_then(|m| m.as_str()) {
                        mac_addresses.insert(mac.to_string());
                    }
                }

                let (rssi_min, rssi_max, rssi_avg) = if !rssi_values.is_empty() {
                    let min = *rssi_values.iter().min().unwrap();
                    let max = *rssi_values.iter().max().unwrap();
                    let avg = rssi_values.iter().sum::<i32>() as f64 / rssi_values.len() as f64;
                    (min, max, avg)
                } else {
                    (0, 0, 0.0)
                };

                let filename = format!("statistics_{}.json", timestamp);
                let path = export_dir.join(&filename);
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "exported_at": chrono::Utc::now().to_rfc3339(),
                    "export_type": "statistics",
                    "total_packets": packets.len(),
                    "packet_types": packet_types,
                    "channels": channels,
                    "rssi": {
                        "min": rssi_min,
                        "max": rssi_max,
                        "avg": rssi_avg,
                    },
                    "unique_mac_addresses": mac_addresses.len(),
                    "mac_addresses": mac_addresses.into_iter().collect::<Vec<_>>(),
                }))?;
                fs::write(&path, json)?;
                Ok(path.to_string_lossy().to_string())
            }

            ExportOption::ComparisonReport => {
                if state.comparison_marks.len() != 2 {
                    return Err(anyhow::anyhow!("Exactly 2 packets must be marked for comparison"));
                }

                let idx1 = state.comparison_marks[0];
                let idx2 = state.comparison_marks[1];
                let pkt1 = packets.get(idx1).ok_or_else(|| anyhow::anyhow!("Packet 1 not found"))?;
                let pkt2 = packets.get(idx2).ok_or_else(|| anyhow::anyhow!("Packet 2 not found"))?;

                let filename = format!("comparison_report_{}.md", timestamp);
                let path = export_dir.join(&filename);

                let get_field = |pkt: &serde_json::Value, field: &str| {
                    pkt.get(field).and_then(|v| v.as_str()).unwrap_or("N/A").to_string()
                };

                let mut report = String::new();
                report.push_str("# Packet Comparison Report\n\n");
                report.push_str(&format!("**Generated**: {}\n\n", chrono::Utc::now().to_rfc3339()));
                report.push_str("## Packet A\n\n");
                report.push_str(&format!("- **Index**: {}\n", idx1));
                report.push_str(&format!("- **Frame**: {}\n", get_field(pkt1, "frame_number")));
                report.push_str(&format!("- **Timestamp**: {}\n", get_field(pkt1, "timestamp")));
                report.push_str(&format!("- **Channel**: {}\n", get_field(pkt1, "channel")));
                report.push_str(&format!("- **RSSI**: {} dBm\n", get_field(pkt1, "rssi")));
                report.push_str(&format!("- **Type**: {}\n", get_field(pkt1, "packet_type")));
                report.push_str(&format!("- **MAC**: {}\n", get_field(pkt1, "mac_address")));
                report.push_str(&format!("- **Protocol**: {}\n", get_field(pkt1, "protocol")));
                report.push_str(&format!("- **Access Address**: {}\n", get_field(pkt1, "access_addr")));
                report.push_str(&format!("- **Summary**: {}\n\n", get_field(pkt1, "summary")));

                if let Some(note) = state.get_annotation(idx1) {
                    report.push_str(&format!("**Note**: {}\n\n", note));
                }

                report.push_str("## Packet B\n\n");
                report.push_str(&format!("- **Index**: {}\n", idx2));
                report.push_str(&format!("- **Frame**: {}\n", get_field(pkt2, "frame_number")));
                report.push_str(&format!("- **Timestamp**: {}\n", get_field(pkt2, "timestamp")));
                report.push_str(&format!("- **Channel**: {}\n", get_field(pkt2, "channel")));
                report.push_str(&format!("- **RSSI**: {} dBm\n", get_field(pkt2, "rssi")));
                report.push_str(&format!("- **Type**: {}\n", get_field(pkt2, "packet_type")));
                report.push_str(&format!("- **MAC**: {}\n", get_field(pkt2, "mac_address")));
                report.push_str(&format!("- **Protocol**: {}\n", get_field(pkt2, "protocol")));
                report.push_str(&format!("- **Access Address**: {}\n", get_field(pkt2, "access_addr")));
                report.push_str(&format!("- **Summary**: {}\n\n", get_field(pkt2, "summary")));

                if let Some(note) = state.get_annotation(idx2) {
                    report.push_str(&format!("**Note**: {}\n\n", note));
                }

                report.push_str("## Differences\n\n");
                let fields = vec![
                    ("Channel", "channel"),
                    ("RSSI", "rssi"),
                    ("Type", "packet_type"),
                    ("MAC Address", "mac_address"),
                    ("Protocol", "protocol"),
                    ("Access Address", "access_addr"),
                ];

                for (label, field) in fields {
                    let val1 = get_field(pkt1, field);
                    let val2 = get_field(pkt2, field);
                    if val1 != val2 {
                        report.push_str(&format!("- **{}**: `{}` → `{}`\n", label, val1, val2));
                    }
                }

                fs::write(&path, report)?;
                Ok(path.to_string_lossy().to_string())
            }

            ExportOption::TimelineData => {
                let filename = format!("timeline_{}.csv", timestamp);
                let path = export_dir.join(&filename);

                let mut csv = String::new();
                csv.push_str("index,frame_number,timestamp,channel,rssi,packet_type,mac_address\n");

                for (idx, packet) in packets.iter().enumerate() {
                    let get_field = |field: &str| {
                        packet.get(field).and_then(|v| v.as_str()).unwrap_or("").to_string()
                    };

                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        idx,
                        get_field("frame_number"),
                        get_field("timestamp"),
                        get_field("channel"),
                        get_field("rssi"),
                        get_field("packet_type"),
                        get_field("mac_address")
                    ));
                }

                fs::write(&path, csv)?;
                Ok(path.to_string_lossy().to_string())
            }

            ExportOption::AllPackets => {
                let filename = format!("all_packets_{}.json", timestamp);
                let path = export_dir.join(&filename);
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "exported_at": chrono::Utc::now().to_rfc3339(),
                    "export_type": "all_packets",
                    "packet_count": packets.len(),
                    "packets": packets,
                }))?;
                fs::write(&path, json)?;
                Ok(path.to_string_lossy().to_string())
            }
        }
    }

    /// Launch capture_tag
    fn launch_capture_tag(&mut self, capture_id: String) -> Result<()> {
        // TODO: Add tag input dialog
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "capture_tag")
            .cloned();

        if let Some(tool) = tool {
            let (tx, rx) = mpsc::channel(1);
            // For now, use empty tags (will need tag input UI)
            let params = serde_json::json!({
                "capture_id": capture_id,
                "tags": ["manual-tag"]
            });

            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            self.state = AppState::Executing {
                tool_name: "capture_tag".to_string(),
                result_rx: Some(rx),
                show_as_notification: false,
            };
        }

        Ok(())
    }

    /// Add tool to execution history (keeps last 10)
    fn add_to_history(&mut self, tool_name: String) {
        // Remove if already in history
        self.tool_history.retain(|t| t != &tool_name);

        // Add to front
        self.tool_history.insert(0, tool_name);

        // Keep only last 10
        self.tool_history.truncate(10);
    }

    /// Toggle tool in favorites
    pub fn toggle_favorite(&mut self, tool_name: String) {
        if self.favorites.contains(&tool_name) {
            self.favorites.retain(|t| t != &tool_name);
        } else {
            self.favorites.push(tool_name);
        }
    }

    /// Check if tool is favorited
    pub fn is_favorite(&self, tool_name: &str) -> bool {
        self.favorites.contains(&tool_name.to_string())
    }

    /// Extract MAC addresses from analysis output and add to recent list
    fn extract_macs_from_output(&mut self, output: &serde_json::Value) {
        // Extract from devices array
        if let Some(devices) = output.get("devices").and_then(|d| d.as_array()) {
            for device in devices {
                if let Some(mac) = device.get("mac_address").and_then(|m| m.as_str()) {
                    self.add_recent_mac(mac.to_string());
                } else if let Some(mac) = device.get("address").and_then(|m| m.as_str()) {
                    self.add_recent_mac(mac.to_string());
                }
            }
        }

        // Extract from summary
        if let Some(mac) = output.get("target_mac").and_then(|m| m.as_str()) {
            self.add_recent_mac(mac.to_string());
        }
    }

    /// Add MAC to recent list (keeps last 20 unique MACs)
    fn add_recent_mac(&mut self, mac: String) {
        // Remove if already exists
        self.recent_macs.retain(|m| m != &mac);

        // Add to front
        self.recent_macs.insert(0, mac);

        // Keep only last 20
        self.recent_macs.truncate(20);
    }

    /// Get recent MAC addresses for filtering
    pub fn get_recent_macs(&self) -> Vec<String> {
        self.recent_macs.clone()
    }
}
