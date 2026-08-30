use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub memory_percent: f32,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
    pub disk_total_read_bytes: u64,
    pub disk_total_written_bytes: u64,
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
    pub top_disk_processes: Vec<ProcessInfo>,

    pub status_level: String, // "NORMAL", "WARNING", "CRITICAL"
}

struct DiskSample {
    time: Instant,
    stats: HashMap<String, (u64, u64)>, // device_name -> (read_bytes, written_bytes)
}

pub struct TelemetryCollector {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    last_disk_sample: Arc<Mutex<Option<DiskSample>>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let initial_stats = Self::read_diskstats();

        Self {
            sys: Arc::new(Mutex::new(sys)),
            disks: Arc::new(Mutex::new(disks)),
            last_disk_sample: Arc::new(Mutex::new(Some(DiskSample {
                time: Instant::now(),
                stats: initial_stats,
            }))),
        }
    }

    fn read_diskstats() -> HashMap<String, (u64, u64)> {
        let mut map = HashMap::new();
        if let Ok(content) = fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 10 {
                    let devname = fields[2].to_string();
                    if (devname.starts_with("sd") && devname.len() == 3)
                        || (devname.starts_with("vd") && devname.len() == 3)
                        || (devname.starts_with("nvme") && !devname.contains('p'))
                        || (devname.starts_with("hd") && devname.len() == 3)
                        || (devname.starts_with("mmcblk") && !devname.contains('p'))
                    {
                        let sectors_read: u64 = fields[5].parse().unwrap_or(0);
                        let sectors_written: u64 = fields[9].parse().unwrap_or(0);
                        let read_bytes = sectors_read.saturating_mul(512);
                        let written_bytes = sectors_written.saturating_mul(512);
                        map.insert(devname, (read_bytes, written_bytes));
                    }
                }
            }
        }
        map
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

        // Disk Throughput calculation
        let now_instant = Instant::now();
        let current_diskstats = Self::read_diskstats();
        let mut last_sample_lock = self.last_disk_sample.lock().unwrap();

        let mut dev_rates: HashMap<String, (u64, u64)> = HashMap::new(); // dev -> (read_Bps, write_Bps)
        let mut total_read_bps: u64 = 0;
        let mut total_write_bps: u64 = 0;

        if let Some(prev) = last_sample_lock.as_ref() {
            let elapsed_secs = now_instant.duration_since(prev.time).as_secs_f64().max(0.1);
            for (dev, (curr_r, curr_w)) in &current_diskstats {
                if let Some((prev_r, prev_w)) = prev.stats.get(dev) {
                    let r_diff = curr_r.saturating_sub(*prev_r);
                    let w_diff = curr_w.saturating_sub(*prev_w);
                    let r_rate = (r_diff as f64 / elapsed_secs) as u64;
                    let w_rate = (w_diff as f64 / elapsed_secs) as u64;
                    dev_rates.insert(dev.clone(), (r_rate, w_rate));
                    total_read_bps += r_rate;
                    total_write_bps += w_rate;
                }
            }
        }

        *last_sample_lock = Some(DiskSample {
            time: now_instant,
            stats: current_diskstats,
        });

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

            let name_str = disk.name().to_string_lossy().to_string();
            // Find rates matching device name (e.g. sda, nvme0n1)
            let mut r_rate = 0;
            let mut w_rate = 0;
            for (dev, (r, w)) in &dev_rates {
                if name_str.contains(dev) || dev.contains(&name_str) {
                    r_rate = *r;
                    w_rate = *w;
                    break;
                }
            }

            disk_list.push(DiskUsageInfo {
                name: name_str,
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                free_bytes: free,
                usage_percent: percent,
                read_bytes_per_sec: r_rate,
                write_bytes_per_sec: w_rate,
            });
        }

        // Processes (with CPU, RAM, Disk I/O)
        let all_procs: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let proc_mem = proc.memory();
                let mem_pct = if total_mem_bytes > 0 {
                    (proc_mem as f32 / total_mem_bytes as f32) * 100.0
                } else {
                    0.0
                };

                let disk_usage = proc.disk_usage();

                ProcessInfo {
                    pid: pid.as_u32(),
                    name: proc.name().to_string_lossy().to_string(),
                    cpu_usage: proc.cpu_usage(),
                    memory_mb: proc_mem / (1024 * 1024),
                    memory_percent: mem_pct,
                    disk_read_bytes: disk_usage.read_bytes,
                    disk_written_bytes: disk_usage.written_bytes,
                    disk_total_read_bytes: disk_usage.total_read_bytes,
                    disk_total_written_bytes: disk_usage.total_written_bytes,
                }
            })
            .collect();

        // Fallback for disk total speeds if /proc/diskstats was 0
        if total_read_bps == 0 && total_write_bps == 0 {
            for p in &all_procs {
                total_read_bps += p.disk_read_bytes;
                total_write_bps += p.disk_written_bytes;
            }
        }

        let total_disk_read_speed_mb = (total_read_bps as f32) / (1024.0 * 1024.0);
        let total_disk_write_speed_mb = (total_write_bps as f32) / (1024.0 * 1024.0);

        // Sort for Top 5 CPU
        let mut top_cpu_processes = all_procs.clone();
        top_cpu_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        top_cpu_processes.truncate(5);

        // Sort for Top 5 Memory
        let mut top_memory_processes = all_procs.clone();
        top_memory_processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
        top_memory_processes.truncate(5);

        // Sort for Top 5 Disk I/O (active first, fallback to cumulative)
        let mut top_disk_processes = all_procs;
        top_disk_processes.sort_by(|a, b| {
            let a_active = a.disk_read_bytes + a.disk_written_bytes;
            let b_active = b.disk_read_bytes + b.disk_written_bytes;
            if a_active != b_active {
                b_active.cmp(&a_active)
            } else {
                let a_total = a.disk_total_read_bytes + a.disk_total_written_bytes;
                let b_total = b.disk_total_read_bytes + b.disk_total_written_bytes;
                b_total.cmp(&a_total)
            }
        });
        top_disk_processes.truncate(5);

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
            total_disk_read_speed_mb,
            total_disk_write_speed_mb,
            top_cpu_processes,
            top_memory_processes,
            top_disk_processes,
            status_level,
        }
    }
}

