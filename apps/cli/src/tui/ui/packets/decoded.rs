//! Decoded packet display rendering

use ratatui::{
    layout::Rect,
    Frame,
};

use super::comparison::render_packet_comparison;
use super::list::render_packet_list;
use super::statistics::render_packet_statistics;
use super::timeline::render_packet_timeline;


/// Categorize error and provide helpful suggestions
/// Render decoded packet list for bt_decode
pub(crate) fn render_decoded_packets(
    f: &mut Frame,
    area: Rect,
    output: &serde_json::Value,
    packet_list_state: Option<&crate::tui::app::PacketListState>,
) {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let packets = match output.get("decoded_packets").and_then(|p| p.as_array()) {
        Some(p) => p,
        None => {
            let text = vec![
                Line::from(""),
                Line::from("No packets to display"),
                Line::from(""),
                Line::from(Span::styled(
                    "The capture may be empty or the decode limit was 0.",
                    Style::default().fg(Color::Gray),
                )),
            ];
            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Decoded Packets"),
                )
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(paragraph, area);
            return;
        }
    };

    let packet_count = packets.len();
    if packet_count == 0 {
        let text = vec![
            Line::from(""),
            Line::from("No packets to display"),
            Line::from(""),
            Line::from(Span::styled(
                "The capture may be empty or the decode limit was 0.",
                Style::default().fg(Color::Gray),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Decoded Packets"),
            )
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let default_state = crate::tui::app::PacketListState::new();
    let state = packet_list_state.unwrap_or(&default_state);

    // Dispatch to appropriate view based on view_mode
    match state.view_mode {
        crate::tui::app::PacketViewMode::List => {
            render_packet_list(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Statistics => {
            render_packet_statistics(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Timeline => {
            render_packet_timeline(f, area, packets, state);
        }
        crate::tui::app::PacketViewMode::Comparison => {
            render_packet_comparison(f, area, packets, state);
        }
    }
}

