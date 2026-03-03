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
use ubertooth_core::ToolRegistry;
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
#[derive(Debug)]
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
    },

    /// Settings page
    Settings {},
}

/// Main TUI application
pub struct App {
    /// Current application state
    state: AppState,

    /// Tool registry
    registry: Arc<ToolRegistry>,

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

        // Create event handler
        let mut events = EventHandler::new(250); // 250ms tick rate

        // Main loop
        loop {
            // Render UI
            terminal.draw(|f| ui::render(f, &self.state, &self.registry))?;

            // Check for tool execution results
            if let AppState::Executing { tool_name, result_rx } = &mut self.state {
                if let Some(rx) = result_rx {
                    if let Ok(result) = rx.try_recv() {
                        // Move out of executing state
                        let tool_name = tool_name.clone();
                        match result {
                            ExecutionResult::Success(output) => {
                                self.state = AppState::Results {
                                    tool_name,
                                    output,
                                    success: true,
                                };
                            }
                            ExecutionResult::Error(error) => {
                                self.state = AppState::Results {
                                    tool_name,
                                    output: serde_json::json!({ "error": error }),
                                    success: false,
                                };
                            }
                        }
                    }
                }
            }

            // Handle events
            if let Some(event) = events.next()? {
                self.handle_event(event)?;
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Handle an input event
    fn handle_event(&mut self, event: crossterm::event::Event) -> Result<()> {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        // Handle form input specially
        if let AppState::ToolForm { form, error } = &mut self.state {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
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
                    KeyCode::Enter => {
                        // Enter to submit form
                        self.execute_tool()?;
                        return Ok(());
                    }
                    _ => {
                        // Pass event to focused input
                        if let Some(input) = form.focused_input_mut() {
                            input.input(event);
                        }
                        // Clear error on input
                        *error = None;
                        return Ok(());
                    }
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
                    self.state = AppState::Settings {};
                }
                KeyCode::Up => {
                    self.move_selection(-1);
                }
                KeyCode::Down => {
                    self.move_selection(1);
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
                let tool_count = category.tool_count(&self.registry);
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
                self.state = AppState::ToolCategory {
                    category,
                    selected_index: 0,
                };
            }
            AppState::ToolCategory { category, selected_index } => {
                // Get selected tool
                let tools = category.get_tools(&self.registry);
                if let Some(tool) = tools.get(*selected_index) {
                    // Create form for this tool
                    match ToolForm::new(tool.clone()) {
                        Ok(form) => {
                            self.state = AppState::ToolForm {
                                form: Box::new(form),
                                error: None,
                            };
                        }
                        Err(e) => {
                            // Show error in form state
                            self.state = AppState::ToolForm {
                                form: Box::new(ToolForm::new(tool.clone()).unwrap()),
                                error: Some(format!("Failed to create form: {}", e)),
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
        if let AppState::ToolForm { form, error } = &mut self.state {
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
}
