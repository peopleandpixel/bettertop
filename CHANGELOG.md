# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of bettertop
- Donut CPU gauge with pie emoji (🟩🟨🟧🟥) and color gradient
- Per-core CPU usage bars with color-coded fill levels
- CPU flame chart visualization with intensity-based shading (█▓▒░·)
- Process heatmap (GitHub-style contribution graph) for top 30 processes
- Hierarchical process tree view with tree-line symbols (├── └── │)
- Memory and swap usage gauges with sparkline history
- Sortable process table with color-coded states (R/S/Z/T)
- Process search by name or PID
- Three switchable process views: Table, Tree, Heatmap (Tab key)
- Vim-style navigation (j/k, arrow keys, PageUp/PageDown, Home/End)
- Multi-column sorting (PID, Name, CPU%, Memory) with direction toggle
- Beautiful rounded borders and color-coded headers
- 500ms refresh rate with 60-sample history buffers

### Performance
- VecDeque for O(1) history buffer management (vs O(n) with Vec)
- Pre-allocated vectors and reusable buffers in all render paths
- Static strings for frequently allocated format strings
- O(1) process count in header (avoids full sort/filter)
- Reduced heap allocations from ~500+ per frame to ~50+ on typical systems

### Tech Stack
- ratatui 0.29 — TUI framework
- sysinfo 0.33 — Cross-platform system information
- crossterm 0.28 — Terminal I/O
- anyhow 1 — Error handling
