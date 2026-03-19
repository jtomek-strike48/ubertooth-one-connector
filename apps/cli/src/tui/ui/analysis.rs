//! Analysis results rendering

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};


/// Render timing analysis view
pub(crate) fn render_analysis_timing(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            lines.push(Line::from(Span::styled(
                "Timing Analysis",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if let Some(duration) = timing.get("duration_sec").and_then(|d| d.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("Capture Duration: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:.2}s", duration),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            if let Some(pps) = timing.get("packets_per_sec").and_then(|p| p.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("Packets per Second: ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:.2}", pps), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Inter-Packet Intervals",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Average: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", avg), Style::default().fg(Color::Green)),
                ]));
            }

            if let Some(min) = timing.get("min_interval_ms").and_then(|m| m.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Minimum: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", min), Style::default().fg(Color::Blue)),
                ]));
            }

            if let Some(max) = timing.get("max_interval_ms").and_then(|m| m.as_f64()) {
                lines.push(Line::from(vec![
                    Span::styled("  Maximum: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.2}ms", max), Style::default().fg(Color::Red)),
                ]));
            }

            if let Some(count) = timing.get("intervals_calculated").and_then(|c| c.as_u64()) {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Intervals analyzed: ", Style::default().fg(Color::Gray)),
                    Span::styled(count.to_string(), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("o/d/s/t", Style::default().fg(Color::Cyan)),
                Span::styled(" to change view", Style::default().fg(Color::Gray)),
            ]));
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Timing Analysis "),
    );
    f.render_widget(content, area);
}


/// Render security observations view
pub(crate) fn render_analysis_security(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: &crate::tui::app::AnalysisViewState,
) {
    let mut lines = Vec::new();

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Security summary
        if let Some(summary) = analysis.get("security_summary").and_then(|s| s.as_object()) {
            lines.push(Line::from(Span::styled(
                "Security Summary",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(privacy) = summary
                .get("privacy_enabled_devices")
                .and_then(|p| p.as_u64())
            {
                lines.push(Line::from(vec![
                    Span::raw("  Privacy-enabled devices: "),
                    Span::styled(privacy.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(public) = summary
                .get("public_address_devices")
                .and_then(|p| p.as_u64())
            {
                lines.push(Line::from(vec![
                    Span::raw("  Public address devices: "),
                    Span::styled(public.to_string(), Style::default().fg(Color::Yellow)),
                ]));
            }
            if let Some(conn) = summary.get("connection_requests").and_then(|c| c.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Connection requests: "),
                    Span::styled(conn.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            if let Some(scan) = summary.get("scan_requests").and_then(|s| s.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Scan requests: "),
                    Span::styled(scan.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Detailed observations
        if let Some(observations) = analysis
            .get("security_observations")
            .and_then(|s| s.as_array())
        {
            if observations.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No security observations",
                    Style::default().fg(Color::Green),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Observations",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));

                for (idx, obs) in observations.iter().enumerate() {
                    let is_selected = idx == state.selected_index;
                    let is_expanded = state.is_expanded(idx);

                    let expand_icon = if is_expanded { "▼" } else { "▶" };

                    // Handle both string and object observations
                    let (obs_type, description, severity) = if let Some(obj) = obs.as_object() {
                        (
                            obj.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("Unknown"),
                            obj.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or(""),
                            obj.get("severity")
                                .and_then(|s| s.as_str())
                                .unwrap_or("info"),
                        )
                    } else if let Some(text) = obs.as_str() {
                        ("Observation", text, "info")
                    } else {
                        ("Unknown", "", "info")
                    };

                    let severity_color = match severity {
                        "critical" => Color::Red,
                        "high" => Color::LightRed,
                        "medium" => Color::Yellow,
                        "low" => Color::Blue,
                        _ => Color::Gray,
                    };

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}] ", severity.to_uppercase()),
                            Style::default()
                                .fg(severity_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(obs_type, style),
                    ]));

                    if is_expanded {
                        lines.push(Line::from(vec![
                            Span::raw("  │ "),
                            Span::styled(description, Style::default().fg(Color::White)),
                        ]));
                        if let Some(obj) = obs.as_object() {
                            if let Some(device) =
                                obj.get("affected_device").and_then(|d| d.as_str())
                            {
                                lines.push(Line::from(vec![
                                    Span::raw("  │ "),
                                    Span::styled("Affected: ", Style::default().fg(Color::Gray)),
                                    Span::styled(device, Style::default().fg(Color::Cyan)),
                                ]));
                            }
                        }
                        lines.push(Line::from("  │"));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw(" expand/collapse"),
                ]));
            }
        }
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Security Analysis "),
    );
    f.render_widget(content, area);
}


/// Render devices view with interactive list
pub(crate) fn render_analysis_devices(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: &crate::tui::app::AnalysisViewState,
) {
    let mut lines = Vec::new();

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        if let Some(devices) = analysis.get("devices").and_then(|d| d.as_array()) {
            if devices.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "No devices detected in this capture",
                    Style::default().fg(Color::Gray),
                )));
            } else {
                for (idx, device) in devices.iter().enumerate() {
                    let is_selected = idx == state.selected_index;
                    let is_expanded = state.is_expanded(idx);

                    let expand_icon = if is_expanded { "▼" } else { "▶" };
                    let mac = device
                        .get("mac_address")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown");
                    let name = device
                        .get("device_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Unknown");
                    let pkts = device
                        .get("packet_count")
                        .and_then(|p| p.as_u64())
                        .unwrap_or(0);

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(expand_icon, style),
                        Span::raw(" "),
                        Span::styled(mac, style.fg(Color::Cyan)),
                        Span::raw(" - "),
                        Span::styled(name, style),
                        Span::styled(
                            format!(" ({} pkts)", pkts),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));

                    if is_expanded {
                        // Show device details
                        if let Some(rssi) = device.get("rssi").and_then(|r| r.as_i64()) {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("RSSI: ", Style::default().fg(Color::Gray)),
                                Span::styled(
                                    format!("{} dBm", rssi),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                        if let Some(pdu) = device.get("pdu_type").and_then(|p| p.as_str()) {
                            lines.push(Line::from(vec![
                                Span::raw("  │ "),
                                Span::styled("PDU Type: ", Style::default().fg(Color::Gray)),
                                Span::styled(pdu, Style::default().fg(Color::White)),
                            ]));
                        }
                        if let Some(first) = device.get("first_seen").and_then(|f| f.as_f64()) {
                            if let Some(last) = device.get("last_seen").and_then(|l| l.as_f64()) {
                                let duration = last - first;
                                lines.push(Line::from(vec![
                                    Span::raw("  │ "),
                                    Span::styled(
                                        "Active Duration: ",
                                        Style::default().fg(Color::Gray),
                                    ),
                                    Span::styled(
                                        format!("{:.2}s", duration),
                                        Style::default().fg(Color::White),
                                    ),
                                ]));
                            }
                        }
                        lines.push(Line::from("  │"));
                    }
                }

                // Footer
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" navigate  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw(" expand/collapse  "),
                    Span::styled("o/d/s/t", Style::default().fg(Color::Cyan)),
                    Span::raw(" change view"),
                ]));
            }
        }
    }

    let content =
        Paragraph::new(Text::from(lines)).block(Block::default().borders(Borders::ALL).title(
            format!(" Devices ({} total) ",
            output.get("analysis")
                .and_then(|a| a.get("devices"))
                .and_then(|d| d.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        ),
        ));
    f.render_widget(content, area);
}


