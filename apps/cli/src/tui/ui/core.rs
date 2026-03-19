//! Core UI rendering logic

use super::analysis::{
    render_analysis_devices, render_analysis_overview, render_analysis_results,
    render_analysis_security, render_analysis_timing,
};
use super::capture::{render_capture_details, render_capture_list_table};
use super::errors::{render_comparison_results, render_error_message};
use super::forms::{render_export_menu, render_filter_dialog, render_tool_form};
use super::live::{
    render_live_capture, render_live_capture_header, render_live_packet_list,
    render_live_statistics,
};
use super::menu::{render_main_menu, render_tool_category, render_tool_hotkeys};
use super::overlays::{
    render_confirmation, render_dialog, render_help_overlay, render_notification,
    render_theme_selector,
};
use super::packets::comparison::render_packet_comparison;
use super::packets::decoded::render_decoded_packets;
use super::packets::list::render_packet_list;
use super::packets::statistics::render_packet_statistics;
use super::packets::timeline::render_packet_timeline;
use super::session::{
    render_session_list, render_session_loading, render_session_manager, render_session_save,
};
use super::utils::{categorize_error, centered_rect};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::sync::Arc;
use ubertooth_core::ToolRegistry;

use crate::tui::app::{AppState, DeviceStatus, Notification, TextInputDialog};
use crate::tui::themes::Theme;
use crate::tui::views::{Category, FieldInputMode, FieldType};

/// Render the entire UI
pub fn render(
    f: &mut Frame,
    state: &AppState,
    registry: &Arc<ToolRegistry>,
    device_status: &DeviceStatus,
    notification: &Option<Notification>,
    frame_count: u64,
    dialog: &Option<TextInputDialog>,
    theme: &Theme,
) {
    // Main layout: header + content + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0], device_status, theme);
    render_content(
        f,
        chunks[1],
        state,
        registry,
        device_status,
        frame_count,
        theme,
    );
    render_footer(f, chunks[2], state);

    // Render notification on top if present
    if let Some(notif) = notification {
        render_notification(f, f.size(), notif);
    }

    // Render dialog overlay on top if present
    if let Some(dlg) = dialog {
        render_dialog(f, f.size(), dlg);
    }
}

