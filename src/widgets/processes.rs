use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;
use sysinfo::ProcessStatus;

use crate::app::{App, SortColumn, SortDirection};

/// Format bytes to human-readable string.
/// Uses a small inline buffer to avoid heap allocation for common cases.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Get color for process state
fn process_state_color(state: ProcessStatus) -> Color {
    match state {
        ProcessStatus::Run => Color::Green,
        ProcessStatus::Sleep => Color::Blue,
        ProcessStatus::Idle => Color::Gray,
        ProcessStatus::Zombie => Color::Red,
        ProcessStatus::Stop => Color::Yellow,
        ProcessStatus::Dead => Color::Red,
        ProcessStatus::Wakekill => Color::Magenta,
        ProcessStatus::Waking => Color::Cyan,
        _ => Color::White,
    }
}

/// Get string representation of process state
fn process_state_string(state: ProcessStatus) -> &'static str {
    match state {
        ProcessStatus::Run => "R",
        ProcessStatus::Sleep => "S",
        ProcessStatus::Idle => "I",
        ProcessStatus::Zombie => "Z",
        ProcessStatus::Stop => "T",
        ProcessStatus::Dead => "D",
        ProcessStatus::Wakekill => "W",
        ProcessStatus::Waking => "W",
        _ => "?",
    }
}

/// Render the process table
pub fn render_processes(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let processes = app.get_processes();
    let process_count = processes.len();

    // Pre-allocate rows vector with known capacity
    let mut rows = Vec::with_capacity(process_count);

    // Reusable styles (avoid creating them per-row)
    let pid_style = Style::default().fg(Color::Gray);
    let name_style = Style::default().fg(Color::White);
    let mem_style = Style::default().fg(Color::Cyan);
    let state_bold = Modifier::BOLD;

    for process in &processes {
        let pid = process.pid().as_u32();
        let cpu_usage = process.cpu_usage();
        let memory = process.memory();
        let state = process.status();
        let state_char = process_state_string(state);
        let state_color = process_state_color(state);

        // Color the CPU usage based on load
        let cpu_color = if cpu_usage > 80.0 {
            Color::Red
        } else if cpu_usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        // Format PID into buffer
        let pid_str = format!("{pid:>7}");

        // Format name, truncating to 20 chars
        let name = process.name().to_string_lossy();
        let name_truncated: String = name.chars().take(20).collect();
        let name_padded = format!("{name_truncated:<20}");

        // Format CPU usage
        let cpu_str = format!("{cpu_usage:>6.1}%");

        // Format memory
        let mem_str = format!("{:>8}", format_bytes(memory));

        rows.push(Row::new(vec![
            Cell::from(Span::styled(pid_str, pid_style)),
            Cell::from(Span::styled(
                state_char.to_string(),
                Style::default().fg(state_color).add_modifier(state_bold),
            )),
            Cell::from(Span::styled(name_padded, name_style)),
            Cell::from(Span::styled(cpu_str, Style::default().fg(cpu_color))),
            Cell::from(Span::styled(mem_str, mem_style)),
        ]));
    }

    // Column headers with sort indicators (cached static strings where possible)
    let sort_indicator_pid = if app.sort_column == SortColumn::Pid {
        if app.sort_direction == SortDirection::Asc { "PID ▲" } else { "PID ▼" }
    } else { "PID" };

    let sort_indicator_s = if app.sort_column == SortColumn::Name {
        if app.sort_direction == SortDirection::Asc { "S ▲" } else { "S ▼" }
    } else { "S" };

    let sort_indicator_cpu = if app.sort_column == SortColumn::Cpu {
        if app.sort_direction == SortDirection::Asc { "CPU% ▲" } else { "CPU% ▼" }
    } else { "CPU%" };

    let sort_indicator_mem = if app.sort_column == SortColumn::Memory {
        if app.sort_direction == SortDirection::Asc { "MEM ▲" } else { "MEM ▼" }
    } else { "MEM" };

    let header_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD | Modifier::REVERSED);

    let header_row = Row::new(vec![
        Span::raw(sort_indicator_pid),
        Span::raw(sort_indicator_s),
        Span::raw("NAME"),
        Span::raw(sort_indicator_cpu),
        Span::raw(sort_indicator_mem),
    ]).style(header_style);

    // Build the table
    let widths = [
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(2),
        ratatui::layout::Constraint::Min(20),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(9),
    ];

    let title_str = format!(" Processes ({process_count} found) ");
    let title_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let bottom_bar = if app.search_mode {
        Span::raw(format!(" Search: {} ", app.search_query))
            .style(Style::default().fg(Color::Yellow))
    } else {
        Span::raw(" q:quit  /:search  ↑↓:navigate  1-4:sort  n:reverse ")
            .style(Style::default().fg(Color::Gray))
    };

    let table = Table::new(
        std::iter::once(header_row).chain(rows),
        widths,
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::raw(title_str).style(title_style))
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_bottom(bottom_bar)
            .title_alignment(ratatui::layout::Alignment::Center),
    )
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
    )
    .highlight_symbol("► ")
    .column_spacing(1);

    // Render with selection state
    let mut table_state = TableState::default();
    if let Some(selected) = app.process_table_state.selected() {
        table_state.select(Some(selected));
    }

    frame.render_stateful_widget(table, area, &mut table_state);
}
