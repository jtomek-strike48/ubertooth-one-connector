//! Overlay rendering functions (notifications, dialogs, help, theme selector)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::super::app::{Notification, TextInputDialog};
use super::super::themes::Theme;

/// Render notification overlay at bottom center
pub(crate) fn render_notification(f: &mut Frame, area: Rect, notification: &Notification) {
    // Calculate notification size and position
    let notif_width = notification.message.len().min(60) as u16 + 4;
    let notif_height = 3;

    // Position at bottom center
    let notif_x = area.width.saturating_sub(notif_width) / 2;
    let notif_y = area.height.saturating_sub(notif_height + 4); // Above footer

    let notif_area = Rect {
        x: area.x + notif_x,
        y: area.y + notif_y,
        width: notif_width,
        height: notif_height,
    };

    // Choose color based on success/failure
    let (bg_color, fg_color) = if notification.success {
        (Color::Green, Color::Black)
    } else {
        (Color::Red, Color::White)
    };

    let notif_widget = Paragraph::new(notification.message.as_str())
        .style(Style::default().fg(fg_color).bg(bg_color))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(notif_widget, notif_area);
}

/// Render confirmation dialog overlay
pub(crate) fn render_confirmation(f: &mut Frame, area: Rect, message: &str) {
    // Create centered dialog
    let dialog_width = message.len().max(40).min(80) as u16 + 4;
    let dialog_height = 7;

    let dialog_x = area.width.saturating_sub(dialog_width) / 2;
    let dialog_y = area.height.saturating_sub(dialog_height) / 2;

    let dialog_area = Rect {
        x: area.x + dialog_x,
        y: area.y + dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Build dialog content
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Y] to confirm or [N] to cancel",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    let dialog = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title("Confirmation")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(dialog, dialog_area);
}

/// Render text input dialog overlay
pub(crate) fn render_dialog(f: &mut Frame, area: Rect, dialog: &TextInputDialog) {
    // Create centered dialog area
    let dialog_width = area.width.saturating_sub(20).min(80);
    let dialog_height = 10;
    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background
    f.render_widget(Clear, dialog_area);

    // Split dialog into title, input, and help
    let dialog_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(4),    // Text input
            Constraint::Length(2), // Help text
        ])
        .split(dialog_area);

    // Render title
    let title_text = format!(" {} ", dialog.title);
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    f.render_widget(title, dialog_chunks[0]);

    // Render textarea widget
    let widget = dialog.textarea.widget();
    f.render_widget(widget, dialog_chunks[1]);

    // Render help text
    let help_text = vec![Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to submit  |  "),
        Span::styled(
            "Esc",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to cancel"),
    ])];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT));
    f.render_widget(help, dialog_chunks[2]);
}