/// Render header with device status
fn render_header(f: &mut Frame, area: Rect, device_status: &DeviceStatus, theme: &Theme) {
    // Build status string
    let device_str = if device_status.connected {
        if let Some(fw) = &device_status.firmware {
            format!("Device: Connected ({})", fw)
        } else {
            "Device: Connected".to_string()
        }
    } else {
        "Device: Not Connected".to_string()
    };

    let backend_str = "Backend: Python";
    let strike48_str = "Strike48: Not Connected";

    let title = format!("{} | {} | {}", device_str, backend_str, strike48_str);

    let header = Paragraph::new("Ubertooth CLI")
        .style(
            Style::default()
                .fg(theme.colors.title.to_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(header, area);
}

/// Render main content based on state
fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    registry: &Arc<ToolRegistry>,
    device_status: &DeviceStatus,
    frame_count: u64,
    theme: &Theme,
) {
    match state {
        AppState::MainMenu { selected_index } => {
            render_main_menu(f, area, *selected_index, device_status);
        }
        AppState::ToolCategory {
            category,
            selected_index,
        } => {
            render_tool_category(f, area, category, *selected_index, registry, device_status);
        }
        AppState::ToolForm {
            form,
            error,
            hotkey_mode,
        } => {
            if *hotkey_mode {
                render_tool_hotkeys(f, area, form.as_ref(), error.as_deref());
            } else {
                render_tool_form(f, area, form.as_ref(), error.as_deref());
            }
        }
        AppState::Executing { tool_name, .. } => {
            render_executing(f, area, tool_name, frame_count);
        }
        AppState::Results {
            tool_name,
            output,
            success,
            selected_capture,
            packet_list_state,
            analysis_view_state,
            ..
        } => {
            render_results(
                f,
                area,
                tool_name,
                output,
                *success,
                *selected_capture,
                packet_list_state.as_ref(),
                analysis_view_state.as_ref(),
            );
        }
        AppState::Settings { selected_index } => {
            render_settings(f, area, *selected_index);
        }
        AppState::Confirmation { message, .. } => {
            render_confirmation(f, area, message);
        }
        AppState::ExportMenu {
            selected_index,
            packets,
            packet_list_state,
            ..
        } => {
            render_export_menu(f, area, *selected_index, packets.len(), packet_list_state);
        }
        AppState::FilterDialog {
            selected_section,
            selected_packet_type,
            packet_type_selections,
            mac_filter,
            rssi_min,
            rssi_max,
            ..
        } => {
            render_filter_dialog(
                f,
                area,
                *selected_section,
                *selected_packet_type,
                packet_type_selections,
                mac_filter,
                rssi_min,
                rssi_max,
            );
        }
        AppState::HelpOverlay { scroll_offset, .. } => {
            render_help_overlay(f, area, *scroll_offset, theme);
        }
        AppState::ThemeSelector {
            selected_index,
            themes,
        } => {
            render_theme_selector(f, area, *selected_index, themes, theme);
        }
        AppState::LiveCapture {
            buffer,
            stats,
            paused,
            limits,
            selected_index,
            scroll_offset,
            tool_name,
        } => {
            render_live_capture(
                f,
                area,
                buffer,
                stats,
                *paused,
                limits,
                *selected_index,
                *scroll_offset,
                tool_name,
            );
        }
        AppState::SessionManager {
            selected_index,
            sessions,
            mode,
            session_name,
        } => {
            render_session_manager(f, area, *selected_index, sessions, mode, session_name);
        }
    }
}









/// Render tool execution progress
fn render_executing(f: &mut Frame, area: Rect, tool_name: &str, frame_count: u64) {
    // Animated spinner frames
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_frames[(frame_count / 2) as usize % spinner_frames.len()];

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(spinner, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("Executing: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                tool_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Please wait...",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This may take a few seconds depending on the tool.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Executing"));

    f.render_widget(paragraph, area);
}

/// Render tool results
fn render_results(
    f: &mut Frame,
    area: Rect,
    tool_name: &str,
    output: &serde_json::Value,
    success: bool,
    selected_capture: Option<usize>,
    packet_list_state: Option<&crate::tui::app::PacketListState>,
    analysis_view_state: Option<&crate::tui::app::AnalysisViewState>,
) {
    // Split into header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header with status
    let status_symbol = if success { "[OK]" } else { "[FAILED]" };
    let status_text = if success { "Success" } else { "Failed" };
    let status_color = if success { Color::Green } else { Color::Red };

    let header_text = format!("{} {}\n\nTool: {}", status_symbol, status_text, tool_name);
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Execution Result"),
        );
    f.render_widget(header, chunks[0]);

    // Special formatting for capture_list - show as table
    if tool_name == "capture_list" && success {
        if let Some(captures_array) = output.get("captures").and_then(|v| v.as_array()) {
            render_capture_list_table(f, chunks[1], captures_array, selected_capture);
            return;
        }
    }

    // Special formatting for bt_compare - show comparison view
    if tool_name == "bt_compare" && success {
        render_comparison_results(f, chunks[1], output);
        return;
    }

    // Special formatting for bt_analyze - show analysis results
    if tool_name == "bt_analyze" && success {
        render_analysis_results(f, chunks[1], output, analysis_view_state);
        return;
    }

    // Special formatting for bt_decode - show packet list
    if tool_name == "bt_decode" && success {
        render_decoded_packets(f, chunks[1], output, packet_list_state);
        return;
    }

    // Special formatting for capture_get - show capture details
    if tool_name == "capture_get" && success {
        render_capture_details(f, chunks[1], output);
        return;
    }

    // Special formatting for errors - show clean error message
    if !success {
        if let Some(error_msg) = output.get("error").and_then(|v| v.as_str()) {
            render_error_message(f, chunks[1], tool_name, error_msg);
            return;
        }
    }

    // Content - format JSON nicely
    let result_json = serde_json::to_string_pretty(output).unwrap_or_else(|_| "{}".to_string());

    // Highlight specific fields for better readability
    let formatted_output = if let Some(obj) = output.as_object() {
        let mut lines = Vec::new();

        // Show important fields first
        if let Some(capture_id) = obj.get("capture_id").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled("Capture ID: ", Style::default().fg(Color::Cyan)),
                Span::styled(capture_id, Style::default().fg(Color::White)),
            ]));
        }

        if let Some(packets) = obj.get("packets_captured").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::styled("Packets: ", Style::default().fg(Color::Cyan)),
                Span::styled(packets.to_string(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(devices) = obj.get("devices_found").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::styled("Devices: ", Style::default().fg(Color::Cyan)),
                Span::styled(devices.to_string(), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(duration) = obj.get("duration").and_then(|v| v.as_f64()) {
            lines.push(Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:.1}s", duration),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Full Output:",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        // Add full JSON
        for line in result_json.lines() {
            lines.push(Line::from(line.to_string()));
        }

        Text::from(lines)
    } else {
        Text::from(result_json)
    };

    let content = Paragraph::new(formatted_output).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Output")
            .title_style(Style::default().fg(Color::Gray)),
    );

    f.render_widget(content, chunks[1]);
}

/// Render settings page
fn render_settings(f: &mut Frame, area: Rect, selected_index: usize) {
    let settings_items = vec![
        ("View Tool History", "Show recently used tools"),
        ("View Favorites", "Show bookmarked tools"),
        (
            "View Recent MAC Addresses",
            "MAC filter helper for analysis",
        ),
        ("Backend Info", "View backend configuration"),
        ("Strike48 Connection", "Configure cloud connection"),
        ("About", "Version and system information"),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, (title, desc))| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(Span::styled(format!("{}. {}", i + 1, title), style)),
                Line::from(Span::styled(
                    format!("   {}", desc),
                    Style::default().fg(Color::Gray),
                )),
                Line::from(""),
            ];

            ListItem::new(Text::from(content))
        })
        .collect();

    let settings_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(settings_list, area);
}

