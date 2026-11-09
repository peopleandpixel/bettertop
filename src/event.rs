use std::io;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};

/// Application events
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(#[allow(dead_code)] u16, #[allow(dead_code)] u16),
}

/// Event handler with configurable tick rate
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate),
        }
    }

    pub fn next(&self) -> io::Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                    Ok(AppEvent::Key(key))
                }
                CrosstermEvent::Resize(width, height) => Ok(AppEvent::Resize(width, height)),
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
