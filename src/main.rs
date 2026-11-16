use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use homelab_monitor::{
    app::App,
    models::{AppConfig, NodeMetrics},
    ui,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, fs, io, path::Path, time::Duration};
use tokio::sync::mpsc;

fn load_or_create_config() -> AppConfig {
    let config_path = Path::new("config.toml");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }

    let default_config = AppConfig::default();
    if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
        let _ = fs::write(config_path, toml_str);
    }
    default_config
}

async fn fetch_node_metrics(
    client: reqwest::Client,
    node_idx: usize,
    url: String,
    tx: mpsc::Sender<(usize, Result<NodeMetrics, String>)>,
) {
    let metrics_url = if url.ends_with("/metrics") {
        url
    } else {
        format!("{}/metrics", url.trim_end_matches('/'))
    };

    let result = match client.get(&metrics_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<NodeMetrics>().await {
                    Ok(m) => Ok(m),
                    Err(e) => Err(format!("Parse error: {}", e)),
                }
            } else {
                Err(format!("HTTP status: {}", resp.status()))
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err("Connection timeout (2s)".to_string())
            } else if e.is_connect() {
                Err("Connection refused".to_string())
            } else {
                Err(format!("Network error: {}", e))
            }
        }
    };

    let _ = tx.send((node_idx, result)).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_or_create_config();
    let refresh_ms = config.refresh_rate_ms;
    let mut app = App::new(config);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Channel for worker results
    let (tx, mut rx) = mpsc::channel::<(usize, Result<NodeMetrics, String>)>(100);

    // HTTP client with 2s timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;

    // Background poller
    let poller_client = client.clone();
    let nodes_info: Vec<(usize, String)> = app
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (i, n.config.url.clone()))
        .collect();

    let poller_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            for (idx, url) in &nodes_info {
                let client = poller_client.clone();
                let tx = poller_tx.clone();
                let url = url.clone();
                let idx = *idx;
                tokio::spawn(fetch_node_metrics(client, idx, url, tx));
            }
            tokio::time::sleep(Duration::from_millis(refresh_ms)).await;
        }
    });

    // UI event loop
    loop {
        // Drain any pending metric updates
        while let Ok((node_idx, result)) = rx.try_recv() {
            if let Some(node) = app.nodes.get_mut(node_idx) {
                node.update_metrics(result);
            }
        }

        // Draw UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll for input events (50ms responsive tick)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                            app.next_node();
                        }
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                            app.prev_node();
                        }
                        KeyCode::Char('r') => {
                            // Reload configuration
                            let cfg = load_or_create_config();
                            app = App::new(cfg);
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
