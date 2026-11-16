use crate::models::{DiskInfo, NetworkInfo, NodeMetrics};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, Networks, System};
use tokio::sync::Mutex;

pub struct SystemCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            networks,
        }
    }

    pub fn collect(&mut self) -> NodeMetrics {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);

        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os_name = System::name().unwrap_or_else(|| "Linux".to_string());
        let uptime_secs = System::uptime();

        let cpu_usage = self.sys.global_cpu_usage();
        let cpus = self.sys.cpus();
        let cpu_cores = cpus.len();
        let cpu_name = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "CPU".to_string());

        let memory_used_bytes = self.sys.used_memory();
        let memory_total_bytes = self.sys.total_memory();
        let swap_used_bytes = self.sys.used_swap();
        let swap_total_bytes = self.sys.total_swap();

        let mut seen_mounts = std::collections::HashSet::new();
        let disks = self
            .disks
            .iter()
            .filter(|d| d.total_space() > 0)
            .filter_map(|d| {
                let mount = d.mount_point().to_string_lossy().to_string();
                if seen_mounts.insert(mount.clone()) {
                    Some(DiskInfo {
                        name: d.name().to_string_lossy().to_string(),
                        mount_point: mount,
                        total_bytes: d.total_space(),
                        used_bytes: d.total_space().saturating_sub(d.available_space()),
                    })
                } else {
                    None
                }
            })
            .collect();

        let networks = self
            .networks
            .iter()
            .map(|(name, data)| NetworkInfo {
                interface_name: name.clone(),
                rx_bytes: data.total_received(),
                tx_bytes: data.total_transmitted(),
                rx_rate_bytes_sec: data.received(),
                tx_rate_bytes_sec: data.transmitted(),
            })
            .collect();

        let load_avg = System::load_average();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        NodeMetrics {
            hostname,
            os_name,
            uptime_secs,
            cpu_usage,
            cpu_cores,
            cpu_name,
            memory_used_bytes,
            memory_total_bytes,
            swap_used_bytes,
            swap_total_bytes,
            disks,
            networks,
            load_average: [load_avg.one, load_avg.five, load_avg.fifteen],
            timestamp,
        }
    }
}

pub type SharedCollector = Arc<Mutex<SystemCollector>>;
