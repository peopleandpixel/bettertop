use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, ProcessTreeNode};

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Get color for CPU usage
fn cpu_color(usage: f32) -> Color {
    if usage > 80.0 {
        Color::Red
    } else if usage > 50.0 {
        Color::Yellow
    } else if usage > 10.0 {
        Color::Green
    } else {
        Color::DarkGray
    }
}

/// Render a single process tree node and return the lines.
/// Uses mutable buffers to avoid string allocations in recursion.
fn render_tree_node(
    node: &ProcessTreeNode,
    prefix_buf: &mut String,
    is_last: bool,
    max_depth: usize,
    current_depth: usize,
    lines: &mut Vec<Line>,
    line_buf: &mut String,
) {
    // Tree connector symbols — build in line_buf to avoid allocation
    line_buf.clear();
    if current_depth > 0 {
        line_buf.push_str(prefix_buf);
        if is_last {
            line_buf.push_str("└── ");
        } else {
            line_buf.push_str("├── ");
        }
    }

    // Build the display line for this node
    let cpu = node.cpu_usage;
    let mem = format_bytes(node.memory);

    // Indent based on depth with color coding
    let depth_color = match current_depth {
        0 => Color::White,
        1 => Color::Cyan,
        2 => Color::Magenta,
        3 => Color::Yellow,
        4 => Color::Green,
        _ => Color::Gray,
    };

    // Truncate name to 30 chars
    let name_truncated: String = node.name.chars().take(30).collect();
    line_buf.push_str(&format!("{name_truncated:<30} {cpu:>6.1}%  {mem:>8}"));

    lines.push(Line::from(Span::raw(line_buf.clone()).style(Style::default().fg(depth_color))));

    // Recurse into children (limit depth to avoid overwhelming the view)
    if current_depth < max_depth && !node.children.is_empty() {
        // Save current prefix length for restoration
        let saved_len = prefix_buf.len();

        // Extend prefix for children
        if current_depth == 0 {
            // Root's children: no prefix yet, just add the connector spacing
            if is_last {
                prefix_buf.push_str("    ");
            } else {
                prefix_buf.push_str("│   ");
            }
        } else {
            // Already has prefix, just extend it
            if is_last {
                prefix_buf.push_str("    ");
            } else {
                prefix_buf.push_str("│   ");
            }
        }

        let children_len = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == children_len - 1;
            render_tree_node(
                child,
                prefix_buf,
                is_last_child,
                max_depth,
                current_depth + 1,
                lines,
                line_buf,
            );
        }

        // Restore prefix
        prefix_buf.truncate(saved_len);
    }
}

/// Render the process tree view
pub fn render_process_tree(app: &App, frame: &mut Frame, area: Rect) {
    let tree_roots = app.build_process_tree();

    // Calculate max depth based on available height
    let max_depth = std::cmp::min(5, (area.height as usize).saturating_sub(4));

    // Pre-allocate lines vector (header + estimated nodes)
    let mut lines: Vec<Line> = Vec::with_capacity(100);

    // Reusable buffers for tree rendering
    let mut prefix_buf = String::with_capacity(64);
    let mut line_buf = String::with_capacity(80);

    // Add header
    lines.push(
        Line::from(Span::raw(" PROCESS TREE (Tab:switch view) ")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    );
    lines.push(Line::default());

    // Render each root node
    let roots_len = tree_roots.len();
    for (i, root) in tree_roots.iter().enumerate() {
        let is_last = i == roots_len - 1;
        render_tree_node(root, &mut prefix_buf, is_last, max_depth, 0, &mut lines, &mut line_buf);
    }

    if lines.len() <= 2 {
        lines.push(
            Line::from(Span::raw("No processes found")
                .style(Style::default().fg(Color::Gray))),
        );
    }

    // Count total processes in tree
    let total_processes = count_tree_processes(&tree_roots);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green))
        .title(
            Span::raw(format!(" Process Tree ({total_processes} processes) "))
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .title_alignment(ratatui::layout::Alignment::Center)
        .title_bottom(
            Span::raw(format!(" Depth: {max_depth}  │  Roots: {roots_len} "))
                .style(Style::default().fg(Color::Gray)),
        )
        .title_alignment(ratatui::layout::Alignment::Center);

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block).wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Count total processes in a tree
fn count_tree_processes(roots: &[ProcessTreeNode]) -> usize {
    let mut count = roots.len();
    for root in roots {
        count += count_tree_processes(&root.children);
    }
    count
}
