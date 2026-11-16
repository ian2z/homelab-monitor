use crate::app::{App, NodeStatus};
use crate::utils::{format_bytes, format_rate, format_uptime};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, LineGauge, Paragraph, Sparkline},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main layout: Header, Content, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    render_header(f, chunks[0], app);
    render_content(f, chunks[1], app);
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let online_count = app
        .nodes
        .iter()
        .filter(|n| matches!(n.status, NodeStatus::Online))
        .count();
    let total_nodes = app.nodes.len();

    let header_text = vec![
        Span::styled("HOMELAB MONITOR ", Style::default().fg(Color::Cyan).bold()),
        Span::styled("(btop style) ", Style::default().fg(Color::DarkGray)),
        Span::raw("│ "),
        Span::styled(
            format!("Nodes Online: {}/{} ", online_count, total_nodes),
            if online_count == total_nodes && total_nodes > 0 {
                Style::default().fg(Color::Green).bold()
            } else if online_count > 0 {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::Red).bold()
            },
        ),
        Span::raw("│ "),
        Span::styled(
            format!("Refresh: {}ms", app.config.refresh_rate_ms),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let header_p = Paragraph::new(Line::from(header_text))
        .block(header_block)
        .alignment(Alignment::Center);

    f.render_widget(header_p, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer_text = Line::from(vec![
        Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" Quit  "),
        Span::styled(" [↑/↓ or j/k] ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" Navigate Nodes  "),
        Span::styled(" [Tab] ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" Next Node  "),
        Span::styled(" [r] ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" Refresh Config "),
    ]);

    let footer_p = Paragraph::new(footer_text).alignment(Alignment::Left);
    f.render_widget(footer_p, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // Responsive layout: side-by-side if wide enough, otherwise vertical split
    let content_chunks = if area.width >= 110 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area)
    };

    render_nodes_list(f, content_chunks[0], app);
    render_node_details(f, content_chunks[1], app);
}

fn render_nodes_list(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Homelab Nodes ")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(block, area);

    // Inner area for the cards
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if app.nodes.is_empty() {
        let msg = Paragraph::new("No nodes configured in config.toml\nAdd [[nodes]] to start monitoring.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(msg, inner_area);
        return;
    }

    // Determine card height
    let card_height = 5;
    let max_visible = (inner_area.height as usize) / card_height;
    let start_idx = if app.selected_index >= max_visible && max_visible > 0 {
        app.selected_index - max_visible + 1
    } else {
        0
    };

    let mut current_y = inner_area.y;

    for (i, node) in app.nodes.iter().enumerate().skip(start_idx) {
        if current_y + card_height as u16 > inner_area.y + inner_area.height {
            break;
        }

        let is_selected = i == app.selected_index;
        let card_rect = Rect {
            x: inner_area.x,
            y: current_y,
            width: inner_area.width,
            height: card_height as u16,
        };

        render_node_card(f, card_rect, node, is_selected);
        current_y += card_height as u16;
    }
}

fn render_node_card(f: &mut Frame, area: Rect, node: &crate::app::NodeState, is_selected: bool) {
    let border_color = if is_selected {
        Color::Cyan
    } else {
        Color::Indexed(238) // dark gray
    };

    let (status_badge, status_color) = match &node.status {
        NodeStatus::Online => ("ONLINE", Color::Green),
        NodeStatus::Connecting => ("CONNECTING", Color::Yellow),
        NodeStatus::Offline(_) => ("OFFLINE", Color::Red),
    };

    let title_line = Line::from(vec![
        Span::styled(
            if is_selected { "▶ " } else { "  " },
            Style::default().fg(Color::Cyan).bold(),
        ),
        Span::styled(&node.config.name, Style::default().bold().fg(if is_selected { Color::White } else { Color::Gray })),
        Span::raw(" "),
        Span::styled(format!("[{}]", status_badge), Style::default().fg(status_color).bold()),
    ]);

    let card_block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(if is_selected { BorderType::Thick } else { BorderType::Rounded })
        .border_style(Style::default().fg(border_color));

    let card_inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    f.render_widget(card_block, area);

    if card_inner.height < 2 {
        return;
    }

    match &node.metrics {
        Some(metrics) => {
            let row_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .split(card_inner);

            // CPU Gauge
            let cpu_ratio = (metrics.cpu_usage / 100.0).clamp(0.0, 1.0) as f64;
            let cpu_color = if cpu_ratio > 0.85 {
                Color::Red
            } else if cpu_ratio > 0.60 {
                Color::Yellow
            } else {
                Color::Green
            };

            let cpu_gauge = LineGauge::default()
                .block(Block::default())
                .filled_style(Style::default().fg(cpu_color))
                .filled_symbol("━")
                .unfilled_symbol("─")
                .label(format!("CPU: {:>4.1}%", metrics.cpu_usage))
                .ratio(cpu_ratio);
            f.render_widget(cpu_gauge, row_layout[0]);

            // RAM Gauge
            let mem_ratio = if metrics.memory_total_bytes > 0 {
                (metrics.memory_used_bytes as f64 / metrics.memory_total_bytes as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mem_gauge = LineGauge::default()
                .block(Block::default())
                .filled_style(Style::default().fg(Color::Magenta))
                .filled_symbol("━")
                .unfilled_symbol("─")
                .label(format!(
                    "RAM: {}/{} ({:.0}%)",
                    format_bytes(metrics.memory_used_bytes),
                    format_bytes(metrics.memory_total_bytes),
                    mem_ratio * 100.0
                ))
                .ratio(mem_ratio);
            f.render_widget(mem_gauge, row_layout[1]);

            // Sub info: IP and Load Average
            if row_layout.len() > 2 {
                let sub_info = Line::from(vec![
                    Span::styled(format!("URL: {} ", node.config.url), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(
                            "Load: {:.2} {:.2} {:.2}",
                            metrics.load_average[0], metrics.load_average[1], metrics.load_average[2]
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                f.render_widget(Paragraph::new(sub_info), row_layout[2]);
            }
        }
        None => {
            let status_msg = match &node.status {
                NodeStatus::Offline(err) => format!("Error: {}", err),
                NodeStatus::Connecting => "Connecting to agent...".to_string(),
                NodeStatus::Online => "Waiting for initial metrics...".to_string(),
            };
            let p = Paragraph::new(format!("URL: {}\n{}", node.config.url, status_msg))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(p, card_inner);
        }
    }
}

fn render_node_details(f: &mut Frame, area: Rect, app: &App) {
    let selected_node = app.nodes.get(app.selected_index);

    let title = match selected_node {
        Some(node) => format!(" Details: {} ", node.config.name),
        None => " Details ".to_string(),
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    f.render_widget(block, area);

    let node = match selected_node {
        Some(n) => n,
        None => return,
    };

    let metrics = match &node.metrics {
        Some(m) => m,
        None => {
            let msg = match &node.status {
                NodeStatus::Offline(err) => format!("Node is currently OFFLINE.\n\nReason: {}\n\nMake sure homelab-agent is running on: {}", err, node.config.url),
                NodeStatus::Connecting => format!("Connecting to {}...", node.config.url),
                NodeStatus::Online => "Waiting for metrics data...".to_string(),
            };
            let p = Paragraph::new(msg)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(p, inner_area);
            return;
        }
    };

    // Split details into sections:
    // 1. Host Info Bar (Height: 3)
    // 2. CPU Panel with Sparkline (Height: 6)
    // 3. Memory & Swap Panel (Height: 5)
    // 4. Disks & Network (Remaining)
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Host info
            Constraint::Length(7), // CPU & Sparkline
            Constraint::Length(5), // Memory & Swap
            Constraint::Min(6),    // Disks & Network
        ])
        .split(inner_area);

    // 1. Host Info Bar
    let host_info_line = Line::from(vec![
        Span::styled("Host: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&metrics.hostname, Style::default().bold().fg(Color::White)),
        Span::raw("  │  "),
        Span::styled("OS: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&metrics.os_name, Style::default().fg(Color::White)),
        Span::raw("  │  "),
        Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format_uptime(metrics.uptime_secs), Style::default().fg(Color::Green)),
        Span::raw("  │  "),
        Span::styled("Cores: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", metrics.cpu_cores), Style::default().fg(Color::Cyan)),
    ]);

    let host_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Indexed(238)));
    let host_p = Paragraph::new(host_info_line).block(host_block);
    f.render_widget(host_p, sections[0]);

    // 2. CPU Panel with Sparkline
    let cpu_block = Block::default()
        .title(format!(" CPU Usage: {:.1}% ({}) ", metrics.cpu_usage, metrics.cpu_name))
        .title_style(Style::default().fg(Color::Green).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let cpu_inner = Rect {
        x: sections[1].x + 1,
        y: sections[1].y + 1,
        width: sections[1].width.saturating_sub(2),
        height: sections[1].height.saturating_sub(2),
    };
    f.render_widget(cpu_block, sections[1]);

    let cpu_data: Vec<u64> = node.cpu_history.iter().copied().collect();
    let sparkline = Sparkline::default()
        .block(Block::default())
        .data(&cpu_data)
        .max(100)
        .style(Style::default().fg(Color::Green));
    f.render_widget(sparkline, cpu_inner);

    // 3. Memory & Swap Panel
    let mem_block = Block::default()
        .title(" Memory & Swap ")
        .title_style(Style::default().fg(Color::Magenta).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mem_inner = Rect {
        x: sections[2].x + 1,
        y: sections[2].y + 1,
        width: sections[2].width.saturating_sub(2),
        height: sections[2].height.saturating_sub(2),
    };
    f.render_widget(mem_block, sections[2]);

    let mem_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(mem_inner);

    // RAM Bar
    let mem_percent = if metrics.memory_total_bytes > 0 {
        ((metrics.memory_used_bytes as f64 / metrics.memory_total_bytes as f64) * 100.0).clamp(0.0, 100.0) as u16
    } else {
        0
    };
    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Indexed(236)))
        .percent(mem_percent)
        .label(format!(
            "RAM: {} / {} ({}%)",
            format_bytes(metrics.memory_used_bytes),
            format_bytes(metrics.memory_total_bytes),
            mem_percent
        ));
    f.render_widget(ram_gauge, mem_split[0]);

    // Swap Bar
    if mem_split.len() > 1 {
        let swap_percent = if metrics.swap_total_bytes > 0 {
            ((metrics.swap_used_bytes as f64 / metrics.swap_total_bytes as f64) * 100.0).clamp(0.0, 100.0) as u16
        } else {
            0
        };
        let swap_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Blue).bg(Color::Indexed(236)))
            .percent(swap_percent)
            .label(format!(
                "Swap: {} / {} ({}%)",
                format_bytes(metrics.swap_used_bytes),
                format_bytes(metrics.swap_total_bytes),
                swap_percent
            ));
        f.render_widget(swap_gauge, mem_split[1]);
    }

    // 4. Disks & Network Split
    let bottom_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[3]);

    // Disks
    let disk_block = Block::default()
        .title(" Disks ")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let disk_inner = Rect {
        x: bottom_split[0].x + 1,
        y: bottom_split[0].y + 1,
        width: bottom_split[0].width.saturating_sub(2),
        height: bottom_split[0].height.saturating_sub(2),
    };
    f.render_widget(disk_block, bottom_split[0]);

    let mut disk_lines = Vec::new();
    for d in &metrics.disks {
        let pct = if d.total_bytes > 0 {
            (d.used_bytes as f64 / d.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        disk_lines.push(Line::from(vec![
            Span::styled(format!("{:<14}", d.mount_point), Style::default().bold().fg(Color::White)),
            Span::styled(
                format!("{}/{} ({:.0}%)", format_bytes(d.used_bytes), format_bytes(d.total_bytes), pct),
                Style::default().fg(if pct > 85.0 { Color::Red } else { Color::Yellow }),
            ),
        ]));
    }
    f.render_widget(Paragraph::new(disk_lines), disk_inner);

    // Network
    let net_block = Block::default()
        .title(" Network Interfaces ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let net_inner = Rect {
        x: bottom_split[1].x + 1,
        y: bottom_split[1].y + 1,
        width: bottom_split[1].width.saturating_sub(2),
        height: bottom_split[1].height.saturating_sub(2),
    };
    f.render_widget(net_block, bottom_split[1]);

    let mut net_lines = Vec::new();
    for n in &metrics.networks {
        // Only show interfaces with traffic or active name
        if n.rx_bytes > 0 || n.tx_bytes > 0 {
            net_lines.push(Line::from(vec![
                Span::styled(format!("{:<10} ", n.interface_name), Style::default().bold().fg(Color::White)),
                Span::styled(format!("RX: {:<9} ", format_rate(n.rx_rate_bytes_sec)), Style::default().fg(Color::Green)),
                Span::styled(format!("TX: {}", format_rate(n.tx_rate_bytes_sec)), Style::default().fg(Color::Blue)),
            ]));
        }
    }
    if net_lines.is_empty() {
        net_lines.push(Line::from("No active network interfaces detected."));
    }
    f.render_widget(Paragraph::new(net_lines), net_inner);
}
