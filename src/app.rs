use ratatui::style::Color;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::collections::VecDeque;
use sysinfo::{Pid, ProcessesToUpdate, Process, System};

use crate::event::AppEvent;

/// Sort column for the process table
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
}

/// View mode for the process panel
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessView {
    Table,
    Tree,
    Heatmap,
}

/// A node in the process tree
#[derive(Clone, Debug)]
pub struct ProcessTreeNode {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub children: Vec<ProcessTreeNode>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Theme colors for the UI
#[derive(Clone, Debug)]
pub struct Theme {
    #[allow(dead_code)]
    pub primary: Color,
    #[allow(dead_code)]
    pub secondary: Color,
    #[allow(dead_code)]
    pub success: Color,
    #[allow(dead_code)]
    pub warning: Color,
    #[allow(dead_code)]
    pub danger: Color,
    #[allow(dead_code)]
    pub background: Color,
    #[allow(dead_code)]
    pub surface: Color,
    #[allow(dead_code)]
    pub text: Color,
    #[allow(dead_code)]
    pub text_muted: Color,
    #[allow(dead_code)]
    pub border: Color,
    #[allow(dead_code)]
    pub cpu_color: Color,
    #[allow(dead_code)]
    pub memory_color: Color,
    #[allow(dead_code)]
    pub swap_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Reset,
            surface: Color::Reset,
            text: Color::White,
            text_muted: Color::Gray,
            border: Color::DarkGray,
            cpu_color: Color::Magenta,
            memory_color: Color::Cyan,
            swap_color: Color::Yellow,
        }
    }
}

/// Configuration for the app
#[derive(Clone, Debug)]
pub struct Config {
    pub refresh_rate_ms: u64,
    pub cpu_history_length: usize,
    #[allow(dead_code)]
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 500,
            cpu_history_length: 60,
            theme: Theme::default(),
        }
    }
}

/// The main application state
pub struct App {
    pub running: bool,
    pub config: Config,
    pub system: System,
    pub process_table_state: TableState,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub search_query: String,
    pub search_mode: bool,
    /// Current process view mode (table, tree, or heatmap)
    pub process_view: ProcessView,
    /// Per-core CPU usage history: each entry is a snapshot of [core1%, core2%, ...]
    pub cpu_history: VecDeque<Vec<f32>>,
    /// Overall CPU usage history
    pub cpu_total_history: VecDeque<f32>,
    /// Memory usage history (used GB)
    pub memory_history: VecDeque<f64>,
    /// Swap usage history (used GB)
    pub swap_history: VecDeque<f64>,
    /// Process CPU history for heatmap: maps process name to recent CPU usage samples
    pub process_cpu_history: HashMap<String, VecDeque<f32>>,
    /// Process filter: only show processes matching this user
    #[allow(dead_code)]
    pub filter_user: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let core_count = std::cmp::max(system.cpus().len(), 1);

