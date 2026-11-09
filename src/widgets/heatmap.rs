use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Process Heatmap - shows CPU usage of top processes over time.
/// Similar to GitHub's contribution graph, each cell represents a process's
/// CPU usage at a point in time, colored by intensity.
pub fn render_process_heatmap(app: &App, frame: &mut Frame, area: Rect) {
    let history = &app.process_cpu_history;

    if history.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(
                Span::raw(" Process Heatmap ")
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .title_alignment(ratatui::layout::Alignment::Center);

        let text = Text::from(vec![
            Line::from(Span::raw("Collecting process data...")
                .style(Style::default().fg(Color::Gray))),
        ]);
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;

    if width < 20 || height < 5 {
        return;
    }

    let chart_width = width.saturating_sub(20);
    let chart_height = height.saturating_sub(4);

    if chart_width < 5 || chart_height < 2 {
        return;
    }

    let mut processes: Vec<_> = history.iter().collect();
    processes.sort_by(|a, b| {
        let cpu_a = a.1.back().copied().unwrap_or(0.0);
        let cpu_b = b.1.back().copied().unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let display_count = std::cmp::min(processes.len(), chart_height);

    // Pre-allocate lines vector
    let mut lines: Vec<Line> = Vec::with_capacity(4 + display_count);

    // Header
    lines.push(
        Line::from(Span::raw(format!("  Process Heatmap (Tab:switch)  {}s window", history.len()))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    );
    lines.push(Line::default());

    // Pre-allocate reusable line buffer
    let mut line_buf = String::with_capacity(20 + chart_width);

    // Render each process row
    for (_row_idx, (name, cpu_data)) in processes.iter().enumerate().take(display_count) {
        line_buf.clear();
        let name_truncated: String = name.chars().take(16).collect();
        line_buf.push_str(&format!("{name_truncated:<16}"));
        line_buf.push_str(" │ ");

        for col in 0..chart_width {
            let time_idx = (col as f64 / chart_width as f64 * cpu_data.len() as f64) as usize;
            let time_idx = time_idx.min(cpu_data.len() - 1);
            let cpu = cpu_data[time_idx];

            let (ch, _color) = cpu_heatmap_style(cpu);
            line_buf.push(ch);
        }

        let avg_cpu: f32 = cpu_data.iter().sum::<f32>() / cpu_data.len() as f32;
        let (_, line_color) = cpu_heatmap_style(avg_cpu);

        lines.push(Line::from(Span::raw(line_buf.clone()).style(Style::default().fg(line_color))));
    }

    // Time axis (avoid .to_string() + loop)
    line_buf.clear();
    line_buf.push_str("     │");
    for _ in 0..chart_width {
        line_buf.push('─');
    }
    lines.push(
        Line::from(Span::raw(line_buf.clone())
            .style(Style::default().fg(Color::Gray))),
    );

    // Time labels (avoid .repeat() allocation)
    line_buf.clear();
    line_buf.push_str("     │");
    let spaces = chart_width / 2 - 4;
    for _ in 0..spaces {
        line_buf.push(' ');
    }
    line_buf.push_str("now│");
    lines.push(
        Line::from(Span::raw(line_buf)
            .style(Style::default().fg(Color::Gray))),
    );

    // Legend (static string)
    lines.push(
        Line::from(Span::raw("     │  ▂ Low  ▃ Medium  ▄ High  ▅ Very High  ▆ Critical  │")
            .style(Style::default().fg(Color::Gray))),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(
            Span::raw(format!(" Process Heatmap ({display_count} processes) "))
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center)
        .title_bottom(
            Span::raw(" ▂<5%  ▃5-20%  ▄20-50%  ▅50-80%  ▆>80% ")
                .style(Style::default().fg(Color::Gray)),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}

/// Get the character and color for a CPU usage value in the heatmap
fn cpu_heatmap_style(cpu: f32) -> (char, Color) {
    if cpu < 1.0 {
        (' ', Color::Reset)
    } else if cpu < 5.0 {
        ('▁', Color::DarkGray)
    } else if cpu < 10.0 {
        ('▂', Color::Blue)
    } else if cpu < 20.0 {
        ('▃', Color::Cyan)
    } else if cpu < 40.0 {
        ('▄', Color::Green)
    } else if cpu < 60.0 {
        ('▅', Color::Yellow)
    } else if cpu < 80.0 {
        ('▆', Color::Magenta)
    } else if cpu < 95.0 {
        ('▇', Color::Red)
    } else {
        ('█', Color::Red)
    }
}

/// Render a detailed heatmap with multi-color cells using styled spans
pub fn render_detailed_heatmap(app: &App, frame: &mut Frame, area: Rect) {
    let history = &app.process_cpu_history;

    if history.is_empty() {
        render_process_heatmap(app, frame, area);
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;

    if width < 20 || height < 5 {
        return;
    }

    let chart_width = width.saturating_sub(20);
    let chart_height = height.saturating_sub(4);

    if chart_width < 5 || chart_height < 2 {
        return;
    }

    let mut processes: Vec<_> = history.iter().collect();
    processes.sort_by(|a, b| {
        let cpu_a = a.1.back().copied().unwrap_or(0.0);
        let cpu_b = b.1.back().copied().unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let display_count = std::cmp::min(processes.len(), chart_height);

    // Pre-allocate lines vector
    let mut lines: Vec<ratatui::text::Line> = Vec::with_capacity(3 + display_count);

    // Header
    lines.push(
        ratatui::text::Line::from(vec![
            Span::raw(" Process Heatmap ").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" ({}s window)", history.len()))
                .style(Style::default().fg(Color::Gray)),
        ])
    );
    lines.push(ratatui::text::Line::default());

    // Reusable style for name labels
    let name_style = Style::default().fg(Color::White);

    // Render each process row with multi-color cells
    for (_row_idx, (name, cpu_data)) in processes.iter().enumerate().take(display_count) {
        let name_truncated: String = name.chars().take(16).collect();
        let name_padded = format!("{name_truncated:<16}");

        // Pre-allocate spans: 1 name + chart_width cells
        let mut spans = Vec::with_capacity(1 + chart_width);
        spans.push(
            Span::raw(format!("{name_padded} │ "))
                .style(name_style),
        );

        for col in 0..chart_width {
            let time_idx = (col as f64 / chart_width as f64 * cpu_data.len() as f64) as usize;
            let time_idx = time_idx.min(cpu_data.len() - 1);
            let cpu = cpu_data[time_idx];

            let (ch, color) = cpu_heatmap_style(cpu);
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }

        lines.push(ratatui::text::Line::from(spans));
    }

    // Time axis — build axis string without .repeat()
    let mut axis_str = String::with_capacity(chart_width);
    for _ in 0..chart_width {
        axis_str.push('─');
    }
    lines.push(ratatui::text::Line::from(vec![
        Span::raw("     │").style(Style::default().fg(Color::Gray)),
        Span::raw(axis_str).style(Style::default().fg(Color::Gray)),
    ]));

    // Legend with colors (static spans — no allocation)
    lines.push(ratatui::text::Line::from(vec![
        Span::raw("     │  ").style(Style::default().fg(Color::Gray)),
        Span::styled("▂ Low", Style::default().fg(Color::Blue)),
        Span::raw("  ").style(Style::default().fg(Color::Gray)),
        Span::styled("▃ Med", Style::default().fg(Color::Cyan)),
        Span::raw("  ").style(Style::default().fg(Color::Gray)),
        Span::styled("▄ High", Style::default().fg(Color::Green)),
        Span::raw("  ").style(Style::default().fg(Color::Gray)),
        Span::styled("▅ Very", Style::default().fg(Color::Yellow)),
        Span::raw("  ").style(Style::default().fg(Color::Gray)),
        Span::styled("▆ Crit", Style::default().fg(Color::Red)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(
            Span::raw(format!(" Process Heatmap ({display_count} processes) "))
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}
