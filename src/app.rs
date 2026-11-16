use crate::models::{AppConfig, HostConfig, NodeMetrics};
use std::collections::VecDeque;

pub const HISTORY_LEN: usize = 60;

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Connecting,
    Online,
    Offline(String),
}

#[derive(Debug, Clone)]
pub struct NodeState {
    pub config: HostConfig,
    pub status: NodeStatus,
    pub metrics: Option<NodeMetrics>,
    pub cpu_history: VecDeque<u64>,
    pub mem_history: VecDeque<u64>,
    pub rx_rate_history: VecDeque<u64>,
    pub tx_rate_history: VecDeque<u64>,
}

impl NodeState {
    pub fn new(config: HostConfig) -> Self {
        Self {
            config,
            status: NodeStatus::Connecting,
            metrics: None,
            cpu_history: VecDeque::with_capacity(HISTORY_LEN),
            mem_history: VecDeque::with_capacity(HISTORY_LEN),
            rx_rate_history: VecDeque::with_capacity(HISTORY_LEN),
            tx_rate_history: VecDeque::with_capacity(HISTORY_LEN),
        }
    }

    pub fn update_metrics(&mut self, result: Result<NodeMetrics, String>) {
        match result {
            Ok(metrics) => {
                self.status = NodeStatus::Online;

                if self.cpu_history.len() >= HISTORY_LEN {
                    self.cpu_history.pop_front();
                }
                self.cpu_history.push_back(metrics.cpu_usage.round() as u64);

                if self.mem_history.len() >= HISTORY_LEN {
                    self.mem_history.pop_front();
                }
                let mem_percent = if metrics.memory_total_bytes > 0 {
                    (metrics.memory_used_bytes * 100 / metrics.memory_total_bytes) as u64
                } else {
                    0
                };
                self.mem_history.push_back(mem_percent);

                let total_rx_rate: u64 = metrics.networks.iter().map(|n| n.rx_rate_bytes_sec).sum();
                let total_tx_rate: u64 = metrics.networks.iter().map(|n| n.tx_rate_bytes_sec).sum();

                if self.rx_rate_history.len() >= HISTORY_LEN {
                    self.rx_rate_history.pop_front();
                }
                self.rx_rate_history.push_back(total_rx_rate);

                if self.tx_rate_history.len() >= HISTORY_LEN {
                    self.tx_rate_history.pop_front();
                }
                self.tx_rate_history.push_back(total_tx_rate);

                self.metrics = Some(metrics);
            }
            Err(err) => {
                self.status = NodeStatus::Offline(err);
            }
        }
    }
}

pub struct App {
    pub config: AppConfig,
    pub nodes: Vec<NodeState>,
    pub selected_index: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let nodes = config
            .nodes
            .iter()
            .cloned()
            .map(NodeState::new)
            .collect();

        Self {
            config,
            nodes,
            selected_index: 0,
            should_quit: false,
        }
    }

    pub fn next_node(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.nodes.len();
        }
    }

    pub fn prev_node(&mut self) {
        if !self.nodes.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.nodes.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}
