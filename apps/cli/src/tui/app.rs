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
    },

    /// Display results
    Results {
        tool_name: String,
        output: serde_json::Value,
        success: bool,
        selected_capture: Option<usize>,
        tool: Option<Arc<dyn PentestTool>>,
    },

    /// Settings page
    Settings {},
}

/// Device connection status
#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub connected: bool,
    pub firmware: Option<String>,
}

/// Main TUI application
pub struct App {
    /// Current application state
    state: AppState,

    /// Tool registry
    registry: Arc<ToolRegistry>,

    /// Device connection status
    device_status: DeviceStatus,

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
                // Render UI (catch and log any render errors)
                if let Err(e) = terminal.draw(|f| ui::render(f, &self.state, &self.registry, &self.device_status)) {
                    tracing::error!("Render error: {}", e);
                    // Continue anyway - might be transient
                }

                // Check for tool execution results
                if let AppState::Executing { tool_name, result_rx } = &mut self.state {
                    if let Some(rx) = result_rx {
                        match rx.try_recv() {
                            Ok(result) => {
                                // Move out of executing state
                                let tool_name = tool_name.clone();
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

                                        // Check if this is capture_list with results
                                        let selected_capture = if tool_name == "capture_list" {
                                            output.get("captures")
                                                .and_then(|c| c.as_array())
                                                .filter(|arr| !arr.is_empty())
                                                .map(|_| 0) // Select first capture
                                        } else {
                                            None
                                        };

                                        self.state = AppState::Results {
                                            tool_name,
                                            output,
                                            success: true,
                                            selected_capture,
                                            tool: None, // TODO: Store tool for re-parameterization
                                        };
                                    }
                                    ExecutionResult::Error(error) => {
                                        self.state = AppState::Results {
                                            tool_name,
                                            output: serde_json::json!({ "error": error }),
                                            success: false,
                                            selected_capture: None,
                                            tool: None,
                                        };
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

        // Normal navigation
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('s') => {
                    // Open settings
                    self.state = AppState::Settings {};
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
            _ => {}
        }
    }

    /// Handle Enter key on current selection
    fn handle_selection(&mut self) -> Result<()> {
        match &self.state {
            AppState::MainMenu { selected_index } => {
                // Navigate to tool category
                let category = Category::from_index(*selected_index);

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
                                };
                            } else {
                                // Show form for tools with required parameters
                                self.state = AppState::ToolForm {
                                    form: Box::new(form),
                                    error: None,
                                    hotkey_mode: true,
                                };
                            }
                        }
                        Err(e) => {
                            // Show error in form state
                            self.state = AppState::ToolForm {
                                form: Box::new(ToolForm::new(tool.clone()).unwrap()),
                                error: Some(format!("Failed to create form: {}", e)),
                                hotkey_mode: true,
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
        if let AppState::ToolForm { form, error, .. } = &mut self.state {
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
            tokio::spawn(async move {
                let result = match tool.execute(params).await {
                    Ok(output) => ExecutionResult::Success(output),
                    Err(e) => ExecutionResult::Error(format!("{}", e)),
                };
                let _ = tx.send(result).await;
            });

            // Transition to executing state
            self.state = AppState::Executing {
                tool_name,
                result_rx: Some(rx),
            };
        }

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
            };
        } else {
            // bt_analyze not found
            self.state = AppState::Results {
                tool_name: "Error".to_string(),
                output: serde_json::json!({ "error": "bt_analyze tool not found" }),
                success: false,
                selected_capture: None,
                tool: None,
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
            };
        }

        Ok(())
    }

    /// Launch capture_delete with confirmation
    fn launch_capture_delete(&mut self, capture_id: String) -> Result<()> {
        // TODO: Add confirmation dialog
        let tool = self.registry.tools()
            .iter()
            .find(|t| t.name() == "capture_delete")
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
                tool_name: "capture_delete".to_string(),
                result_rx: Some(rx),
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
            };
        }

        Ok(())
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
            };
        }

        Ok(())
    }
}
