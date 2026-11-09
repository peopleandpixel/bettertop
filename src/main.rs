mod app;
mod event;
mod ui;
mod widgets;

use std::io;

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use event::EventHandler;
use ui::render;

fn main() -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Run app
    let res = run_app(&mut terminal);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    res
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let events = EventHandler::new(app.config.refresh_rate_ms);

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        if let Ok(event) = events.next() {
            app.handle_event(event);
            if !app.running {
                break;
            }
        }
    }

    Ok(())
}
