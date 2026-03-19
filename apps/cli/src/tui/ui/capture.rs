//! Capture results rendering

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};


/// Render capture list as a formatted table
pub(crate) fn render_capture_list_table(
    f: &mut Frame,
    area: Rect,
    captures: &[serde_json::Value],
    selected_index: Option<usize>,
) {
    let mut lines = Vec::new();

    // Header line
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Found {} capture(s)", captures.len()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    // Show navigation hint if captures available
    if !captures.is_empty() {
        lines.push(Line::from(Span::styled(
            "  [↑/↓] Navigate  [Enter] Analyze  [V] View  [D] Delete  [E] Export  [T] Tag  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));

    // If no captures, show message
    if captures.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No captures found. Run a scan to create captures.",
            Style::default().fg(Color::Gray),
        )));
    } else {
        // Table header
        lines.push(Line::from(vec![
            Span::styled(
                "  ID                  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Type           ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Pkts    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Duration    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Timestamp              ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Description",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::raw(format!("  {}", "-".repeat(120)))));

        // Table rows
        for (idx, capture) in captures.iter().enumerate() {
            let is_selected = selected_index == Some(idx);
            let id = capture
                .get("capture_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .split('-')
                .nth(2)
                .unwrap_or("?")
                .chars()
                .take(16)
                .collect::<String>();

            let cap_type = capture.get("type").and_then(|v| v.as_str()).unwrap_or("?");

            let packet_count = capture
                .get("packet_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let duration = capture
                .get("duration_sec")
                .and_then(|v| v.as_u64())
                .map(|d| format!("{}s", d))
                .unwrap_or_else(|| "N/A".to_string());

            let timestamp = capture
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .chars()
                .take(19)
                .collect::<String>();

            let description = capture
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags = capture
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            // Main row - highlight if selected
            let row_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "> " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(format!("{}{:<18}  ", prefix, id), row_style.fg(Color::Cyan)),
                Span::styled(format!("{:<13}  ", cap_type), row_style.fg(Color::White)),
                Span::styled(format!("{:<6}  ", packet_count), row_style.fg(Color::Green)),
                Span::styled(format!("{:<10}  ", duration), row_style.fg(Color::Blue)),
                Span::styled(format!("{:<21}  ", timestamp), row_style.fg(Color::Gray)),
                Span::styled(description, row_style.fg(Color::White)),
            ]));

            // Tags row if present
            if !tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Tags: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tags, Style::default().fg(Color::DarkGray)),
                ]));
            }

            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Esc] Back to menu",
        Style::default().fg(Color::DarkGray),
    )));

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Capture List")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(content, area);
}


/// Render capture details in readable format
pub(crate) fn render_capture_details(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(obj) = output.as_object() {
        // Capture ID
        if let Some(id) = obj.get("capture_id").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled(
                    "  Capture ID: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(id, Style::default().fg(Color::Cyan)),
            ]));
        }

        // Type
        if let Some(cap_type) = obj.get("type").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::raw("  Type: "),
                Span::styled(cap_type, Style::default().fg(Color::White)),
            ]));
        }

        // Timestamp
        if let Some(timestamp) = obj.get("timestamp").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::raw("  Timestamp: "),
                Span::styled(timestamp, Style::default().fg(Color::Gray)),
            ]));
        }
        lines.push(Line::from(""));

        // Stats
        lines.push(Line::from(Span::styled(
            "  Statistics",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(packets) = obj.get("packet_count").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::raw("    Packets: "),
                Span::styled(packets.to_string(), Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(duration) = obj.get("duration_sec").and_then(|v| v.as_u64()) {
            lines.push(Line::from(vec![
                Span::raw("    Duration: "),
                Span::styled(format!("{}s", duration), Style::default().fg(Color::Blue)),
            ]));
        }

        if let Some(size) = obj.get("file_size_bytes").and_then(|v| v.as_u64()) {
            let size_kb = size as f64 / 1024.0;
            lines.push(Line::from(vec![
                Span::raw("    File Size: "),
                Span::styled(
                    format!("{:.2} KB", size_kb),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }
        lines.push(Line::from(""));

        // Tags
        if let Some(tags) = obj.get("tags").and_then(|v| v.as_array()) {
            if !tags.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Tags",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                let tag_str = tags
                    .iter()
                    .filter_map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(Line::from(format!("    {}", tag_str)));
                lines.push(Line::from(""));
            }
        }

        // Description
        if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
            if !desc.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Description",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(format!("    {}", desc)));
                lines.push(Line::from(""));
            }
        }

        // PCAP Path
        if let Some(path) = obj.get("pcap_path").and_then(|v| v.as_str()) {
            lines.push(Line::from(vec![
                Span::styled("  PCAP File: ", Style::default().fg(Color::Gray)),
                Span::styled(path, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Capture Details"),
    );
    f.render_widget(content, area);
}


