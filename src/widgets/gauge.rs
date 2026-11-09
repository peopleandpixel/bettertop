use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render a beautiful speedometer-style gauge for total CPU usage.
/// Uses a semi-circular arc with a needle indicator.
pub fn render_circular_gauge(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: f32,
    max_value: f32,
    color: Color,
) {
    let ratio = if max_value > 0.0 {
        (value / max_value).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let gauge_lines = build_speedometer(ratio, label, color);
    let paragraph = Paragraph::new(gauge_lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Build a donut-style gauge using pie characters arranged in a circle.
/// Uses colored circle emoji (🟩🟨🟧🟥) for filled segments and dim circles for empty.
/// The ring fills clockwise based on the ratio.
///
/// Each pie emoji is double-width (2 terminal columns). Terminal cells are roughly
/// 2× taller than wide, so to appear visually round, the gauge width in columns
/// should be approximately 2× the height in rows.
///
/// Layout: 7 rows of content, 22 columns wide → visually ~22×14 (accounts for char aspect ratio).
///
///        🟩🟩🟩             (row 0: top arc)
///      🟩         🟩        (row 1: upper sides)
///    🟩             🟩      (row 2: mid-upper sides)
///    🟩   50%     🟩        (row 3: middle with percentage)
///    🟩             🟩      (row 4: mid-lower sides)
///      🟩         🟩        (row 5: lower sides)
///        🟩🟩🟩             (row 6: bottom arc)
///        TOTAL CPU         (row 7: label)
///
/// Uses 16 cells for smooth clockwise fill.
fn build_speedometer(ratio: f32, label: &str, _color: Color) -> Vec<Line<'static>> {
    let pct = (ratio * 100.0) as i32;

    // 16 cells in the ring, ordered clockwise from top-left
    let filled_cells = (ratio * 16.0) as usize;

    // Helper: get pie char and color for a ring position.
    // Uses static strings to avoid heap allocation.
    let pie = |pos: usize| -> (&'static str, Color) {
        let is_filled = pos < filled_cells;
        if !is_filled {
            return ("⚫", Color::DarkGray);
        }
        let fraction = pos as f32 / 16.0;
        if fraction < 0.25 {
            ("🟩", Color::Green)
        } else if fraction < 0.5 {
            ("🟨", Color::Yellow)
        } else if fraction < 0.75 {
            ("🟧", Color::LightYellow)
        } else {
            ("🟥", Color::Red)
        }
    };

    // 16 cells arranged clockwise around the ring:
    // Row 0:       0  1  2              (top arc, 3 cells)
    // Row 1:  15              3         (upper sides, 2 cells)
    // Row 2: 14                4        (mid-upper sides, 2 cells)
    // Row 3: 13      %%      5          (middle, 2 cells)
    // Row 4: 12                6        (mid-lower sides, 2 cells)
    // Row 5: 11              7          (lower sides, 2 cells)
    // Row 6:       8  9 10             (bottom arc, 3 cells)
    // Total: 3+2+2+2+2+2+3 = 16 cells

    let c0 = pie(0); let c1 = pie(1); let c2 = pie(2);
    let c3 = pie(3);
    let c4 = pie(4);
    let c5 = pie(5);
    let c6 = pie(6);
    let c7 = pie(7);
    let c8 = pie(8); let c9 = pie(9); let c10 = pie(10);
    let c11 = pie(11);
    let c12 = pie(12);
    let c13 = pie(13);
    let c14 = pie(14);
    let c15 = pie(15);

    // Pre-allocate with capacity for 8 lines
    let mut lines = Vec::with_capacity(8);

    // Each emoji occupies 2 columns (double-width).
    // All rows target 22 columns total width for a consistent oval.

    // Row 0: top arc
    // 8 spaces + 3×emoji(6 cols) + 8 spaces = 22 ✓
    lines.push(Line::from(vec![
        Span::raw("        "),
        Span::styled(c0.0, Style::default().fg(c0.1)),
        Span::styled(c1.0, Style::default().fg(c1.1)),
        Span::styled(c2.0, Style::default().fg(c2.1)),
        Span::raw("        "),
    ]));

    // Row 1: upper sides
    // 6 spaces + emoji(2) + 6 spaces + emoji(2) + 6 spaces = 22 ✓
    lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(c15.0, Style::default().fg(c15.1)),
        Span::raw("      "),
        Span::styled(c3.0, Style::default().fg(c3.1)),
        Span::raw("      "),
    ]));

    // Row 2: mid-upper sides
    // 4 spaces + emoji(2) + 10 spaces + emoji(2) + 4 spaces = 22 ✓
    lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(c14.0, Style::default().fg(c14.1)),
        Span::raw("          "),
        Span::styled(c4.0, Style::default().fg(c4.1)),
        Span::raw("    "),
    ]));

    // Row 3: middle with percentage
    // 4 spaces + emoji(2) + 1 space + " XX%"(4) + 5 spaces + emoji(2) + 4 spaces = 22 ✓
    lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(c13.0, Style::default().fg(c13.1)),
        Span::raw(" "),
        Span::styled(format!("{:>3}%", pct), Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)),
        Span::raw("     "),
        Span::styled(c5.0, Style::default().fg(c5.1)),
        Span::raw("    "),
    ]));

    // Row 4: mid-lower sides — same layout as row 2
    lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(c12.0, Style::default().fg(c12.1)),
        Span::raw("          "),
        Span::styled(c6.0, Style::default().fg(c6.1)),
        Span::raw("    "),
    ]));

    // Row 5: lower sides — same layout as row 1
    lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(c11.0, Style::default().fg(c11.1)),
        Span::raw("      "),
        Span::styled(c7.0, Style::default().fg(c7.1)),
        Span::raw("      "),
    ]));

    // Row 6: bottom arc (reversed order for clockwise fill) — same layout as row 0
    lines.push(Line::from(vec![
        Span::raw("        "),
        Span::styled(c10.0, Style::default().fg(c10.1)),
        Span::styled(c9.0, Style::default().fg(c9.1)),
        Span::styled(c8.0, Style::default().fg(c8.1)),
        Span::raw("        "),
    ]));

    // Row 7: label
    lines.push(Line::from(Span::styled(
        format!("        {label}        "),
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::DIM),
    )));

    lines
}

