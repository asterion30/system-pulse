use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub id: Option<i64>,
    pub timestamp: String,
    pub trigger_type: String, // "MANUAL", "THRESHOLD_AUTO", "SCHEDULED"
    pub cpu_usage: f32,
    pub memory_percent: f32,
    pub status_level: String,
    pub top_process_name: String,
    pub metrics_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub cpu_threshold: f32,       // e.g. 90.0
    pub memory_threshold: f32,    // e.g. 85.0
    pub disk_io_threshold_mb: f32,// e.g. 100.0
    pub auto_capture_enabled: bool,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            cpu_threshold: 90.0,
            memory_threshold: 85.0,
            disk_io_threshold_mb: 100.0,
            auto_capture_enabled: true,
        }
    }
}

#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let manager = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        manager.init_tables()?;
        Ok(manager)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                trigger_type TEXT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_percent REAL NOT NULL,
                status_level TEXT NOT NULL,
                top_process_name TEXT NOT NULL,
                metrics_json TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS threshold_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                cpu_threshold REAL NOT NULL,
                memory_threshold REAL NOT NULL,
                disk_io_threshold_mb REAL NOT NULL,
                auto_capture_enabled INTEGER NOT NULL
            );",
            [],
        )?;

        // Insert default config if empty
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM threshold_config",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let def = ThresholdConfig::default();
            conn.execute(
                "INSERT INTO threshold_config (id, cpu_threshold, memory_threshold, disk_io_threshold_mb, auto_capture_enabled)
                 VALUES (1, ?1, ?2, ?3, ?4)",
                params![def.cpu_threshold, def.memory_threshold, def.disk_io_threshold_mb, def.auto_capture_enabled as i32],
            )?;
        }

        Ok(())
    }

    pub fn save_snapshot(&self, snapshot: &SnapshotRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO snapshots (timestamp, trigger_type, cpu_usage, memory_percent, status_level, top_process_name, metrics_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.timestamp,
                snapshot.trigger_type,
                snapshot.cpu_usage,
                snapshot.memory_percent,
                snapshot.status_level,
                snapshot.top_process_name,
                snapshot.metrics_json
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_snapshots(&self, limit: usize) -> Result<Vec<SnapshotRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, trigger_type, cpu_usage, memory_percent, status_level, top_process_name, metrics_json
             FROM snapshots ORDER BY id DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(SnapshotRecord {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                trigger_type: row.get(2)?,
                cpu_usage: row.get(3)?,
                memory_percent: row.get(4)?,
                status_level: row.get(5)?,
                top_process_name: row.get(6)?,
                metrics_json: row.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_config(&self) -> Result<ThresholdConfig> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT cpu_threshold, memory_threshold, disk_io_threshold_mb, auto_capture_enabled FROM threshold_config WHERE id = 1",
            [],
            |row| {
                let auto_cap: i32 = row.get(3)?;
                Ok(ThresholdConfig {
                    cpu_threshold: row.get(0)?,
                    memory_threshold: row.get(1)?,
                    disk_io_threshold_mb: row.get(2)?,
                    auto_capture_enabled: auto_cap != 0,
                })
            },
        )
    }

    pub fn update_config(&self, cfg: &ThresholdConfig) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE threshold_config SET cpu_threshold = ?1, memory_threshold = ?2, disk_io_threshold_mb = ?3, auto_capture_enabled = ?4 WHERE id = 1",
            params![cfg.cpu_threshold, cfg.memory_threshold, cfg.disk_io_threshold_mb, cfg.auto_capture_enabled as i32],
        )?;
        Ok(())
    }
}
