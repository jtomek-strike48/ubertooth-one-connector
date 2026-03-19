//! Session management view rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::Arc;

use super::utils::centered_rect;
use crate::tui::app::{AppState, SessionMode};


/// Render session manager view
pub(crate) fn render_session_manager(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
    sessions: &[ubertooth_platform::SessionMetadata],
    mode: &crate::tui::app::SessionMode,
    session_name: &str,
) {
    use crate::tui::app::SessionMode;

    match mode {
        SessionMode::List => {
            render_session_list(f, area, selected_index, sessions);
        }
        SessionMode::Save => {
            render_session_save(f, area, session_name);
        }
        SessionMode::Load => {
            render_session_loading(f, area);
        }
    }
}


/// Render session list
pub(crate) fn render_session_list(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
    sessions: &[ubertooth_platform::SessionMetadata],
) {
    if sessions.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No saved sessions yet",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 's' to save current session",
                Style::default().fg(Color::Yellow),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Session Manager "),
        )
        .alignment(Alignment::Center);

        f.render_widget(empty, area);
        return;
    }

    // Build session list items
    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let is_selected = i == selected_index;

            let tool_str = session
                .current_tool
                .as_ref()
                .map(|t| t.as_str())
                .unwrap_or("None");
            let updated = session.updated_at.format("%Y-%m-%d %H:%M:%S");

            let lines = vec![
                Line::from(vec![Span::styled(
                    format!("  {}", session.name),
                    Style::default()
                        .fg(if is_selected {
                            Color::Yellow
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::styled("    Tool: ", Style::default().fg(Color::Gray)),
                    Span::styled(tool_str, Style::default().fg(Color::Cyan)),
                    Span::raw("  "),
                    Span::styled(
                        format!("{} capture(s)", session.captures_count),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    Updated: ", Style::default().fg(Color::Gray)),
                    Span::styled(updated.to_string(), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(""), // Blank line between items
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Session Manager ({} sessions) ", sessions.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_widget(list, area);
}


/// Render session save dialog
pub(crate) fn render_session_save(f: &mut Frame, area: Rect, session_name: &str) {
    // Create centered dialog
    let dialog_area = centered_rect(60, 30, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Save Current Session",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Session Name:",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", session_name),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Save  [Esc] Cancel",
            Style::default().fg(Color::Gray),
        )),
    ];

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Save Session "),
        )
        .alignment(Alignment::Center);

    // Clear background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    f.render_widget(dialog, dialog_area);
}


/// Render session loading message
pub(crate) fn render_session_loading(f: &mut Frame, area: Rect) {
    let text = Paragraph::new("Loading session...")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Session Manager "),
        );

    f.render_widget(text, area);
}