/// Render CPU gauges: per-core mini bars on the left, large speedometer on the right
pub fn render_circular_cpu_gauges(app: &App, frame: &mut Frame, area: Rect) {
    let cores = app.system.cpus();
    let core_count = cores.len();

    if core_count == 0 || area.width < 20 || area.height < 3 {
        return;
    }

    // Split area: left side for per-core bars, right side for total speedometer
    let chunks = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]).split(area);

    // --- Left: per-core bar gauges ---
    render_core_bars(app, frame, chunks[0]);

    // --- Right: total CPU speedometer ---
    let total_usage = if core_count > 0 {
        cores.iter().map(|c| c.cpu_usage()).sum::<f32>() / core_count as f32
    } else {
        0.0
    };

    let total_color = if total_usage < 30.0 {
        Color::Green
    } else if total_usage < 70.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    render_circular_gauge(frame, chunks[1], "TOTAL CPU", total_usage, 100.0, total_color);
}

/// Render compact per-core bar gauges
fn render_core_bars(app: &App, frame: &mut Frame, area: Rect) {
    let cores = app.system.cpus();
    let core_count = cores.len();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .title(
            Span::raw(format!(" CPU Cores (/{core_count}) "))
                .style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(block, area);
    let inner = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });

    if inner.height < 1 || inner.width < 10 {
        return;
    }

    // Calculate bar width per core row
    let max_cores = inner.height as usize;
    let display_cores = std::cmp::min(core_count, max_cores);

    // Label width + bar width + percentage width
    let label_width = 6;  // "C  0 "
    let pct_width = 6;    // " 12.3%"
    let bar_width = (inner.width as usize).saturating_sub(label_width + pct_width + 2);
    if bar_width < 2 {
        return;
    }

    // Reusable buffer for bar characters
    let mut bar_buf = String::with_capacity(bar_width);

    // Reusable styles
    let label_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
    let pct_bold = Modifier::BOLD;

    for row in 0..display_cores {
        let core_idx = row;
        let usage = cores[core_idx].cpu_usage().min(100.0);

        let filled = (usage / 100.0 * bar_width as f32) as usize;
        let empty = bar_width - filled;

        let color = if usage < 30.0 {
            Color::Green
        } else if usage < 70.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Build bar string in reusable buffer
        bar_buf.clear();
        for _ in 0..filled {
            bar_buf.push('█');
        }
        for _ in 0..empty {
            bar_buf.push('·');
        }

        let line = Line::from(vec![
            Span::styled(format!("C{core_idx:>2} "), label_style),
            Span::styled(bar_buf.clone(), Style::default().fg(color)),
            Span::styled(format!("{usage:>5.1}%"), Style::default().fg(color).add_modifier(pct_bold)),
        ]);

        let paragraph = Paragraph::new(line);
        let line_area = Rect {
            x: inner.x,
            y: inner.y + row as u16,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(paragraph, line_area);
    }
}