pub(crate) fn render_help_overlay(f: &mut Frame, area: Rect, scroll_offset: usize, theme: &Theme) {
    // Create centered overlay area (80% width, 90% height)
    let overlay_width = (area.width * 80) / 100;
    let overlay_height = (area.height * 90) / 100;
    let overlay_x = (area.width - overlay_width) / 2;
    let overlay_y = (area.height - overlay_height) / 2;

    let overlay_area = Rect {
        x: overlay_x,
        y: overlay_y,
        width: overlay_width,
        height: overlay_height,
    };

    // Create help text with comprehensive keyboard shortcuts
    let help_lines = vec![
        Line::from(vec![Span::styled(
            "Ubertooth CLI - Keyboard Reference",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        // Global shortcuts
        Line::from(vec![Span::styled(
            "GLOBAL SHORTCUTS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  ?  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Show this help overlay"),
        ]),
        Line::from(vec![
            Span::styled(
                "  q  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Go back / Cancel current action"),
        ]),
        Line::from(vec![
            Span::styled(
                "  s  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Open settings"),
        ]),
        Line::from(vec![
            Span::styled(
                " 1-9 ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quick select menu item"),
        ]),
        Line::from(""),
        // Navigation
        Line::from(vec![Span::styled(
            "NAVIGATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate up/down in menus and lists"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select item / Confirm action"),
        ]),
        Line::from(vec![
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch between form fields"),
        ]),
        Line::from(vec![
            Span::styled(
                "PgUp ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/ "),
            Span::styled(
                "PgDn",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Page up/down in lists"),
        ]),
        Line::from(vec![
            Span::styled(
                "Home ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/ "),
            Span::styled(
                " End",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Jump to start/end of list"),
        ]),
        Line::from(""),
        // Capture management
        Line::from(vec![Span::styled(
            "CAPTURE MANAGEMENT (capture_list view)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Analyze selected capture"),
        ]),
        Line::from(vec![
            Span::styled(
                "  V  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("View capture details"),
        ]),
        Line::from(vec![
            Span::styled(
                "  D  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Delete capture (with confirmation)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  E  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Export capture to file"),
        ]),
        Line::from(vec![
            Span::styled(
                "  T  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Add/edit tags for capture"),
        ]),
        Line::from(""),
        // Packet analysis
        Line::from(vec![Span::styled(
            "PACKET ANALYSIS (bt_decode view)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate packet list"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" / "),
            Span::styled(
                "Space",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Expand/collapse packet details"),
        ]),
        Line::from(vec![
            Span::styled(
                "  b  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Bookmark packet (mark with ★)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  m  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Mark packet for comparison"),
        ]),
        Line::from(vec![
            Span::styled(
                "  f  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Follow stream (filter by MAC address)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  /  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Open filter dialog"),
        ]),
        Line::from(vec![
            Span::styled(
                "  e  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Open export menu"),
        ]),
        Line::from(vec![
            Span::styled(
                "  n  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Add/edit annotation for packet"),
        ]),
        Line::from(vec![
            Span::styled(
                "Del  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Delete annotation from packet"),
        ]),
        Line::from(""),
        // View modes
        Line::from(vec![Span::styled(
            "VIEW MODES (bt_decode)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  l  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch to List view"),
        ]),
        Line::from(vec![
            Span::styled(
                "  s  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch to Statistics view"),
        ]),
        Line::from(vec![
            Span::styled(
                "  t  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch to Timeline view"),
        ]),
        Line::from(vec![
            Span::styled(
                "  c  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch to Comparison view (side-by-side)"),
        ]),
        Line::from(""),
        // Analysis results
        Line::from(vec![Span::styled(
            "ANALYSIS RESULTS (bt_analyze view)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  o  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Overview mode - show summary"),
        ]),
        Line::from(vec![
            Span::styled(
                "  d  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Devices mode - interactive device list"),
        ]),
        Line::from(vec![
            Span::styled(
                "  s  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Security observations mode"),
        ]),
        Line::from(vec![
            Span::styled(
                "  t  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Timing analysis mode"),
        ]),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate items in devices/security modes"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Expand/collapse item details"),
        ]),
        Line::from(""),
        // Filter dialog
        Line::from(vec![Span::styled(
            "FILTER DIALOG",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate between filter sections"),
        ]),
        Line::from(vec![
            Span::styled(
                "←/→ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate packet types"),
        ]),
        Line::from(vec![
            Span::styled(
                "Space",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Toggle packet type selection"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Apply filters"),
        ]),
        Line::from(vec![
            Span::styled(
                "  C  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Clear all filters"),
        ]),
        Line::from(""),
        // Export menu
        Line::from(vec![Span::styled(
            "EXPORT MENU",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Select export option"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Execute export"),
        ]),
        Line::from(""),
        // Form input
        Line::from(vec![Span::styled(
            "TOOL PARAMETER FORMS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Move to next field"),
        ]),
        Line::from(vec![
            Span::styled(
                " ↑/↓ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Change dropdown selection"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Submit form and execute tool"),
        ]),
        Line::from(""),
        // Tips
        Line::from(vec![Span::styled(
            "TIPS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("• Press "),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" anytime to see this help overlay"),
        ]),
        Line::from(vec![
            Span::raw("• Use "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to go back or cancel dialogs"),
        ]),
        Line::from(vec![
            Span::raw("• Number keys "),
            Span::styled(
                "1-9",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" provide quick navigation shortcuts"),
        ]),
        Line::from(vec![
            Span::raw("• Most views support "),
            Span::styled(
                "PgUp/PgDn",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" for fast scrolling"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this help", Style::default().fg(Color::Gray)),
        ]),
    ];

    // Calculate visible lines based on scroll offset
    let max_lines = (overlay_height as usize).saturating_sub(4); // Account for borders and padding
    let total_lines = help_lines.len();
    let max_scroll = total_lines.saturating_sub(max_lines);
    let clamped_scroll = scroll_offset.min(max_scroll);

    let visible_lines: Vec<Line> = help_lines
        .into_iter()
        .skip(clamped_scroll)
        .take(max_lines)
        .collect();

    // Show scroll indicator if needed
    let scroll_indicator = if total_lines > max_lines {
        format!(" [↑/↓ to scroll {}/{}] ", clamped_scroll + 1, total_lines)
    } else {
        " ".to_string()
    };

    let help_text = Text::from(visible_lines);
    let help_widget = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(vec![
                    Span::raw(" "),
                    Span::styled(
                        "Keyboard Reference",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                ])
                .title_bottom(vec![
                    Span::raw(" "),
                    Span::styled(scroll_indicator, Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                ]),
        );

    // Clear background
    let clear_widget = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear_widget, area);

    // Render help overlay
    f.render_widget(help_widget, overlay_area);
}

/// Render theme selector dialog

pub(crate) fn render_theme_selector(
    f: &mut Frame,
    area: Rect,
    selected_index: usize,
    themes: &[Theme],
    current_theme: &Theme,
) {
    // Create centered dialog area (60% width, 70% height)
    let dialog_width = (area.width * 60) / 100;
    let dialog_height = (area.height * 70) / 100;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Create theme list items
    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let is_current = theme.name == current_theme.name;
            let is_selected = i == selected_index;

            let content = if is_current {
                format!("★ {} - {} (current)", theme.name, theme.description)
            } else {
                format!("  {} - {}", theme.name, theme.description)
            };

            let style = if is_selected {
                Style::default()
                    .fg(current_theme.colors.selected.to_color())
                    .bg(current_theme.colors.selected_bg.to_color())
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(current_theme.colors.success.to_color())
            } else {
                Style::default().fg(current_theme.colors.foreground.to_color())
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(current_theme.colors.border_focused.to_color()))
                .title(vec![
                    Span::raw(" "),
                    Span::styled(
                        "Select Theme",
                        Style::default()
                            .fg(current_theme.colors.title.to_color())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                ])
                .title_bottom(vec![
                    Span::raw(" "),
                    Span::styled(
                        "[↑/↓] Navigate  [Enter] Apply  [Esc] Cancel  [1-5] Quick Select",
                        Style::default().fg(current_theme.colors.dimmed.to_color()),
                    ),
                    Span::raw(" "),
                ]),
        )
        .highlight_style(
            Style::default()
                .bg(current_theme.colors.selected_bg.to_color())
                .add_modifier(Modifier::BOLD),
        );

    // Clear background with semi-transparent effect
    let clear_widget =
        Block::default().style(Style::default().bg(current_theme.colors.background.to_color()));
    f.render_widget(clear_widget, area);

    // Render theme list
    f.render_widget(list, dialog_area);
}

