use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;

pub type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init() -> anyhow::Result<Tui> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    Ok(Terminal::new(backend)?)
}

pub fn restore() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
