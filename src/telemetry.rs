use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, ProcessesToUpdate, RefreshKind, System};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub memory_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub core_id: usize,
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f32,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: String,
    pub hostname: String,
    pub os_name: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,

    pub cpu_usage_total: f32,
    pub cpu_cores: Vec<CpuCoreInfo>,
    pub cpu_count: usize,

    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_percent: f32,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_percent: f32,

    pub disks: Vec<DiskUsageInfo>,
    pub total_disk_read_speed_mb: f32,
    pub total_disk_write_speed_mb: f32,

    pub top_cpu_processes: Vec<ProcessInfo>,
    pub top_memory_processes: Vec<ProcessInfo>,

    pub status_level: String, // "NORMAL", "WARNING", "CRITICAL"
}

pub struct TelemetryCollector {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    _last_disk_sample_time: Arc<Mutex<Instant>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();

        Self {
            sys: Arc::new(Mutex::new(sys)),
            disks: Arc::new(Mutex::new(disks)),
            _last_disk_sample_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn collect(&self) -> SystemMetrics {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut disks_lock = self.disks.lock().unwrap();
        disks_lock.refresh(true);

        let now = chrono::Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let hostname = System::host_name().unwrap_or_else(|| "Linux-Node".to_string());
        let os_name = System::name().unwrap_or_else(|| "Linux".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let uptime_seconds = System::uptime();

        // CPU Metrics
        let cpu_cores: Vec<CpuCoreInfo> = sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(idx, cpu)| CpuCoreInfo {
                core_id: idx,
                usage: cpu.cpu_usage(),
            })
            .collect();

        let cpu_usage_total = if !cpu_cores.is_empty() {
            cpu_cores.iter().map(|c| c.usage).sum::<f32>() / cpu_cores.len() as f32
        } else {
            0.0
        };

        // Memory Metrics
        let total_mem_bytes = sys.total_memory();
        let used_mem_bytes = sys.used_memory();
        let memory_total_mb = total_mem_bytes / (1024 * 1024);
        let memory_used_mb = used_mem_bytes / (1024 * 1024);
        let memory_percent = if total_mem_bytes > 0 {
            (used_mem_bytes as f32 / total_mem_bytes as f32) * 100.0
        } else {
            0.0
        };

        let total_swap_bytes = sys.total_swap();
        let used_swap_bytes = sys.used_swap();
        let swap_total_mb = total_swap_bytes / (1024 * 1024);
        let swap_used_mb = used_swap_bytes / (1024 * 1024);
        let swap_percent = if total_swap_bytes > 0 {
            (used_swap_bytes as f32 / total_swap_bytes as f32) * 100.0
        } else {
            0.0
        };

        // Disks Metrics
        let mut disk_list = Vec::new();
        for disk in disks_lock.iter() {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total.saturating_sub(free);
            let percent = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            disk_list.push(DiskUsageInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                free_bytes: free,
                usage_percent: percent,
                read_bytes_per_sec: 0,
                write_bytes_per_sec: 0,
            });
        }

        // Top Processes
        let mut all_procs: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let proc_mem = proc.memory();
                let mem_pct = if total_mem_bytes > 0 {
                    (proc_mem as f32 / total_mem_bytes as f32) * 100.0
                } else {
                    0.0
                };

                ProcessInfo {
                    pid: pid.as_u32(),
                    name: proc.name().to_string_lossy().to_string(),
                    cpu_usage: proc.cpu_usage(),
                    memory_mb: proc_mem / (1024 * 1024),
                    memory_percent: mem_pct,
                }
            })
            .collect();

        // Sort for Top 5 CPU
        let mut top_cpu_processes = all_procs.clone();
        top_cpu_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        top_cpu_processes.truncate(5);

        // Sort for Top 5 Memory
        all_procs.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
        all_procs.truncate(5);
        let top_memory_processes = all_procs;

        // Determine Overall Status Level
        let status_level = if cpu_usage_total > 90.0 || memory_percent > 90.0 {
            "CRITICAL".to_string()
        } else if cpu_usage_total > 75.0 || memory_percent > 80.0 {
            "WARNING".to_string()
        } else {
            "NORMAL".to_string()
        };

        SystemMetrics {
            timestamp,
            hostname,
            os_name,
            kernel_version,
            uptime_seconds,
            cpu_usage_total,
            cpu_cores,
            cpu_count: sys.cpus().len(),
            memory_total_mb,
            memory_used_mb,
            memory_percent,
            swap_total_mb,
            swap_used_mb,
            swap_percent,
            disks: disk_list,
            total_disk_read_speed_mb: 0.0,
            total_disk_write_speed_mb: 0.0,
            top_cpu_processes,
            top_memory_processes,
            status_level,
        }
    }
}
