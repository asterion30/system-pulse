use crate::db::{DatabaseManager, SnapshotRecord, ThresholdConfig};
use crate::telemetry::SystemMetrics;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct ThresholdEngine {
    db: DatabaseManager,
    last_trigger_time: Arc<Mutex<Option<Instant>>>,
}

impl ThresholdEngine {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db,
            last_trigger_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn evaluate(&self, metrics: &SystemMetrics, config: &ThresholdConfig) -> bool {
        if !config.auto_capture_enabled {
            return false;
        }

        let mut triggered = false;
        let mut reason = String::new();

        if metrics.cpu_usage_total >= config.cpu_threshold {
            triggered = true;
            reason = format!("CPU ({:.1}%) superó el umbral ({:.1}%)", metrics.cpu_usage_total, config.cpu_threshold);
        } else if metrics.memory_percent >= config.memory_threshold {
            triggered = true;
            reason = format!("Memoria ({:.1}%) superó el umbral ({:.1}%)", metrics.memory_percent, config.memory_threshold);
        }

        if triggered {
            let mut last_time = self.last_trigger_time.lock().unwrap();
            let should_save = match *last_time {
                Some(t) => t.elapsed() > Duration::from_secs(30), // Cooldown 30s
                None => true,
            };

            if should_save {
                *last_time = Some(Instant::now());

                let top_proc = metrics
                    .top_cpu_processes
                    .first()
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "N/A".to_string());

                let metrics_json = serde_json::to_string(metrics).unwrap_or_default();

                let record = SnapshotRecord {
                    id: None,
                    timestamp: metrics.timestamp.clone(),
                    trigger_type: format!("THRESHOLD_AUTO ({})", reason),
                    cpu_usage: metrics.cpu_usage_total,
                    memory_percent: metrics.memory_percent,
                    status_level: metrics.status_level.clone(),
                    top_process_name: top_proc,
                    metrics_json,
                };

                let _ = self.db.save_snapshot(&record);
                return true;
            }
        }

        false
    }
}
