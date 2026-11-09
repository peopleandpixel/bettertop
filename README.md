# bettertop

A beautiful, feature-rich system monitor for the terminal, written in Rust.

bettertop goes beyond traditional htop clones with stunning visualizations including a donut CPU gauge, flame charts, process heatmaps, and hierarchical process trees — all rendered with smooth animations and color-coded metrics.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-GPL--3.0-or--later-blue)

## Features

- **🍩 Donut CPU Gauge** — Eye-catching pie-emoji donut chart showing total CPU usage with a green→yellow→orange→red color gradient
- **📊 CPU Flame Chart** — Horizontal histogram visualization of CPU load over time with intensity-based shading (█▓▒░·)
- **🔥 Process Heatmap** — GitHub-style contribution graph showing top 30 processes' CPU usage over time
- **🌳 Process Tree View** — Hierarchical parent-child process display with tree-line symbols and depth-based coloring
- **💾 Memory & Swap Bars** — Color-coded horizontal gauges with sparkline history graphs
- **📋 Sortable Process Table** — Full process list with color-coded states (running/sleeping/zombie), CPU usage coloring, and multi-column sorting
- **🔍 Search** — Filter processes by name or PID
- **⌨️ Vim-style Navigation** — `j`/`k` keys, arrow keys, PageUp/PageDown, Home/End

## Screenshots

### CPU Gauge & Flame Chart
The top panel shows a donut CPU gauge with per-core bars and a flame chart visualization.

### Process Views
Switch between three views with `Tab`:
- **Table** — Sortable process list
- **Tree** — Hierarchical process tree
- **Heatmap** — Process CPU usage heatmap

## Installation

### From Source

Prerequisites: [Rust](https://rustup.rs/) (stable toolchain)

```bash
git clone https://github.com/peopleandpixel/bettertop.git
cd bettertop
cargo build --release
./target/release/bettertop
```

### One-liner

```bash
git clone https://github.com/peopleandpixel/bettertop.git && cd bettertop && cargo build --release && ./target/release/bettertop
```

## Usage

Run `bettertop` in your terminal:

```bash
./target/release/bettertop
```

### Key Bindings

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `/` | Start search mode |
| `Esc` | Exit search mode |
| `Backspace` | Delete character in search |
| `Tab` | Switch view (Table → Tree → Heatmap) |
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `PageUp` | Page up (20 rows) |
| `PageDown` | Page down (20 rows) |
| `Home` | Jump to first process |
| `End` | Jump to last process |
| `1` | Sort by PID |
| `2` | Sort by Name |
| `3` | Sort by CPU% |
| `4` | Sort by Memory |
| `n` | Toggle sort direction (ascending/descending) |

## Architecture

```
src/
├── main.rs           # Entry point, terminal init/restore
├── app.rs            # App state, system refresh, event handling
├── ui.rs             # Main layout and rendering
├── event.rs          # Keyboard and terminal event handling
└── widgets/
    ├── mod.rs        # Widget module exports
    ├── cpu.rs        # CPU flame chart widget
    ├── gauge.rs      # Donut CPU gauge and per-core bars
    ├── memory.rs     # Memory & swap gauges with sparklines
    ├── processes.rs  # Sortable process table
    ├── heatmap.rs    # Process CPU usage heatmap
    └── tree.rs       # Hierarchical process tree view
```

## Tech Stack

- **[ratatui](https://github.com/ratatui/ratatui)** — TUI framework for rich terminal widgets
- **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** — Cross-platform system information
- **[crossterm](https://github.com/crossterm-rs/crossterm)** — Terminal I/O and event handling
- **[anyhow](https://github.com/dtolnay/anyhow)** — Ergonomic error handling

## Performance

bettertop is optimized for speed and low memory usage:

- `VecDeque` for O(1) history buffer management
- Pre-allocated vectors and reusable buffers in render paths
- Static strings for frequently allocated format strings
- O(1) process count in header (avoids full sort/filter)

## License

Licensed under the GNU General Public License v3.0 or later ([LICENSE](LICENSE)).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [bottom](https://github.com/ClementTsang/bottom) — Inspiration for beautiful TUI system monitoring
- [ratatui](https://github.com/ratatui/ratatui) — Excellent TUI framework
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — Reliable cross-platform system metrics
