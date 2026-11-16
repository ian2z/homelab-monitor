use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_rate_bytes_sec: u64,
    pub tx_rate_bytes_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub hostname: String,
    pub os_name: String,
    pub uptime_secs: u64,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub cpu_name: String,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub load_average: [f64; 3],
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_refresh_ms")]
    pub refresh_rate_ms: u64,
    #[serde(default)]
    pub nodes: Vec<HostConfig>,
}

fn default_refresh_ms() -> u64 {
    1000
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: 1000,
            nodes: vec![HostConfig {
                name: "Localhost".to_string(),
                url: "http://127.0.0.1:9191".to_string(),
            }],
        }
    }
}