/// Render analysis overview (summary of all sections)
pub(crate) fn render_analysis_overview(f: &mut Frame, area: Rect, output: &serde_json::Value) {
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if let Some(analysis) = output.get("analysis").and_then(|a| a.as_object()) {
        // Protocol Summary
        if let Some(proto) = analysis.get("protocol_summary").and_then(|p| p.as_object()) {
            lines.push(Line::from(Span::styled(
                "Protocol Summary",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            if let Some(ptype) = proto.get("type").and_then(|t| t.as_str()) {
                lines.push(Line::from(vec![
                    Span::raw("  Type: "),
                    Span::styled(ptype, Style::default().fg(Color::Cyan)),
                ]));
            }
            if let Some(count) = proto.get("packet_count").and_then(|c| c.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Packets: "),
                    Span::styled(count.to_string(), Style::default().fg(Color::Green)),
                ]));
            }
            if let Some(devices) = proto.get("unique_devices").and_then(|d| d.as_u64()) {
                lines.push(Line::from(vec![
                    Span::raw("  Devices: "),
                    Span::styled(devices.to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Quick stats
        if let Some(devices) = analysis.get("devices").and_then(|d| d.as_array()) {
            lines.push(Line::from(vec![
                Span::styled("📱 Devices: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} found", devices.len()),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled(
                    "(press 'd' for details)",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        if let Some(security) = analysis
            .get("security_observations")
            .and_then(|s| s.as_array())
        {
            if !security.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("🔒 Security: ", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{} observations", security.len()),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        "(press 's' for details)",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }

        if let Some(timing) = analysis.get("timing_analysis").and_then(|t| t.as_object()) {
            if let Some(avg) = timing.get("avg_interval_ms").and_then(|a| a.as_f64()) {
                if avg > 0.0 {
                    lines.push(Line::from(vec![
                        Span::styled("⏱️  Timing: ", Style::default().fg(Color::Blue)),
                        Span::styled(
                            format!("{:.2}ms avg", avg),
                            Style::default().fg(Color::White),
                        ),
                        Span::raw("  "),
                        Span::styled(
                            "(press 't' for details)",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        // Navigation hint
        lines.push(Line::from(vec![
            Span::styled("Navigation: ", Style::default().fg(Color::Yellow)),
            Span::styled("o", Style::default().fg(Color::Cyan)),
            Span::raw(" overview  "),
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" devices  "),
            Span::styled("s", Style::default().fg(Color::Cyan)),
            Span::raw(" security  "),
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(" timing"),
        ]));
    }

    let content = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Analysis Overview "),
    );
    f.render_widget(content, area);
}


/// Render analysis results in readable format
pub(crate) fn render_analysis_results(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    state: Option<&crate::tui::app::AnalysisViewState>,
) {
    use crate::tui::app::AnalysisViewMode;

    // Get state or use default
    let default_state = crate::tui::app::AnalysisViewState::new();
    let state = state.unwrap_or(&default_state);

    // Dispatch based on view mode
    match state.view_mode {
        AnalysisViewMode::Overview => render_analysis_overview(f, area, output),
        AnalysisViewMode::Devices => render_analysis_devices(f, area, output, state),
        AnalysisViewMode::Security => render_analysis_security(f, area, output, state),
        AnalysisViewMode::Timing => render_analysis_timing(f, area, output),
    }
}


