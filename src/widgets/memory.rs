use ratatui::layout::Layout;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, LineGauge, Sparkline};
use ratatui::Frame;

use crate::app::App;

/// Render memory and swap usage bars with sparkline history
pub fn render_memory(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let total_mem = app.system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_mem = app.system.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let total_swap = app.system.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_swap = app.system.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);

    let mem_ratio = if total_mem > 0.0 {
        used_mem / total_mem
    } else {
        0.0
    };
    let swap_ratio = if total_swap > 0.0 {
        used_swap / total_swap
    } else {
        0.0
    };

    // Color based on usage
    let mem_color = if mem_ratio < 0.5 {
        Color::Green
    } else if mem_ratio < 0.8 {
        Color::Yellow
    } else {
        Color::Red
    };

    let swap_color = if swap_ratio < 0.5 {
        Color::Green
    } else if swap_ratio < 0.8 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Memory gauge
    let mem_gauge = LineGauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(
                    Span::raw(" Memory ")
                        .style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .title_alignment(ratatui::layout::Alignment::Center),
        )
        .ratio(mem_ratio)
        .filled_style(Style::default().fg(mem_color))
        .style(Style::default().fg(Color::DarkGray))
        .label(
            Span::raw(format!(
                "{:.1} GB / {:.1} GB ({:.0}%)",
                used_mem,
                total_mem,
                mem_ratio * 100.0
            ))
            .style(Style::default().fg(Color::White)),
        );

    // Swap gauge
    let swap_gauge = LineGauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(
                    Span::raw(" Swap ")
                        .style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .title_alignment(ratatui::layout::Alignment::Center),
        )
        .ratio(swap_ratio)
        .filled_style(Style::default().fg(swap_color))
        .style(Style::default().fg(Color::DarkGray))
        .label(
            Span::raw(format!(
                "{:.1} GB / {:.1} GB ({:.0}%)",
                used_swap,
                total_swap,
                swap_ratio * 100.0
            ))
            .style(Style::default().fg(Color::White)),
        );

    // Split area for memory and swap
    let areas: Vec<_> = Layout::horizontal([
        ratatui::layout::Constraint::Percentage(50),
        ratatui::layout::Constraint::Percentage(50),
    ])
    .split(area)
    .iter()
    .copied()
    .collect();

    let mem_area = areas[0];
    let swap_area = areas[1];

    frame.render_widget(mem_gauge, mem_area);
    frame.render_widget(swap_gauge, swap_area);

    // Render sparklines below memory/swap
    let sparkline_height = 3u16;
    let spark_areas: Vec<_> = Layout::vertical([
        ratatui::layout::Constraint::Min(0),
        ratatui::layout::Constraint::Length(sparkline_height),
    ])
    .split(area)
    .iter()
    .copied()
    .collect();
    let sparkline_area = spark_areas[1];

    let spark_cols: Vec<_> = Layout::horizontal([
        ratatui::layout::Constraint::Percentage(50),
        ratatui::layout::Constraint::Percentage(50),
    ])
    .split(sparkline_area)
    .iter()
    .copied()
    .collect();

    let mem_spark_area = spark_cols[0];
    let swap_spark_area = spark_cols[1];

    // Memory sparkline - convert to u64 for Sparkline
    let mem_spark_data: Vec<u64> = app
        .memory_history
        .iter()
        .map(|&v| {
            ((v / total_mem * 255.0).clamp(0.0, 255.0)) as u64
        })
        .collect();

    if !mem_spark_data.is_empty() {
        let mem_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .data(&mem_spark_data)
            .style(Style::default().fg(mem_color))
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(mem_sparkline, mem_spark_area);
    }

    // Swap sparkline
    let swap_spark_data: Vec<u64> = app
        .swap_history
        .iter()
        .map(|&v| {
            if total_swap > 0.0 {
                ((v / total_swap * 255.0).clamp(0.0, 255.0)) as u64
            } else {
                0u64
            }
        })
        .collect();

    if !swap_spark_data.is_empty() {
        let swap_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .data(&swap_spark_data)
            .style(Style::default().fg(swap_color))
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(swap_sparkline, swap_spark_area);
    }
}
