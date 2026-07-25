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
use sysinfo::System;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?; // desable terminal eco and line buffer
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // open clear slip screen
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new();

    // TUI loop
    loop {
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        //CPU global use, in %
        let cpu_usage = sys.global_cpu_usage();
        
        //RAM in MB
        let used_mem = sys.used_memory() / 1024 / 1024;
        let total_mem = sys.total_memory() / 1024 /1024;

        terminal.draw(|f| {
            let area = f.area(); // terminal window size

            // border and header
            let block = Block::default()
            .title( " Homelab Monitor - Press 'q' to exit ")
            .borders(Borders::ALL);

            let metrics_text = format!(
                "\n CPU use: {:.1}%\n\n RAM use: {} MB / {} MB",
                cpu_usage, used_mem, total_mem
            );

            let paragraph = Paragraph::new(metrics_text).block(block);

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