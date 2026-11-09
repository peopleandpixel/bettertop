use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, ProcessView};
use crate::widgets::{flame_chart, gauge, heatmap, memory, processes, tree};

/// Render the main UI layout
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: split into header, metrics row, and bottom panel
    let main_chunks: Vec<_> = Layout::vertical([
        Constraint::Length(3),    // Header
        Constraint::Min(5),       // CPU gauges + memory row
        Constraint::Min(10),      // Bottom panel (chart + processes)
    ])
    .split(area)
    .iter()
    .copied()
    .collect();

    let header_area = main_chunks[0];
    let top_area = main_chunks[1];
    let bottom_area = main_chunks[2];

    // Header
    render_header(frame, header_area, app);

    // Top section: CPU gauges and memory side by side
    let top_chunks: Vec<_> = Layout::horizontal([
        Constraint::Percentage(40), // CPU gauges
        Constraint::Percentage(60), // Memory
    ])
    .split(top_area)
    .iter()
    .copied()
    .collect();

    // Use circular gauge widget instead of the old CPU gauges
    gauge::render_circular_cpu_gauges(app, frame, top_chunks[0]);
    memory::render_memory(app, frame, top_chunks[1]);

    // Bottom section: flame chart and process view
    let bottom_chunks: Vec<_> = Layout::vertical([
        Constraint::Percentage(40), // Flame chart
        Constraint::Percentage(60), // Process view (table/tree/heatmap)
    ])
    .split(bottom_area)
    .iter()
    .copied()
    .collect();

    // Use flame chart instead of the line chart
    flame_chart::render_flame_chart(app, frame, bottom_chunks[0]);

    // Render the appropriate process view
    match app.process_view {
        ProcessView::Table => {
            processes::render_processes(app, frame, bottom_chunks[1]);
        }
        ProcessView::Tree => {
            tree::render_process_tree(app, frame, bottom_chunks[1]);
        }
        ProcessView::Heatmap => {
            heatmap::render_detailed_heatmap(app, frame, bottom_chunks[1]);
        }
    }
}

/// Render the header bar
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let view_name = match app.process_view {
        ProcessView::Table => "Table",
        ProcessView::Tree => "Tree",
        ProcessView::Heatmap => "Heatmap",
    };

    // Use raw process count instead of calling get_processes() which sorts and filters
    let process_count = app.system.processes().len();
    let cores = app.system.cpus();
    let core_count = cores.len();
    let total_cpu = if core_count > 0 {
        cores.iter().map(|c| c.cpu_usage()).sum::<f32>() / core_count as f32
    } else {
        0.0
    };

    let header_text = format!(
        "  bettertop v0.2.0  │  CPU: {total_cpu:.1}%  │  Cores: {core_count}  │  Processes: {process_count}  │  View: {view_name}  │  Tab:switch  q:quit  /:search  "
    );

    let header = Paragraph::new(Span::raw(header_text))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        )
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(header, area);
}

