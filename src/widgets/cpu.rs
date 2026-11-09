use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Chart, Dataset};
use ratatui::Frame;

use crate::app::App;

/// Render CPU usage gauges for each core (uses frame area, overridden by ui.rs)
#[allow(dead_code)]
pub fn render_cpu_gauges(_app: &App, _frame: &mut Frame) {
    // This is overridden by ui.rs render_cpu_gauges_in_area
}

/// Render CPU usage chart over time
pub fn render_cpu_chart(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.cpu_total_history.is_empty() {
        return;
    }

    // Build the overall CPU dataset
    let cpu_data: Vec<(f64, f64)> = app
        .cpu_total_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let cpu_dataset = Dataset::default()
        .name("CPU %")
        .marker(symbols::Marker::Dot)
        .graph_type(ratatui::widgets::GraphType::Line)
        .data(&cpu_data)
        .style(Style::default().fg(Color::Magenta));

    let chart = Chart::new(vec![cpu_dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .title(
                    Span::raw(" CPU History (60s) ")
                        .style(
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .title_alignment(ratatui::layout::Alignment::Center),
        )
        .x_axis(
            ratatui::widgets::Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, app.cpu_total_history.len() as f64]),
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0]),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(chart, area);
}
