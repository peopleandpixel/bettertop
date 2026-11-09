use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// CPU Flame Chart - a horizontal histogram showing CPU usage distribution over time.
/// Unlike a line chart, this shows the "shape" of CPU load as stacked bars,
/// making it easy to see spikes and patterns at a glance.
pub fn render_flame_chart(app: &App, frame: &mut Frame, area: Rect) {
    let history = app.cpu_total_history.as_slices();
    // Combine the two slices from VecDeque::as_slices() into a single view
    let history: Vec<f32> = if history.0.is_empty() {
        history.1.to_vec()
    } else if history.1.is_empty() {
        history.0.to_vec()
    } else {
        let mut combined = Vec::with_capacity(history.0.len() + history.1.len());
        combined.extend_from_slice(history.0);
        combined.extend_from_slice(history.1);
        combined
    };

    if history.is_empty() {
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;

    if width < 10 || height < 3 {
        return;
    }

    // Available height for the chart (minus borders and labels)
    let chart_height = height.saturating_sub(4);
    if chart_height == 0 {
        return;
    }

    let history_len = history.len();
    let step = 100.0 / chart_height as f32;

    // Pre-allocate line buffer (label width + chart width)
    let mut line_buf = String::with_capacity(6 + width);

    // Build the flame chart as lines
    let mut lines: Vec<Line> = Vec::with_capacity(height);

    // Add time labels at the top (avoid .repeat() allocation)
    let padding = width.saturating_sub(20);
    line_buf.clear();
    line_buf.push_str("  ");
    line_buf.push_str(&history_len.to_string());
    line_buf.push_str("s ago");
    for _ in 0..padding {
        line_buf.push(' ');
    }
    line_buf.push_str("       now");
    lines.push(
        Line::from(Span::raw(line_buf.clone())
            .style(Style::default().fg(Color::Gray))),
    );

    // For each row (from top = 100% to bottom = 0%)
    for row in (0..chart_height).rev() {
        let threshold = (row as f32 / chart_height as f32) * 100.0;
        line_buf.clear();

        // Percentage label on the left
        line_buf.push_str(&format!("{:>3}% │", threshold as i32));

        // For each column (time sample)
        let cell_threshold = threshold + step;
        for col in 0..width {
            let idx = (col as f64 / width as f64 * history_len as f64) as usize;
            let idx = idx.min(history_len - 1);
            let value = history[idx];

            if value >= cell_threshold {
                // Choose character based on intensity
                let intensity = if (100.0 - threshold) > 0.0 {
                    (value - threshold) / (100.0 - threshold)
                } else {
                    0.0
                };
                let ch = if intensity > 0.8 {
                    '█'
                } else if intensity > 0.6 {
                    '▓'
                } else if intensity > 0.4 {
                    '▒'
                } else if intensity > 0.2 {
                    '░'
                } else {
                    '·'
                };
                line_buf.push(ch);
            } else {
                line_buf.push(' ');
            }
        }

        // Color based on the row's threshold
        let row_color = if threshold > 80.0 {
            Color::Red
        } else if threshold > 50.0 {
            Color::Yellow
        } else if threshold > 20.0 {
            Color::Green
        } else {
            Color::DarkGray
        };

        lines.push(Line::from(Span::raw(line_buf.clone()).style(Style::default().fg(row_color))));
    }

    // Add bottom axis (avoid .repeat() allocation)
    line_buf.clear();
    line_buf.push_str("     │");
    for _ in 0..width {
        line_buf.push('─');
    }
    lines.push(
        Line::from(Span::raw(line_buf.clone())
            .style(Style::default().fg(Color::Gray))),
    );

    // Calculate statistics (single pass)
    let (sum, count, max, min) = history.iter().fold(
        (0.0f32, 0.0f32, 0.0f32, 100.0f32),
        |(sum, count, max_v, min_v), &v| {
            (sum + v, count + 1.0, max_v.max(v), min_v.min(v))
        }
    );
    let avg = if count > 0.0 { sum / count } else { 0.0 };

    line_buf.clear();
    line_buf.push_str("  Avg: ");
    line_buf.push_str(&format!("{:.1}%", avg));
    line_buf.push_str("  │  Min: ");
    line_buf.push_str(&format!("{:.1}%", min));
    line_buf.push_str("  │  Max: ");
    line_buf.push_str(&format!("{:.1}%", max));
    line_buf.push_str("  │  Samples: ");
    line_buf.push_str(&history_len.to_string());
    lines.push(
        Line::from(Span::raw(line_buf)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .title(
            Span::raw(" CPU Flame Chart ")
                .style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}

/// Render a per-core flame chart showing each CPU core's usage as a separate "flame"
pub fn render_core_flame_chart(app: &App, frame: &mut Frame, area: Rect) {
    if app.cpu_history.is_empty() {
        return;
    }

    let history = &app.cpu_history;
    let core_count = history[0].len();
    let width = area.width as usize;
    let height = area.height as usize;

    if width < 10 || height < 3 {
        return;
    }

    let chart_height = height.saturating_sub(4);
    if chart_height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(height);

    // Time label
    let time_label = format!("  {}s ago{}       now", history.len(), " ".repeat(width.saturating_sub(20)));
    lines.push(
        Line::from(Span::raw(time_label)
            .style(Style::default().fg(Color::Gray))),
    );

    // For each row (from top = 100% to bottom = 0%)
    for row in (0..chart_height).rev() {
        let threshold = (row as f32 / chart_height as f32) * 100.0;
        let mut line_str = String::new();

        // For each column (time sample)
        for col in 0..width {
            let idx = (col as f64 / width as f64 * history.len() as f64) as usize;
            let idx = idx.min(history.len() - 1);

            // Sum up all cores' usage at this time point
            let total_usage: f32 = history[idx].iter().sum();
            let avg_usage = total_usage / core_count as f32;

            let cell_threshold = threshold + (100.0 / chart_height as f32);
            if avg_usage >= cell_threshold {
                // Use different characters based on how many cores are active
                let active_cores = history[idx].iter().filter(|&&v| v > cell_threshold).count();
                let ratio = active_cores as f32 / core_count as f32;

                let ch = if ratio > 0.8 {
                    '█'
                } else if ratio > 0.6 {
                    '▓'
                } else if ratio > 0.4 {
                    '▒'
                } else if ratio > 0.2 {
                    '░'
                } else {
                    '·'
                };
                line_str.push(ch);
            } else {
                line_str.push(' ');
            }
        }

        let pct_label = format!("{:>3}% │", threshold as i32);
        let full_line = format!("{}{}", pct_label, line_str);

        let row_color = if threshold > 80.0 {
            Color::Red
        } else if threshold > 50.0 {
            Color::Yellow
        } else if threshold > 20.0 {
            Color::Green
        } else {
            Color::DarkGray
        };

        lines.push(Line::from(Span::raw(full_line).style(Style::default().fg(row_color))));
    }

    // Bottom axis
    let axis_line = "     │".to_string() + &"─".repeat(width);
    lines.push(
        Line::from(Span::raw(axis_line)
            .style(Style::default().fg(Color::Gray))),
    );

    // Core legend
    let mut legend = "  Cores: ".to_string();
    for i in 0..std::cmp::min(core_count, 8) {
        let color_name = match i {
            0 => "C1",
            1 => "C2",
            2 => "C3",
            3 => "C4",
            4 => "C5",
            5 => "C6",
            6 => "C7",
            _ => "C8",
        };
        legend.push_str(&format!("{}{} ", color_name, if i < core_count - 1 { "," } else { "" }));
    }
    if core_count > 8 {
        legend.push_str(&format!(" +{} more", core_count - 8));
    }
    lines.push(
        Line::from(Span::raw(legend)
            .style(Style::default().fg(Color::Cyan))),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .title(
            Span::raw(format!(" Multi-Core Flame Chart ({} cores) ", core_count))
                .style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}