        let history_len = Config::default().cpu_history_length;
        Self {
            running: true,
            config: Config::default(),
            system,
            process_table_state: TableState::default().with_selected(Some(0)),
            sort_column: SortColumn::Pid,
            sort_direction: SortDirection::Desc,
            search_query: String::new(),
            search_mode: false,
            process_view: ProcessView::Table,
            cpu_history: {
                let mut deque = VecDeque::with_capacity(history_len);
                for _ in 0..history_len {
                    deque.push_back(vec![0.0; core_count]);
                }
                deque
            },
            cpu_total_history: {
                let mut deque = VecDeque::with_capacity(history_len);
                for _ in 0..history_len {
                    deque.push_back(0.0);
                }
                deque
            },
            memory_history: {
                let mut deque = VecDeque::with_capacity(history_len);
                for _ in 0..history_len {
                    deque.push_back(0.0);
                }
                deque
            },
            swap_history: {
                let mut deque = VecDeque::with_capacity(history_len);
                for _ in 0..history_len {
                    deque.push_back(0.0);
                }
                deque
            },
            process_cpu_history: HashMap::new(),
            filter_user: None,
        }
    }

    /// Refresh system data and update history buffers
    pub fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        // Collect per-core CPU usage
        let core_usages: Vec<f32> = self.system.cpus().iter().map(|c| c.cpu_usage()).collect();
        let total_usage = if core_usages.is_empty() {
            0.0
        } else {
            core_usages.iter().sum::<f32>() / core_usages.len() as f32
        };

        // Update CPU history (O(1) pop_front vs O(n) remove(0))
        self.cpu_history.push_back(core_usages);
        if self.cpu_history.len() > self.config.cpu_history_length {
            self.cpu_history.pop_front();
        }

        self.cpu_total_history.push_back(total_usage);
        if self.cpu_total_history.len() > self.config.cpu_history_length {
            self.cpu_total_history.pop_front();
        }

        // Update memory history
        let mem_used_gb = self.system.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        self.memory_history.push_back(mem_used_gb);
        if self.memory_history.len() > self.config.cpu_history_length {
            self.memory_history.pop_front();
        }

        // Update swap history
        let swap_used_gb = self.system.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
        self.swap_history.push_back(swap_used_gb);
        if self.swap_history.len() > self.config.cpu_history_length {
            self.swap_history.pop_front();
        }

        // Update process CPU history for heatmap (top 30 processes by name)
        let mut cpu_by_name: HashMap<String, f32> = HashMap::new();
        for process in self.system.processes().values() {
            let name = process.name().to_string_lossy().to_string();
            let entry = cpu_by_name.entry(name).or_insert(0.0);
            *entry += process.cpu_usage();
        }
        // Keep top 30 processes by CPU
        let mut cpu_entries: Vec<_> = cpu_by_name.into_iter().collect();
        cpu_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cpu_entries.truncate(30);

        // Update history for each tracked process
        for (name, cpu) in &cpu_entries {
            let entry = self.process_cpu_history.entry(name.clone()).or_insert_with(|| {
                let mut deque = VecDeque::with_capacity(self.config.cpu_history_length);
                for _ in 0..self.config.cpu_history_length {
                    deque.push_back(0.0);
                }
                deque
            });
            entry.push_back(*cpu);
            if entry.len() > self.config.cpu_history_length {
                entry.pop_front();
            }
        }
        // Remove processes no longer in top 30
        self.process_cpu_history.retain(|name, _| {
            cpu_entries.iter().any(|(n, _)| n == name)
        });
    }

    /// Build a process tree from the current system state
    pub fn build_process_tree(&self) -> Vec<ProcessTreeNode> {
        let processes = self.system.processes();

        // Build a map of pid -> process
        let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut root_pids = Vec::new();

        for process in processes.values() {
            let pid_u32 = process.pid().as_u32();
            if let Some(parent) = process.parent() {
                let parent_u32 = parent.as_u32();
                children_map.entry(parent_u32).or_default().push(pid_u32);
            } else {
                root_pids.push(pid_u32);
            }
        }

        // Recursively build tree nodes
        let mut roots = Vec::new();
        for &root_pid in &root_pids {
            if let Some(node) = build_tree_node(root_pid, processes, &children_map) {
                roots.push(node);
            }
        }

        // Sort roots by CPU usage (descending)
        roots.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        roots
    }

    /// Get sorted and filtered process list
    pub fn get_processes(&self) -> Vec<&sysinfo::Process> {
        let mut processes: Vec<&sysinfo::Process> = self.system.processes().values().collect();

        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            let query_str = query.as_str();
            processes.retain(|p| {
                let name = p.name().to_string_lossy();
                name.as_ref().to_lowercase().contains(query_str)
                    || p.pid().as_u32().to_string().contains(query_str)
            });
        }

        // Sort processes
        match self.sort_column {
            SortColumn::Pid => processes.sort_by(|a, b| {
                if self.sort_direction == SortDirection::Asc {
                    a.pid().as_u32().cmp(&b.pid().as_u32())
                } else {
                    b.pid().as_u32().cmp(&a.pid().as_u32())
                }
            }),
            SortColumn::Name => processes.sort_by(|a, b| {
                let name_a = a.name().to_string_lossy();
                let name_b = b.name().to_string_lossy();
                if self.sort_direction == SortDirection::Asc {
                    name_a.cmp(&name_b)
                } else {
                    name_b.cmp(&name_a)
                }
            }),
            SortColumn::Cpu => processes.sort_by(|a, b| {
                if self.sort_direction == SortDirection::Asc {
                    a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    b.cpu_usage()
                        .partial_cmp(&a.cpu_usage())
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            SortColumn::Memory => processes.sort_by(|a, b| {
                if self.sort_direction == SortDirection::Asc {
                    a.memory()
                        .partial_cmp(&b.memory())
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    b.memory()
                        .partial_cmp(&a.memory())
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
        }

        processes
    }

    /// Handle application events
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key_event) => self.handle_key(key_event),
            AppEvent::Tick => self.refresh(),
            AppEvent::Resize(_, _) => {}
        }
    }

    fn handle_key(&mut self, key_event: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Search mode handling
        if self.search_mode {
            match key_event.code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    if self.search_query.is_empty() {
                        self.search_mode = false;
                    }
                }
                _ => {}
            }
            return;
        }

        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.running = false,
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
            }
            KeyCode::Char('n') => {
                // Toggle sort direction
                self.sort_direction = match self.sort_direction {
                    SortDirection::Asc => SortDirection::Desc,
                    SortDirection::Desc => SortDirection::Asc,
                };
            }
            KeyCode::Char('1') => self.sort_column = SortColumn::Pid,
            KeyCode::Char('2') => self.sort_column = SortColumn::Name,
            KeyCode::Char('3') => self.sort_column = SortColumn::Cpu,
            KeyCode::Char('4') => self.sort_column = SortColumn::Memory,
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = self.process_table_state.selected().unwrap_or(0);
                if selected > 0 {
                    self.process_table_state.select(Some(selected - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = self.process_table_state.selected().unwrap_or(0);
                let count = self.get_processes().len();
                if selected < count.saturating_sub(1) {
                    self.process_table_state.select(Some(selected + 1));
                }
            }
            KeyCode::PageUp => {
                let selected = self.process_table_state.selected().unwrap_or(0);
                let new_selected = selected.saturating_sub(20);
                self.process_table_state.select(Some(new_selected));
            }
            KeyCode::PageDown => {
                let selected = self.process_table_state.selected().unwrap_or(0);
                let count = self.get_processes().len();
                let new_selected = (selected + 20).min(count.saturating_sub(1));
                self.process_table_state.select(Some(new_selected));
            }
            KeyCode::Home => {
                self.process_table_state.select(Some(0));
            }
            KeyCode::End => {
                let count = self.get_processes().len();
                if count > 0 {
                    self.process_table_state.select(Some(count - 1));
                }
            }
            KeyCode::Tab => {
                // Cycle through process views
                self.process_view = match self.process_view {
                    ProcessView::Table => ProcessView::Tree,
                    ProcessView::Tree => ProcessView::Heatmap,
                    ProcessView::Heatmap => ProcessView::Table,
                };
            }
            _ => {}
        }
    }
}

/// Recursively build a process tree node
fn build_tree_node(
    pid: u32,
    processes: &std::collections::HashMap<Pid, Process>,
    children_map: &HashMap<u32, Vec<u32>>,
) -> Option<ProcessTreeNode> {
    let sys_pid = Pid::from_u32(pid);
    let process = processes.get(&sys_pid)?;

    let name = process.name().to_string_lossy().to_string();
    let cpu_usage = process.cpu_usage();
    let memory = process.memory();

    let mut children = Vec::new();
    if let Some(child_pids) = children_map.get(&pid) {
        for &child_pid in child_pids {
            if let Some(child_node) = build_tree_node(child_pid, processes, children_map) {
                children.push(child_node);
            }
        }
    }

    // Sort children by CPU usage (descending)
    children.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(ProcessTreeNode {
        pid,
        name,
        cpu_usage,
        memory,
        children,
    })
}
