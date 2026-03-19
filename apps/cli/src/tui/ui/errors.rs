//! Error and comparison rendering

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::utils::categorize_error;


/// Render notification as a centered popup at the bottom of the screen

/// Render confirmation dialog

/// Render error message with helpful formatting
pub(crate) fn render_error_message(f: &mut Frame, area: Rect, tool_name: &str, error_msg: &str) {
    // Parse common error patterns and provide helpful context
    let (category, message, suggestion) = categorize_error(error_msg);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Error Category: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                category,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("Details:", Style::default().fg(Color::Yellow))),
        Line::from(""),
    ];

    // Word-wrap the error message
    for chunk in message.chars().collect::<Vec<_>>().chunks(60) {
        let chunk_str: String = chunk.iter().collect();
        lines.push(Line::from(Span::styled(
            format!("  {}", chunk_str),
            Style::default().fg(Color::White),
        )));
    }

    lines.push(Line::from(""));

    if let Some(suggestion_text) = suggestion {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Suggestion:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", suggestion_text),
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Esc] Back to menu",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Error: {}", tool_name))
                .title_style(Style::default().fg(Color::Red)),
        );

    f.render_widget(paragraph, area);
}


/// Render capture comparison results
pub(crate) fn render_comparison_results(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if let Some(comparison) = output.get("comparison").and_then(|c| c.as_object()) {
        // Split into left and right panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut lines_left = Vec::new();
        let mut lines_right = Vec::new();

        // Header
        lines_left.push(Line::from(Span::styled(
            "Capture A",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines_right.push(Line::from(Span::styled(
            "Capture B",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Get capture IDs from output root
        let capture_a = output
            .get("capture_id_a")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let capture_b = output
            .get("capture_id_b")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");

        lines_left.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(capture_a, Style::default().fg(Color::White)),
        ]));
        lines_right.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Gray)),
            Span::styled(capture_b, Style::default().fg(Color::White)),
        ]));

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Similarity score (in both panels for emphasis)
        if let Some(similarity) = comparison
            .get("similarity_percent")
            .and_then(|s| s.as_f64())
        {
            let similarity_color = if similarity > 80.0 {
                Color::Green
            } else if similarity > 50.0 {
                Color::Yellow
            } else {
                Color::Red
            };

            lines_left.push(Line::from(Span::styled(
                format!("Similarity: {:.1}%", similarity),
                Style::default()
                    .fg(similarity_color)
                    .add_modifier(Modifier::BOLD),
            )));
            lines_right.push(Line::from(Span::styled(
                format!("Similarity: {:.1}%", similarity),
                Style::default()
                    .fg(similarity_color)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Statistics
        if let Some(common) = comparison.get("common_packets").and_then(|c| c.as_u64()) {
            lines_left.push(Line::from(vec![
                Span::styled("Common packets: ", Style::default().fg(Color::Gray)),
                Span::styled(common.to_string(), Style::default().fg(Color::Green)),
            ]));
            lines_right.push(Line::from(vec![
                Span::styled("Common packets: ", Style::default().fg(Color::Gray)),
                Span::styled(common.to_string(), Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(unique_a) = comparison.get("unique_to_a").and_then(|u| u.as_u64()) {
            lines_left.push(Line::from(vec![
                Span::styled("Unique packets: ", Style::default().fg(Color::Gray)),
                Span::styled(unique_a.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        }

        if let Some(unique_b) = comparison.get("unique_to_b").and_then(|u| u.as_u64()) {
            lines_right.push(Line::from(vec![
                Span::styled("Unique packets: ", Style::default().fg(Color::Gray)),
                Span::styled(unique_b.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        }

        lines_left.push(Line::from(""));
        lines_right.push(Line::from(""));

        // Differences
        if let Some(diffs) = comparison.get("differences").and_then(|d| d.as_array()) {
            if !diffs.is_empty() {
                lines_left.push(Line::from(Span::styled(
                    "Differences Detected",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                lines_right.push(Line::from(Span::styled(
                    "Differences Detected",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));

                for diff in diffs.iter().take(10) {
                    // Show first 10 differences
                    if let Some(diff_str) = diff.as_str() {
                        lines_left.push(Line::from(vec![
                            Span::styled("• ", Style::default().fg(Color::Red)),
                            Span::styled(diff_str, Style::default().fg(Color::White)),
                        ]));
                        lines_right.push(Line::from(""));
                    } else if let Some(diff_obj) = diff.as_object() {
                        // Structured difference
                        let field = diff_obj
                            .get("field")
                            .and_then(|f| f.as_str())
                            .unwrap_or("unknown");
                        let val_a = diff_obj
                            .get("value_a")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        let val_b = diff_obj
                            .get("value_b")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");

                        lines_left.push(Line::from(vec![
                            Span::styled(format!("{}: ", field), Style::default().fg(Color::Gray)),
                            Span::styled(val_a, Style::default().fg(Color::Yellow)),
                        ]));
                        lines_right.push(Line::from(vec![
                            Span::styled(format!("{}: ", field), Style::default().fg(Color::Gray)),
                            Span::styled(val_b, Style::default().fg(Color::Yellow)),
                        ]));
                    }
                }

                if diffs.len() > 10 {
                    lines_left.push(Line::from(Span::styled(
                        format!("... and {} more differences", diffs.len() - 10),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            } else {
                lines_left.push(Line::from(Span::styled(
                    "✓ No significant differences",
                    Style::default().fg(Color::Green),
                )));
                lines_right.push(Line::from(Span::styled(
                    "✓ No significant differences",
                    Style::default().fg(Color::Green),
                )));
            }
        }

        // Render left panel (Capture A)
        let left_content = Paragraph::new(Text::from(lines_left))
            .block(Block::default().borders(Borders::ALL).title(" Capture A "))
            .wrap(Wrap { trim: false });
        f.render_widget(left_content, chunks[0]);

        // Render right panel (Capture B)
        let right_content = Paragraph::new(Text::from(lines_right))
            .block(Block::default().borders(Borders::ALL).title(" Capture B "))
            .wrap(Wrap { trim: false });
        f.render_widget(right_content, chunks[1]);
    } else {
        // Fallback if comparison object is missing
        let error_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Error: Invalid comparison result",
                Style::default().fg(Color::Red),
            )),
        ];
        let content = Paragraph::new(Text::from(error_text)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Comparison Error "),
        );
        f.render_widget(content, area);
    }
}