/// Render footer with keyboard shortcuts
fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let shortcuts = match state {
        AppState::MainMenu { .. } => {
            "[0-6] Quick Select  [↑/↓] Navigate  [Enter] Select  [?] Help  [q] Quit"
        }
        AppState::ToolCategory { category, .. } => {
            if matches!(category, Category::DeviceManagement) {
                "[1-9] Quick Select  [→] Device Status  [Enter] Select  [?] Help  [Esc] Back"
            } else {
                "[1-9] Quick Select  [↑/↓] Navigate  [Enter] Select  [?] Help  [Esc] Back"
            }
        }
        AppState::ToolForm { hotkey_mode, .. } => {
            if *hotkey_mode {
                "[1-9] Set Value  [Enter] Execute  [?] Help  [Esc] Back"
            } else {
                "[Tab] Next  [Enter] Execute  [?] Help  [Esc] Cancel"
            }
        }
        AppState::Executing { .. } => {
            "Executing... please wait"
        }
        AppState::Results { tool_name, .. } => {
            match tool_name.as_str() {
                "capture_list" => "[↑/↓] Navigate  [Enter] Analyze  [V] View  [D] Delete  [E] Export  [T] Tag  [?] Help",
                "bt_decode" => "[↑/↓] Navigate  [Enter] Expand  [b] Bookmark  [m] Mark  [/] Filter  [e] Export  [?] Help",
                "bt_analyze" => "[o/d/s/t] View Modes  [↑/↓] Navigate  [Enter] Expand  [?] Help  [Esc] Back",
                "bt_compare" => "[?] Help  [Esc] Back to Menu - Side-by-side capture comparison",
                _ => "[?] Help  [Esc] Back to Menu"
            }
        }
        AppState::Settings { .. } => {
            "[?] Help  [Esc] Back to Menu"
        }
        AppState::Confirmation { .. } => {
            "[Y] Confirm  [N] Cancel"
        }
        AppState::ExportMenu { .. } => {
            "[↑/↓] Navigate  [Enter] Export  [?] Help  [Esc] Cancel"
        }
        AppState::FilterDialog { .. } => {
            "[↑/↓] Navigate  [←/→] Select  [Space] Toggle  [Enter] Apply  [C] Clear  [?] Help  [Esc] Cancel"
        }
        AppState::HelpOverlay { .. } => {
            "[↑/↓/PgUp/PgDn] Scroll  [Esc/?/q] Close Help"
        }
        AppState::ThemeSelector { .. } => {
            "[↑/↓] Navigate  [Enter] Apply Theme  [1-5] Quick Select  [Esc] Cancel"
        }
        AppState::LiveCapture { paused, .. } => {
            if *paused {
                "[Space] Resume  [s] Save  [c] Clear  [Esc] Stop  [?] Help"
            } else {
                "[Space] Pause  [s] Save  [c] Clear  [Esc] Stop  [?] Help"
            }
        }
        AppState::SessionManager { mode, .. } => {
            use crate::tui::app::SessionMode;
            match mode {
                SessionMode::List => "[↑/↓] Navigate  [Enter] Load  [s] Save  [d] Delete  [Esc] Back",
                SessionMode::Save => "[Type name]  [Enter] Save  [Esc] Cancel",
                SessionMode::Load => "Loading session...",
            }
        }
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}


