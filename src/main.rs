use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{error::Error, io, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?; // desable terminal eco and line buffer
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // open clear slip screen
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TUI loop
    loop {
        terminal.draw(|f| {
            let area = f.area(); // terminal window size

            // border and header
            let block = Block::default()
            .title( " Homelab Monitor - Press 'q' to exit ")
            .borders(Borders::ALL);

            let paragraph = Paragraph::new("\n waiting new system metrics...").block(block);

            // render widget
            f.render_widget(paragraph, area);
        })?;

        // events: 100ms checker
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break; // loop breaker, exit to terminal
                }
            }
        }
    }
    // TEARDOWN
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}